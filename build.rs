use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 关键：项目根目录
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let shader_dir = manifest_dir.join("src").join("shaders");

    println!("cargo:rerun-if-changed=shaders/");

    let entries = std::fs::read_dir(&shader_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to read shader dir {}: {}\n\
                 项目根目录={}",
            shader_dir.display(),
            e,
            manifest_dir.display()
        )
    });

    for entry in entries {
        let path = entry.unwrap().path();

        let ext = match path.extension().and_then(|s| s.to_str()) {
            Some(e) => e,
            None => continue,
        };

        if ext != "vert" && ext != "frag" {
            continue;
        }

        let out = path.with_extension("spv");

        println!("cargo:rerun-if-changed={}", path.display());

        let status = Command::new("glslc")
            .arg(&path)
            .arg("-o")
            .arg(&out)
            .status()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to run glslc: {}\n\
                     请确认 Vulkan SDK 已安装且 glslc 在 PATH 中。",
                    e
                )
            });

        if !status.success() {
            panic!("glslc failed on {}", path.display());
        }
    }
}
