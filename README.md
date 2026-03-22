# gifcap

## 1. Что это

Запись прямоугольной области экрана **под** панелью окна приложения.

| Сборка | Запись | Экспорт |
|--------|--------|---------|
| **full** | GIF или MP4 | WebP из MP4 при сохранении |
| **slim** | только GIF | PNG скрин кнопкой Screen (`image`, без WebP в FFmpeg) |

**Только Windows**, Win32 + статический FFmpeg (через vcpkg). Других ОС нет.

---

## 2. Как пользоваться

- Запуск: `gifcap.exe`.
- Под верхней панелью — зона захвата.
- Готовые файлы: `%USERPROFILE%\Pictures\gifcap\`.
- Пока идёт запись: `%USERPROFILE%\.gifcap\active\` — `recording.gif` (slim и режим GIF в full) или `recording.mp4` (full, MP4/WebP).
- Лог: `%USERPROFILE%\.gifcap\logs\gifcap.log`.

---

## 3. Зависимости

| Что | Зачем | Откуда |
|-----|--------|--------|
| **Git for Windows** | Git Bash, `cygpath` для путей `C:\...` в `vcpkg.exe` | [git-scm.com](https://git-scm.com/download/win) |
| **Visual Studio** или Build Tools, workload **Desktop development with C++**, **x64** | MSVC, линковка | [Visual Studio](https://visualstudio.microsoft.com/) |
| **Clang / LLVM** (VS: компонент *C++ Clang Compiler for Windows*, или отдельный [LLVM](https://github.com/llvm/llvm-project/releases)) | каталог с **`libclang.dll`** для bindgen | см. `LIBCLANG_PATH` ниже |
| **Rust**, toolchain **`stable-x86_64-pc-windows-msvc`** | `cargo`, `rustc` | [rustup.rs](https://rustup.rs/) → `rustup default stable-x86_64-pc-windows-msvc` |
| **vcpkg** (клон репо, `bootstrap-vcpkg.bat`) | сборка FFmpeg | [github.com/microsoft/vcpkg](https://github.com/microsoft/vcpkg) |

В клоне vcpkg должен быть triplet **`triplets/community/x64-windows-static-md-release.cmake`** (старый клон — `git pull`).

**Обязательно задать перед сборкой** (и скрипты это проверяют):

```bash
export VCPKG_ROOT=/c/путь/к/vcpkg          # каталог, где лежит vcpkg.exe
export LIBCLANG_PATH="/c/Program Files/Microsoft Visual Studio/…/VC/Tools/Llvm/x64/bin"
```

Подставь реальный путь к каталогу с **`libclang.dll`** (версия VS/Build Tools у всех разная). `VCPKG_ROOT` — в стиле Git Bash (`/c/...`).

Дополнительно для **ручной** сборки slim без скрипта — см. п.4: **`FFMPEG_DIR`** на префикс `vcpkg_installed_slim/...` (скрипт выставляет сам).

---

## 4. Сборка вручную (Git Bash)

Рабочая копия = корень репозитория (там `vcpkg.json`). Команды `vcpkg.exe` — из Git Bash; пути для `--x-manifest-root`, `--x-install-root`, `VCPKG_OVERLAY_PORTS` должны быть **Windows** (`C:\...`) — через `cygpath -w`.

```bash
cd /c/путь/к/gifcap
export VCPKG_ROOT=/c/путь/к/vcpkg
export LIBCLANG_PATH="/c/.../Llvm/x64/bin"

WIN_ROOT="$(cygpath -w "$PWD")"
TRIPLET=x64-windows-static-md-release
```

### 4.1 vcpkg: full

```bash
export VCPKG_OVERLAY_PORTS="$(cygpath -w "$PWD/vcpkg-overlays/full")"
WIN_INSTALL="$(cygpath -w "$PWD/vcpkg_installed_full")"

"$VCPKG_ROOT/vcpkg.exe" install \
  --triplet "$TRIPLET" \
  --x-manifest-root="$WIN_ROOT" \
  --x-install-root="$WIN_INSTALL"
```

### 4.2 vcpkg: slim

```bash
export VCPKG_OVERLAY_PORTS="$(cygpath -w "$PWD/vcpkg-overlays/slim")"
WIN_INSTALL="$(cygpath -w "$PWD/vcpkg_installed_slim")"

"$VCPKG_ROOT/vcpkg.exe" install \
  --triplet "$TRIPLET" \
  --x-no-default-features \
  --x-manifest-root="$WIN_ROOT" \
  --x-install-root="$WIN_INSTALL"
```

### 4.3 Cargo

**Full** (префикс по умолчанию в `.cargo/config.toml` — `vcpkg_installed_full`; `FFMPEG_DIR` можно не трогать):

```bash
cargo clean
cargo build --release -p gifcap
```

**Slim** — укажи префикс slim и фичу:

```bash
export FFMPEG_DIR="$(cygpath -w "$PWD/vcpkg_installed_slim/$TRIPLET")"
cargo clean
cargo build --release -p gifcap --features slim
```

Артефакт: `target/x86_64-pc-windows-msvc/release/gifcap.exe`.

Сменил overlay, `FFMPEG_DIR` или triplet → снова **`cargo clean`**, потом build.

---

## 5. Сборка через bash-скрипты

Из корня репозитория, после `export VCPKG_ROOT` и `export LIBCLANG_PATH`:

```bash
./build-full.bash   # vcpkg_installed_full + cargo без --features slim
./build-slim.bash    # vcpkg_installed_slim + --x-no-default-features + cargo --features slim
```

Скрипты делают `cargo clean` и выставляют `VCPKG_OVERLAY_PORTS`, `FFMPEG_DIR`, профиль release через переменные `CARGO_PROFILE_RELEASE_*`.

---

## 6. Лицензии

Файлы **`LICENSE`**, **`NOTICE`**, **`THIRD_PARTY.md`** сгенерированы нейросетью; автор к ним отношения не имеет и не гарантирует юридическую корректность. Разбирайся сам.

Исходный код репозитория по задумке — **MIT** (`LICENSE`). В бинарник со vcpkg входит **FFmpeg** (лицензия FFmpeg/LGPL и т.д. — см. [ffmpeg.org/legal.html](https://ffmpeg.org/legal.html)).

---

## Обновление overlay под новый vcpkg

`vcpkg-overlays/full/ffmpeg` и `vcpkg-overlays/slim/ffmpeg` — копии порта FFmpeg. Если после `git pull` в vcpkg сборка ломается, синхронизируй **`ports/ffmpeg`** из того же коммита vcpkg в оба overlay.
