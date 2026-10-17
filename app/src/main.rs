use std::borrow::Cow;
use std::fs;

use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use sph_shared::{Particle, SimParams, MAX_PARTICLES};
use wgpu::util::DeviceExt;

/// Ячейка сетки для SPH.
const GRID_SIZE: u32 = 32;
const NUM_CELLS: u32 = GRID_SIZE * GRID_SIZE * GRID_SIZE;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuParticle {
    position: [f32; 3],
    mass: f32,
}

impl From<Particle> for GpuParticle {
    fn from(p: Particle) -> Self {
        Self {
            position: [p.position.x, p.position.y, p.position.z],
            mass: p.mass,
        }
    }
}

fn main() {
    env_logger::init();

    let shader_path = env!("SHADER_PATH");
    let spirv_bytes = fs::read(shader_path).expect("Не удалось прочитать SPIR-V");

    pollster::block_on(run(&spirv_bytes));
}

async fn run(spirv_bytes: &[u8]) {
    // --- Инициализация wgpu ---
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("Не удалось найти адаптер");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("SPH Device"),
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    max_storage_buffers_per_shader_stage: 9,
                    ..Default::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("Не удалось создать устройство");

    // --- Генерация начальных частиц ---
    let mut particles: Vec<GpuParticle> = Vec::with_capacity(MAX_PARTICLES);
    for i in 0..MAX_PARTICLES {
        let x = ((i % 20) as f32 - 10.0) * 0.08;
        let y = (((i / 20) % 20) as f32 - 10.0) * 0.08 + 0.5;
        let z = ((i / 400) as f32 - 2.0) * 0.08;
        particles.push(GpuParticle {
            position: [x, y, z],
            mass: 0.02,
        });
    }

    let mut velocities = vec![Vec4::ZERO; MAX_PARTICLES];
    let mut densities = vec![0.0f32; MAX_PARTICLES];
    let mut pressures = vec![0.0f32; MAX_PARTICLES];
    let mut forces = vec![Vec4::ZERO; MAX_PARTICLES];
    let mut cell_counts = vec![0u32; NUM_CELLS as usize];
    let mut cell_start = vec![0u32; NUM_CELLS as usize];
    let mut cell_end = vec![0u32; NUM_CELLS as usize];
    let mut sorted_indices = vec![0u32; MAX_PARTICLES];

    // --- Создание буферов ---
    let particles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Particles"),
        contents: bytemuck::cast_slice(&particles),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });

    let velocities_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Velocities"),
        contents: bytemuck::cast_slice(&velocities),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let densities_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Densities"),
        contents: bytemuck::cast_slice(&densities),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let pressures_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Pressures"),
        contents: bytemuck::cast_slice(&pressures),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let forces_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Forces"),
        contents: bytemuck::cast_slice(&forces),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let cell_counts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("CellCounts"),
        contents: bytemuck::cast_slice(&cell_counts),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let cell_start_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("CellStart"),
        contents: bytemuck::cast_slice(&cell_start),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let cell_end_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("CellEnd"),
        contents: bytemuck::cast_slice(&cell_end),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let sorted_indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("SortedIndices"),
        contents: bytemuck::cast_slice(&sorted_indices),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    // --- Шейдерный модуль ---
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SPH Shader"),
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(bytemuck::cast_slice(spirv_bytes))),
    });

    // --- Bind Group Layout ---
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("SPH Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    // Для краткости создаём один bind group, покрывающий все binding'и
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("SPH Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: particles_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: velocities_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: densities_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: pressures_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: forces_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: cell_counts_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: cell_start_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: cell_end_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: sorted_indices_buffer.as_entire_binding(),
            },
        ],
    });

    // --- Параметры симуляции ---
    let mut params = SimParams::default();
    params.num_particles = MAX_PARTICLES as u32;

    // --- Цикл симуляции ---
    let num_steps = 100;
    for step in 0..num_steps {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SPH Encoder"),
        });

        // Здесь должны быть dispatch'и для каждого прохода:
        // clear_cells, count_particles, compute_density_pressure,
        // compute_forces, integrate
        //
        // Для краткости показан только один пример:
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SPH Compute Pass"),
                timestamp_writes: None,
            });
            // cpass.set_pipeline(&clear_cells_pipeline);
            // cpass.set_bind_group(0, &bind_group, &[]);
            // cpass.set_push_constants(wgpu::ShaderStages::COMPUTE, 0, bytemuck::bytes_of(&params));
            // cpass.dispatch_workgroups((NUM_CELLS + 255) / 256, 1, 1);
        }

        queue.submit(Some(encoder.finish()));

        if step % 10 == 0 {
            println!("Шаг {}/{}", step, num_steps);
        }
    }

    // --- Чтение результатов обратно на CPU ---
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output"),
        size: (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &particles_buffer,
        0,
        &output_buffer,
        0,
        (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
    );
    queue.submit(Some(encoder.finish()));

    let slice = output_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();

    let data = slice.get_mapped_range();
    let result: &[GpuParticle] = bytemuck::cast_slice(&data);

    println!("Первая частица после симуляции: {:?}", result[0].position);
    println!("Последняя частица: {:?}", result[MAX_PARTICLES - 1].position);
}
