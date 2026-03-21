//! `ffmpeg-sys-next` adds Windows system DLLs when using vcpkg; with `FFMPEG_DIR` it does not.
//! It also skips transitive static deps — FFmpeg + `webp` needs libwebp (and mux / sharpyuv when present).

use std::path::PathBuf;

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os != "windows" {
        return;
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");

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

    // Order: dependency libs before dependents (static MSVC link).
    for stem in ["libsharpyuv", "libwebp", "libwebpmux", "zlib"] {
        if lib_dir.join(format!("{stem}.lib")).is_file() {
            println!("cargo:rustc-link-lib=static={stem}");
        }
    }
}
