//! `ffmpeg-sys-next` adds Windows system DLLs when using vcpkg; with `FFMPEG_DIR` it does not.
//! Full FFmpeg links transitive static deps for libwebp; slim omits them.

use std::path::PathBuf;

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os != "windows" {
        return;
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");
    println!("cargo:rerun-if-changed=Cargo.toml");

    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=secur32");
    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=bcrypt");
    println!("cargo:rustc-link-lib=user32");

    let Some(dir) = std::env::var_os("FFMPEG_DIR") else {
        return;
    };
    let lib_dir = PathBuf::from(dir).join("lib");
    if !lib_dir.is_dir() {
        return;
    }
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    fn link_first_present(lib_dir: &std::path::Path, candidates: &[&str]) {
        for stem in candidates {
            if lib_dir.join(format!("{stem}.lib")).is_file() {
                println!("cargo:rustc-link-lib=static={stem}");
                return;
            }
        }
    }

    let slim = std::env::vars()
        .any(|(k, v)| k == "CARGO_FEATURE_SLIM" && v == "1");
    if slim {
        link_first_present(&lib_dir, &["zlib"]);
        return;
    }

    link_first_present(&lib_dir, &["zlib"]);
    link_first_present(&lib_dir, &["sharpyuv", "libsharpyuv"]);
    link_first_present(&lib_dir, &["webp", "libwebp"]);
    link_first_present(&lib_dir, &["webpmux", "libwebpmux"]);
}
