#[path = "src/icon_bitmap.rs"]
mod icon_bitmap;

fn main() {
    #[cfg(windows)]
    {
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let ico_path = out_dir.join("app.ico");
        let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
        for &sz in &[32u32, 64u32] {
            let rgba = icon_bitmap::icon_rgba(sz);
            let image = ico::IconImage::from_rgba_data(sz, sz, rgba);
            let entry = ico::IconDirEntry::encode(&image).expect("ico encode");
            dir.add_entry(entry);
        }
        let mut f = std::fs::File::create(&ico_path).expect("create app.ico");
        dir.write(&mut f).expect("write app.ico");

        let mut res = winres::WindowsResource::new();
        res.set_icon(ico_path.to_str().expect("utf8 path"));
        res.compile().expect("winres compile");

        // Static FFmpeg (vcpkg) links avdevice (DirectShow) and avcodec (Media Foundation) but
        // those .lib files do not pull in the GUID/import libraries for the final exe link.
        println!("cargo:rustc-link-lib=strmiids");
        println!("cargo:rustc-link-lib=mfuuid");
        println!("cargo:rustc-link-lib=mfplat");
        println!("cargo:rustc-link-lib=mf");
    }
}
