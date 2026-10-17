use spirv_builder::{SpirvBuilder, SpirvMetadata};
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shader_dir = manifest_dir.parent().unwrap().join("shader");

    let result = SpirvBuilder::new(shader_dir, "spirv-unknown-vulkan1.2")
        .spirv_metadata(SpirvMetadata::Full)
        .build()
        .unwrap();

    println!("cargo:rustc-env=SHADER_PATH={}", result.module.unwrap_single().display());
}
