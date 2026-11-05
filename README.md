# gifcap

Windows program that records a GIF of the area you frame next to the toolbar.

The source in this repository is under the MIT license. A release build statically links FFmpeg under the GNU LGPL v2.1 or later, so the distributed executable is covered by the LGPL. `COPYING` is that license. `NOTICE` names the obligation. The generated `THIRD-PARTY.txt` lists the library versions and the Rust crates, and it travels with the zip. Source and downloads: https://github.com/machine202502/gifcap

## Build

Windows x64, a stable Rust toolchain, and the FFmpeg overlay in `vcpkg-overlays` (GIF codecs only). Point `FFMPEG_DIR` at the release prefix `vcpkg_installed\x64-windows-static-md-release`.

`scripts\package.ps1` builds the release executable and writes `dist\gifcap-<version>-windows-x64.zip`. Pushing a new `version` in `Cargo.toml` to `master` runs the same packaging on GitHub and publishes the zip when that version has no release yet.

## Test

```powershell
cargo test --release --workspace
```
