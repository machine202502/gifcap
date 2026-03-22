# Сторонние компоненты и лицензии (gifcap)

Код **этого репозитория** распространяется под **MIT** — см. файл [`LICENSE`](LICENSE).

Ниже — то, что важно для **готового `gifcap.exe`**, если ты отдаёшь бинарник другим людям.

---

## FFmpeg

- **Что используется:** библиотеки **libavcodec**, **libavformat**, **libavutil**, **libswscale** (и при **full**-сборке — опционально зависимости вроде **libwebp**, задаётся манифестом `vcpkg.json`).
- **Как подключается:** сборка через **[vcpkg](https://github.com/microsoft/vcpkg)** и **статическая** линковка в исполняемый файл.
- **Версия в портах проекта:** **8.0.1** (см. `vcpkg-overlays/full/ffmpeg/vcpkg.json` и `vcpkg-overlays/slim/ffmpeg/vcpkg.json`).
- **Исходники FFmpeg:** репозиторий <https://github.com/FFmpeg/FFmpeg>, для соответствия версии порта удобно взять тег **`n8.0.1`**.
- **Лицензия FFmpeg:** в типичной конфигурации **этого** проекта (в `vcpkg.json` для `ffmpeg` указано `"default-features": false` и только нужные фичи) основной код FFmpeg идёт под **GNU LGPL v2.1 or later** (LGPL-2.1-or-later). Если ты сам включишь в vcpkg фичи уровня **GPL** (например `gpl`, `x264` и т.п.), состав лицензий может измениться — смотри <https://ffmpeg.org/legal.html>.

Полный текст LGPL-2.1:  
<https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html>

### Если распространяешь собранный `.exe`

Краткий практический чеклист (не юридическая консультация):

1. Сохраняй **копирайты и уведомления** FFmpeg (как в исходниках / документации порта).
2. Указывай, что используется **FFmpeg** под **LGPL**, и давай ссылку на **исходники** той версии, с которой собирал (или эквивалентный способ получить исходный код).
3. При **статической** линковке LGPL предъявляет дополнительные требования (в духе возможности **заменить** библиотеку и **перелинковать** приложение). Обычно это оформляют как набор **объектных файлов** + инструкции/скрипты линковки вместе с релизом; детали — **в тексте LGPL-2.1**, раздел про статическую линковку.

В репозитории лежит краткий файл **[`NOTICE`](NOTICE)** — его можно класть рядом с `gifcap.exe` в архиве релиза.

---

## Rust-зависимости (crates.io)

Снимок по **`cargo license`** для текущего [`Cargo.lock`](Cargo.lock). После `cargo update` или смены зависимостей перегенерируй: `cargo install cargo-license && cargo license`.

### (Apache-2.0 OR MIT) AND OFL-1.1 AND LicenseRef-UFL-1.0

- `epaint_default_fonts`

### (Apache-2.0 OR MIT) AND Unicode-3.0

- `unicode-ident`

### 0BSD OR Apache-2.0 OR MIT

- `adler2`

### Apache-2.0

- `ab_glyph`, `ab_glyph_rasterizer`, `clang-sys`, `gethostname`, `gl_generator`, `glutin`, `glutin_egl_sys`, `glutin_glx_sys`, `glutin_wgl_sys`, `khronos_api`, `owned_ttf_parser`, `winit`

### Apache-2.0 AND MIT

- `dpi`

### Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT

- `linux-raw-sys`, `rustix`, `wasip2`, `wasip3`, `wasm-encoder`, `wasm-metadata`, `wasmparser`, `wit-bindgen`, `wit-bindgen-core`, `wit-bindgen-rust`, `wit-bindgen-rust-macro`, `wit-component`, `wit-parser`

### Apache-2.0 OR BSD-2-Clause OR MIT

- `zerocopy`, `zerocopy-derive`

### Apache-2.0 OR BSD-3-Clause

- `moxcms`, `pxfm`

### Apache-2.0 OR BSD-3-Clause OR MIT

- `num_enum`, `num_enum_derive`

### Apache-2.0 OR LGPL-2.1-or-later OR MIT

- `r-efi`

### Apache-2.0 OR MIT

`ahash`, `android-activity`, `android_system_properties`, `anyhow`, `arboard`, `as-raw-xcb-connection`, `atomic-waker`, `autocfg`, `bitflags`, `bumpalo`, `cc`, `cesu8`, `cexpr`, `cfg-if`, `cgl`, `chrono`, `concurrent-queue`, `core-foundation`, `core-foundation-sys`, `core-graphics`, `core-graphics-types`, `crc32fast`, `crossbeam-utils`, `displaydoc`, `document-features`, `downcast-rs`, `ecolor`, `eframe`, `egui`, `egui-winit`, `egui_glow`, `either`, `emath`, `epaint`, `equivalent`, `errno`, `fdeflate`, `find-msvc-tools`, `flate2`, `foreign-types`, `foreign-types-macros`, `foreign-types-shared`, `form_urlencoded`, `futures-core`, `futures-task`, `futures-util`, `getrandom`, `glob`, `hashbrown`, `heck`, `hermit-abi`, `iana-time-zone`, `iana-time-zone-haiku`, `id-arena`, `idna`, `idna_adapter`, `image`, `indexmap`, `itertools`, `itoa`, `jni`, `jni-macros`, `jni-sys`, `jni-sys-macros`, `jobserver`, `js-sys`, `leb128fmt`, `libc`, `litrs`, `lock_api`, `log`, `memmap2`, `minimal-lexical`, `ndk`, `ndk-context`, `ndk-sys`, `nohash-hasher`, `num-traits`, `num_cpus`, `once_cell`, `parking_lot`, `parking_lot_core`, `percent-encoding`, `pin-project`, `pin-project-internal`, `pin-project-lite`, `pkg-config`, `plain`, `png`, `polling`, `prettyplease`, `proc-macro-crate`, `proc-macro2`, `quote`, `regex`, `regex-automata`, `regex-syntax`, `rustc-hash`, `rustc_version`, `rustversion`, `scoped-tls`, `scopeguard`, `semver`, `serde`, `serde_core`, `serde_derive`, `serde_json`, `shlex`, `simd_cesu8`, `simdutf8`, `smallvec`, `smol_str`, `stable_deref_trait`, `static_assertions`, `syn`, `thiserror`, `thiserror-impl`, `toml`, `toml_datetime`, `toml_edit`, `toml_parser`, `ttf-parser`, `unicode-segmentation`, `unicode-xid`, `url`, `utf8_iter`, `uuid`, `vcpkg`, `version_check`, `wasm-bindgen`, `wasm-bindgen-futures`, `wasm-bindgen-macro`, `wasm-bindgen-macro-support`, `wasm-bindgen-shared`, `web-sys`, `web-time`, `webbrowser`, `winapi`, `winapi-i686-pc-windows-gnu`, `winapi-x86_64-pc-windows-gnu`, `windows`, `windows-core`, `windows-implement`, `windows-interface`, `windows-link`, `windows-result`, `windows-strings`, `windows-sys`, `windows-targets`, `windows_aarch64_gnullvm`, `windows_aarch64_msvc`, `windows_i686_gnu`, `windows_i686_gnullvm`, `windows_i686_msvc`, `windows_x86_64_gnu`, `windows_x86_64_gnullvm`, `windows_x86_64_msvc`, `x11rb`, `x11rb-protocol`

### Apache-2.0 OR MIT OR Zlib

- `bytemuck`, `bytemuck_derive`, `cursor-icon`, `dispatch2`, `glow`, `miniz_oxide`, `objc2-app-kit`, `objc2-core-foundation`, `objc2-core-graphics`, `objc2-io-surface`, `raw-window-handle`, `xkeysym`

### BSD-3-Clause

- `bindgen`

### BSL-1.0

- `clipboard-win`, `error-code`

### ISC

- `libloading`

### MIT

- `android-properties`, `block2`, `bytes`, `calloop`, `calloop-wayland-source`, `cfg_aliases`, `combine`, `dispatch`, `dlib`, `gifcap`, `gifcap-core`, `gifcap-windows`, `glutin-winit`, `ico`, `libredox`, `memoffset`, `nom`, `objc-sys`, `objc2`, `objc2-app-kit`, `objc2-cloud-kit`, `objc2-contacts`, `objc2-core-data`, `objc2-core-image`, `objc2-core-location`, `objc2-encode`, `objc2-foundation`, `objc2-link-presentation`, `objc2-metal`, `objc2-quartz-core`, `objc2-symbols`, `objc2-ui-kit`, `objc2-uniform-type-identifiers`, `objc2-user-notifications`, `orbclient`, `quick-xml`, `redox_syscall`, `simd-adler32`, `slab`, `smithay-client-toolkit`, `smithay-clipboard`, `synstructure`, `tracing`, `tracing-core`, `wayland-backend`, `wayland-client`, `wayland-csd-frame`, `wayland-cursor`, `wayland-protocols`, `wayland-protocols-experimental`, `wayland-protocols-misc`, `wayland-protocols-plasma`, `wayland-protocols-wlr`, `wayland-scanner`, `wayland-sys`, `winnow`, `winres`, `x11-dl`, `xcursor`, `xkbcommon-dl`, `xml-rs`, `zmij`

### MIT OR Unlicense

- `aho-corasick`, `byteorder`, `byteorder-lite`, `memchr`, `same-file`, `walkdir`, `winapi-util`

### Unicode-3.0

- `icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`, `icu_properties`, `icu_properties_data`, `icu_provider`, `litemap`, `potential_utf`, `tinystr`, `writeable`, `yoke`, `yoke-derive`, `zerofrom`, `zerofrom-derive`, `zerotrie`, `zerovec`, `zerovec-derive`

### WTFPL

- `ffmpeg-next`, `ffmpeg-sys-next` (обёртки над FFmpeg; сам FFmpeg — отдельно, см. выше)

### Zlib

- `foldhash`, `slotmap`

---

## MIT и LGPL

Лицензия **MIT** относится к **твоему коду** в этом репозитории. Она **не отменяет** условий **LGPL** для части, которая пришла из **FFmpeg** внутри бинарника: для exe действуют **оба слоя** — MIT (твой код) + LGPL (FFmpeg), каждый со своими правилами при распространении.
