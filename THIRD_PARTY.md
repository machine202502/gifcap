# Third-party notices

The source in this repository is MIT. See LICENSE.

A release executable statically links FFmpeg (libavcodec, libavformat, libavutil, libswscale) built from the overlay in `vcpkg-overlays/ffmpeg` (FFmpeg 8.0.1, tag `n8.0.1`). That build does not enable GPL-only features, so FFmpeg is used under the GNU LGPL v2.1 or later. The license text is COPYING. Versions and the Rust crate list for a given executable are written to THIRD-PARTY.txt when the zip is built.
