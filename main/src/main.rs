use rsph_lib::{devices::CPU, runtime::Runtime, traits::Math};

struct NativeMath {}
impl Math for NativeMath {
    type Float = f32;
    type Float3 = f32;
}

fn main() {
    let _runtime: Runtime<CPU, NativeMath> = Runtime::<CPU, NativeMath>::builder()
        .configure_device(|cpu| cpu.with_workers(3))
        .build();
}
