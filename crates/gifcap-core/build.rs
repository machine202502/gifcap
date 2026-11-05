//! `ffmpeg-sys-next` adds Windows system DLLs when using vcpkg; with `FFMPEG_DIR` it does not.
//! Link the static libraries named by the FFmpeg pkgconfig files that are actually installed.
//! A GIF-only prefix has zlib. A fuller prefix also has libwebp, x264, and the rest.

use std::collections::BTreeSet;
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
    // A fuller static FFmpeg still contains Media Foundation and Schannel objects.
    // The slim GIF prefix compiles those out; these import libs satisfy the fuller one.
    println!("cargo:rustc-link-lib=strmiids");
    println!("cargo:rustc-link-lib=mfuuid");
    println!("cargo:rustc-link-lib=mfplat");
    println!("cargo:rustc-link-lib=mf");
    println!("cargo:rustc-link-lib=crypt32");
    println!("cargo:rustc-link-lib=ncrypt");

    let Some(dir) = std::env::var_os("FFMPEG_DIR").map(PathBuf::from) else {
        return;
    };
    let lib_dir = dir.join("lib");
    if !lib_dir.is_dir() {
        return;
    }
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    let ffmpeg = [
        "avcodec",
        "avdevice",
        "avfilter",
        "avformat",
        "avutil",
        "swresample",
        "swscale",
    ];
    let mut libs = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(lib_dir.join("pkgconfig")) {
        for entry in entries.flatten() {
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            for line in text.lines() {
                let Some(rest) = line
                    .strip_prefix("Libs.private:")
                    .or_else(|| line.strip_prefix("Libs:"))
                else {
                    continue;
                };
                for flag in rest.split_whitespace() {
                    if let Some(name) = flag.strip_prefix("-l") {
                        if !ffmpeg.contains(&name) {
                            libs.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }
    for name in libs {
        if lib_dir.join(format!("{name}.lib")).is_file() {
            println!("cargo:rustc-link-lib=static={name}");
        }
    }
    // A fuller FFmpeg build references swresample from avcodec even when gifcap does not resample.
    if lib_dir.join("swresample.lib").is_file() {
        println!("cargo:rustc-link-lib=static=swresample");
    }
}
