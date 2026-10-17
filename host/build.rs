use std::process::Command;

fn main() {
    let shader_crate = "../shaders";
    let output_dir = "shaders-spirv";

    std::fs::create_dir_all(output_dir).unwrap();
    let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(output_dir);
    println!("cargo:rustc-env=SPIRV_DIR={}", output_dir.display());

    let status = Command::new("cargo")
        .args([
            "gpu",
            "build",
            "--auto-install-rust-toolchain",
            "--shader-crate",
            shader_crate,
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--multimodule",
        ])
        .status()
        .expect("Failed to run cargo gpu build");

    if !status.success() {
        panic!("cargo gpu build failed");
    }
}