use khal::backend::{Backend, Buffer, Encoder, GpuBackend, WebGpu};
use khal::Shader;
use shaders::AddAssign;

static SPIRV_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/shaders-spirv");

#[derive(Shader)]
pub struct GpuKernels {
    add_assign: AddAssign,
}

fn main() {
    pollster::block_on(async {
        let backend = GpuBackend::WebGpu(WebGpu::default().await.unwrap());
        let kernels = GpuKernels::from_backend(&backend).unwrap();

        let a_data = vec![1.0f32, 2.0, 3.0];
        let b_data = vec![10.0f32, 20.0, 30.0];

        let mut a = backend.init_buffer(&a_data, khal::BufferUsages::STORAGE | khal::BufferUsages::COPY_SRC).unwrap();
        let b = backend.init_buffer(&b_data, khal::BufferUsages::STORAGE).unwrap();

        let mut encoder = backend.begin_encoding();
        let mut pass = encoder.begin_pass("add_assign", None);
        kernels.add_assign.call(&mut pass, a.len(), &mut a, &b).unwrap();
        drop(pass);
        backend.submit(encoder).unwrap();

        let result = backend.slow_read_vec(&a).await.unwrap();
        println!("{result:?}"); // [11.0, 22.0, 33.0]
    });
}
