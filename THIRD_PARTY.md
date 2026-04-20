# Third-Party Components and Licenses (gifcap)

The source code in this repository is distributed under **MIT**. See [`LICENSE`](LICENSE).

This file summarizes third-party licensing considerations for distributed binaries (`gifcap.exe`).

---

## FFmpeg

- **Used libraries:** `libavcodec`, `libavformat`, `libavutil`, `libswscale` (and optional dependencies in full builds, controlled by `vcpkg.json`).
- **Integration:** built via [vcpkg](https://github.com/microsoft/vcpkg) and linked statically.
- **Port version in this project:** `8.0.1` (see `vcpkg-overlays/full/ffmpeg/vcpkg.json` and `vcpkg-overlays/slim/ffmpeg/vcpkg.json`).
- **Upstream source:** <https://github.com/FFmpeg/FFmpeg> (`n8.0.1` is the matching tag for this setup).
- **License baseline:** LGPL (`LGPL-2.1-or-later`) for the default project configuration. If GPL-level FFmpeg features are enabled (for example `gpl`, `x264`), licensing obligations may change.

Reference:

- [FFmpeg legal page](https://ffmpeg.org/legal.html)
- [GNU LGPL v2.1 text](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html)

---

## Rust Dependencies

The project also includes crates from crates.io under their respective licenses.
To regenerate a complete dependency license report for the current `Cargo.lock`, use:

```bash
cargo install cargo-license
cargo license
```

---

## Distribution Reminder

When distributing binaries, satisfy both:

- **MIT** terms for this repository code.
- **FFmpeg license terms** for linked FFmpeg components.

These obligations apply together and must both be respected.
