use rsph_lib::{runtime::Runtime, types::{backend::CPU, math_implementations::Math32, physics_model::Model}};

fn main() {
    Runtime::builder::<Math32>()
        .model(Model::Classical)
        .cpu_backend(CPU::with_single_thread());
}
