use std::path::PathBuf;

const HEADER_NAME:&str = "driver";
const LIB_NAME:&str = "cute_driver";

fn main() {
    cute_generate();

    println!("cargo:warning=cute-generate Success.");
    if let Err(e) = cute_linker() {
        println!("cargo:warning=cute-linker failed: {}", e);
    }
    println!("cargo:warning=cute-linker Success.");
}

fn cute_generate() {
    let root_path  = PathBuf::from("cute-driver").join("headers");
    let out_folder = "src/ffi/generated";
    let header_path = root_path.join("include");
    let bindings = bindgen::Builder::default()
        .header(format!("{}/{}.h", header_path.to_str().unwrap(), HEADER_NAME))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(format!("{}/cute_{}_generated.rs", out_folder, HEADER_NAME))
        .expect("Cannot write ffi to project");
}

fn cute_linker() -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        let lib_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").map_err(|err|std::io::Error::new(std::io::ErrorKind::Interrupted, err.to_string()))?)
            .join("cute-driver")
            .join("libs");

        if let Some(link_str) = lib_path.to_str() {
            println!("cargo:rustc-link-search=native={}", link_str);
            println!("cargo:rustc-link-lib=dylib={}", LIB_NAME);
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "lib path None!!!"))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let lib_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").map_err(|err|std::io::Error::new(std::io::ErrorKind::Interrupted, err.to_string()))?)
            .join("cute-driver")
            .join("libs");

        if let Some(link_str) = lib_path.to_str() {
            println!("cargo:rustc-link-search=native={}", link_str);
            println!("cargo:rustc-link-lib=dylib={}",crate::LIB_NAME);
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "lib path None!!!"))
        }
    }

    #[cfg(target_os = "windows")]
    {
        Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "This operating system is not supported!!!"))
    }
}