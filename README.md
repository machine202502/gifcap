# gifcap

Запись области экрана под панелью окна в **GIF**, **MP4** или **WebP** (запись в MP4, при сохранении — конвертация в WebP). Windows, Win32 + статически линкованный FFmpeg.

---

## Что нужно на машине

| Компонент | Зачем |
|-----------|--------|
| **Visual Studio** или Build Tools, нагрузка «Разработка классических приложений на C++», x64 | MSVC, линковка |
| **Clang для Windows** (VS Installer → отдельные компоненты → *C++ Clang Compiler for Windows*) или [LLVM](https://github.com/llvm/llvm-project/releases) | `libclang.dll` для bindgen (`ffmpeg-sys-next`) |
| **Rust** [rustup](https://rustup.rs/), toolchain **`stable-x86_64-pc-windows-msvc`** | `rustc -vV` → host/target `x86_64-pc-windows-msvc` |
| **vcpkg** — [клон репозитория](https://github.com/microsoft/vcpkg), `bootstrap-vcpkg.bat` | Сборка FFmpeg и libwebp |

Задай `LIBCLANG_PATH` на каталог с `libclang.dll` (часто `...\Microsoft Visual Studio\...\VC\Tools\Llvm\x64\bin`).

---

## Один раз: клон vcpkg

```bat
git clone https://github.com/microsoft/vcpkg.git C:\path\to\vcpkg
cd /d C:\path\to\vcpkg
bootstrap-vcpkg.bat
```

`VCPKG_ROOT` = этот каталог (где лежит `vcpkg.exe`). В клоне должен быть triplet **`triplets/community/x64-windows-static-md-release.cmake`**; если нет — `git pull` в vcpkg.

---

## Зависимости C++ (FFmpeg): только из корня репозитория gifcap

Список пакетов и фич задаёт **`vcpkg.json`**. Порт FFmpeg из репо подменяется **overlay** (`vcpkg-overlays/ffmpeg` + `vcpkg-configuration.json`) — урезанный набор кодеков под gifcap, меньший статический exe.

**Обязательно** выполнять установку из **корня gifcap** (где лежат `vcpkg.json` и `vcpkg-configuration.json`), иначе vcpkg уйдёт в classic mode без манифеста.

**CMD:**

```bat
set VCPKG_ROOT=C:\path\to\vcpkg
cd /d C:\path\to\gifcap
%VCPKG_ROOT%\vcpkg.exe install --triplet x64-windows-static-md-release
```

**Git Bash:**

```bash
export VCPKG_ROOT=/c/path/to/vcpkg
cd /c/path/to/gifcap
"$VCPKG_ROOT/vcpkg.exe" install --triplet x64-windows-static-md-release
```

Артефакты: `gifcap/vcpkg_installed/x64-windows-static-md-release/` (в `.gitignore`).

В **`.cargo/config.toml`** уже указаны `FFMPEG_DIR` на этот префикс и target `x86_64-pc-windows-msvc`. Для Cargo **не** выставляй `VCPKG_ROOT` на `.../gifcap/vcpkg_installed`.

Если сборка Rust не видит заголовки (`avutil.h` и т.п.) — проверь, что есть файл  
`vcpkg_installed\x64-windows-static-md-release\include\libavutil\avutil.h` и при необходимости задай явно:

```bat
set FFMPEG_DIR=C:\path\to\gifcap\vcpkg_installed\x64-windows-static-md-release
```

---

## Сборка Rust

```bat
cd /d C:\path\to\gifcap
set LIBCLANG_PATH=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin
cargo build --release -p gifcap
```

(Путь к LLVM подставь свой: издание Community/Professional и номер папки VS могут отличаться.)

Результат: `target\x86_64-pc-windows-msvc\release\gifcap.exe`.

После смены triplet, `FFMPEG_DIR` или vcpkg-overlay: `cargo clean`, затем снова `cargo build`.

---

## Overlay и обновление vcpkg

Файлы в **`vcpkg-overlays/ffmpeg`** — копия порта vcpkg с небольшим дополнением в `portfile.cmake`. Если обновляешь клон vcpkg и сборка FFmpeg ломается из‑за расхождения версий порта — подтяни в overlay актуальный `ports/ffmpeg` **с того же коммита**, что и твой `vcpkg.exe`.

---

## Пользование

- Запуск: `gifcap.exe`. Под панелью — зона захвата.
- Сохранённые файлы: `%USERPROFILE%\Pictures\gifcap\`.
- Во время записи: `%USERPROFILE%\.gifcap\active\` (`recording.gif` или `recording.mp4`).
- Лог: `%USERPROFILE%\.gifcap\logs\gifcap.log`.
