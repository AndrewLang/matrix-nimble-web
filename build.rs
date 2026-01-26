use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest
        .join("examples")
        .join("light_server")
        .join("web.config.json");

    if !src.exists() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap_or(&out_dir).to_path_buf();
    let dst = target_dir.join("web.config.json");
    let example_dir = target_dir.join("examples");
    let dst_example = example_dir.join("web.config.json");

    println!("cargo:rerun-if-changed={}", src.display());
    let _ = fs::copy(&src, &dst);
    let _ = fs::create_dir_all(&example_dir);
    let _ = fs::copy(&src, &dst_example);
}
