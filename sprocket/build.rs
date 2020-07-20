fn path_to_name<'a>(path: &'a str) -> &'a str {
    let last_delimiter = match path.rfind("/") {
        Some(i) => i + 1,
        None => 0,
    };
    
    let dot = match path.rfind(".") {
        Some(i) if i < last_delimiter => path.len(),
        Some(i ) => i,
        None => path.len(),
    };

    println!("{}", &path[last_delimiter..dot]);
    &path[last_delimiter..dot]
}

fn main() {
    let header_libs = ["./vendor/stb_image.c"];

    for header_lib in header_libs.iter() {
        println!("cargo:rerun-if-changed={}", header_lib);
        cc::Build::new()
            .define("STB_IMAGE_IMPLEMENTATION", None)
            .define("STBI_NO_GIF", None)
            .file(header_lib)
            .warnings(false)
            .compile(path_to_name(header_lib));
    }
}
