use std::fs;
use std::path::{Path, PathBuf};

const HEADER_NAME:&str = "driver";

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
    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Failed to find target directory")
        .to_path_buf();

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
        let lib_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").map_err(|err|std::io::Error::new(std::io::ErrorKind::Interrupted, err.to_string()))?)
            .join("cute-driver")
            .join("libs");

        println!("cargo:rustc-link-search=native={}/", lib_path.display());
        println!("cargo:rustc-link-lib=static=cute_driver");
        if let Err(_) = copy_dir_all(&lib_path, &target_dir) {
            Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Failed to copy files!!!"))
        } else {
            Ok(())
        }
    }
}