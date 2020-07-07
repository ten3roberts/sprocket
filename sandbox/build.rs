use std::fs;
use std::path::Path;
use std::process;
pub fn compile_shaders<P: AsRef<Path> + std::fmt::Debug>(path: &P) {
    let entries = fs::read_dir(path).expect("Failed to read directory");
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(v) => v,
            Err(msg) => {
                panic!("Unable to get metadata for entry {:?}, {}", path, msg);
            }
        };
        let path = entry.path();

        if metadata.is_dir() {
            compile_shaders(&entry.path());
        }

        let filename = entry.file_name();
        let filename = match filename.to_str() {
            Some(filename) => filename,
            None => continue,
        };

        if filename.ends_with(".frag") || filename.ends_with(".vert") {
            match compile_shader(&path) {
                0 => {}
                code => {
                    eprintln!("Failed to compile shader {:?}, exit code {}", path, code);
                    process::exit(code)
                }
            };
        }
    }
}

fn compile_shader(path: &Path) -> i32 {
    let mut output_path = path.to_string_lossy().to_string();
    output_path += ".spv";
    let mut process = process::Command::new("glslc")
        .args(&["-o", &output_path, &path.to_string_lossy()])
        .spawn()
        .expect("Failed to spawn glslc process");

    // Wait on process to complete
    match process.wait() {
        Ok(status) => status.code().unwrap_or_else(|| {
            eprintln!("Failed to get exit code from child");
            2000
        }),
        Err(e) => {
            eprintln!("Failed to wait on child process '{}'", e);
            1000
        }
    }
}

fn main() {
    compile_shaders(&std::path::Path::new("./data"));
}
