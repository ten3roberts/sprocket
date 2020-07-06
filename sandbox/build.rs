use std::fs;
use std::process;

pub fn compile_shaders(path: &std::path::Path) {
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
            println!("Entering dir {:?}", path);
            compile_shaders(&entry.path());
        }

        let filename = entry.file_name();
        let filename = match filename.to_str() {
            Some(filename) => filename,
            None => continue,
        };

        if filename.ends_with(".frag") || filename.ends_with(".vert") {
            let mut output_path = path.to_string_lossy().to_owned();
            output_path += ".spv";
            process::Command::new("glslc")
                .args(&["-o", &output_path, &path.to_string_lossy()])
                .spawn()
                .expect("Failed to spawn glslc process");
        }
    }
}

fn main() {
    compile_shaders(&std::path::Path::new("./data"));
}
