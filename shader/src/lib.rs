#![cfg_attr(target_arch = "spirv", no_std)]
#![feature(asm_experimental_arch)]

use glam::{IVec3, UVec3, Vec3, Vec4};
use spirv_std::glam;
use spirv_std::num_traits::Float;
use spirv_std::spirv;

use sph_shared::{
    equation_of_state, kernel, kernel_grad, SimParams,
};

/// Хеш-функция для пространственной сетки.
#[inline]
fn hash_cell(c: IVec3) -> u32 {
    (c.x as u32).wrapping_mul(73856093)
        ^ (c.y as u32).wrapping_mul(19349663)
        ^ (c.z as u32).wrapping_mul(83492791)
}

/// Проход 1: очистка счётчиков ячеек.
#[spirv(compute(threads(256)))]
pub fn clear_cells(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] cell_counts: &mut [u32],
    #[spirv(push_constant)] params: &SimParams,
) {
    let idx = id.x as usize;
    if idx >= cell_counts.len() {
        return;
    }
    cell_counts[idx] = 0;
}

/// Проход 2: подсчёт частиц в ячейках (гистограмма).
#[spirv(compute(threads(256)))]
pub fn count_particles(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &[sph_shared::Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] cell_counts: &mut [u32],
    #[spirv(push_constant)] params: &SimParams,
) {
    let idx = id.x as usize;
    if idx >= params.num_particles as usize {
        return;
    }

    let pos = particles[idx].position;
    let cell = IVec3::new(
        (pos.x / params.h).floor() as i32,
        (pos.y / params.h).floor() as i32,
        (pos.z / params.h).floor() as i32,
    );
    let num_cells = cell_counts.len() as u32;
    let h = hash_cell(cell) % num_cells;

    cell_counts[h as usize] += 1;
}

/// Проход 3: вычисление плотности и давления.
#[spirv(compute(threads(256)))]
pub fn compute_density_pressure(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &[sph_shared::Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] densities: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] pressures: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] cell_start: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] cell_end: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] sorted_indices: &[u32],
    #[spirv(push_constant)] params: &SimParams,
) {
    let i = id.x as usize;
    if i >= params.num_particles as usize {
        return;
    }

    let pi = particles[i].position;
    let cell = IVec3::new(
        (pi.x / params.h).floor() as i32,
        (pi.y / params.h).floor() as i32,
        (pi.z / params.h).floor() as i32,
    );
    let num_cells = cell_start.len() as u32;

    let mut rho = 0.0f32;

    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            for dz in -1i32..=1 {
                let nc = IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz);
                let h = hash_cell(nc) % num_cells;
                let start = cell_start[h as usize];
                let end = cell_end[h as usize];

                for k in start..end {
                    let j = sorted_indices[k as usize] as usize;
                    let pj = particles[j].position;
                    let r = pi - pj;
                    let r2 = r.dot(r);
                    if r2 < params.h2 {
                        let w = kernel(r2.sqrt(), params.h);
                        rho += particles[j].mass * w;
                    }
                }
            }
        }
    }

    densities[i] = rho;
    pressures[i] = equation_of_state(rho, params.rest_density, params.stiffness);
}

/// Проход 4: вычисление сил (давление + вязкость + гравитация).
#[spirv(compute(threads(256)))]
pub fn compute_forces(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &[sph_shared::Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] velocities: &[Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] densities: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] pressures: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] forces: &mut [Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] cell_start: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] cell_end: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)] sorted_indices: &[u32],
    #[spirv(push_constant)] params: &SimParams,
) {
    let i = id.x as usize;
    if i >= params.num_particles as usize {
        return;
    }

    let pi = particles[i].position;
    let rho_i = densities[i];
    let p_i = pressures[i];
    let vi = velocities[i].truncate();

    if rho_i < 1e-6 {
        forces[i] = Vec4::new(0.0, params.gravity, 0.0, 0.0);
        return;
    }

    let cell = IVec3::new(
        (pi.x / params.h).floor() as i32,
        (pi.y / params.h).floor() as i32,
        (pi.z / params.h).floor() as i32,
    );
    let num_cells = cell_start.len() as u32;

    let mut acc = Vec3::ZERO;

    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            for dz in -1i32..=1 {
                let nc = IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz);
                let h = hash_cell(nc) % num_cells;
                let start = cell_start[h as usize];
                let end = cell_end[h as usize];

                for k in start..end {
                    let j = sorted_indices[k as usize] as usize;
                    if i == j {
                        continue;
                    }

                    let pj = particles[j].position;
                    let r = pi - pj;
                    let r_len = r.length();
                    if r_len >= params.h || r_len < 1e-6 {
                        continue;
                    }

                    let rho_j = densities[j];
                    let p_j = pressures[j];

                    // Сила давления
                    let pressure_term =
                        p_i / (rho_i * rho_i) + p_j / (rho_j * rho_j);
                    let grad_w = kernel_grad(r, r_len, params.h);
                    acc -= particles[j].mass * pressure_term * grad_w;

                    // Вязкость
                    let vj = velocities[j].truncate();
                    let vij = vi - vj;
                    if vij.dot(r) < 0.0 {
                        let visc = -params.viscosity
                            * particles[j].mass
                            * vij.dot(r)
                            / (r_len * r_len + 0.01 * params.h2);
                        acc -= visc * grad_w;
                    }
                }
            }
        }
    }

    // Гравитация
    acc.y += params.gravity;

    // Границы (простое отталкивание)
    let b = params.boundary;
    if pi.x < -b {
        acc.x += (b + pi.x) * 1000.0;
    }
    if pi.x > b {
        acc.x -= (pi.x - b) * 1000.0;
    }
    if pi.y < -b {
        acc.y += (b + pi.y) * 1000.0;
    }
    if pi.y > b {
        acc.y -= (pi.y - b) * 1000.0;
    }
    if pi.z < -b {
        acc.z += (b + pi.z) * 1000.0;
    }
    if pi.z > b {
        acc.z -= (pi.z - b) * 1000.0;
    }

    forces[i] = Vec4::new(acc.x, acc.y, acc.z, 0.0);
}

/// Проход 5: интегрирование (полушаг по скорости + полный шаг по позиции).
#[spirv(compute(threads(256)))]
pub fn integrate(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &mut [sph_shared::Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] velocities: &mut [Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] forces: &[Vec4],
    #[spirv(push_constant)] params: &SimParams,
) {
    let i = id.x as usize;
    if i >= params.num_particles as usize {
        return;
    }

    let mut v = velocities[i];
    v.x += forces[i].x * params.dt;
    v.y += forces[i].y * params.dt;
    v.z += forces[i].z * params.dt;
    velocities[i] = v;

    let mut p = particles[i].position;
    p.x += v.x * params.dt;
    p.y += v.y * params.dt;
    p.z += v.z * params.dt;
    particles[i].position = p;
}
