# gifcap

## О проекте

Запись прямоугольной области экрана под панелью окна в GIF или анимированный WebP (как GifCam): кадры идут в FFmpeg и пишутся на диск по мере захвата, без накопления всей сессии в RAM.

Только Windows: захват и UI на Win32.

## Зависимости

Порядок установки:

1. Visual Studio или Build Tools: рабочая нагрузка «Разработка классических приложений на C++», MSVC x64. Хост Rust: `x86_64-pc-windows-msvc` (проверка: `rustc -vV`).
2. LLVM (libclang). Сборка `ffmpeg-sys-next` вызывает bindgen: нужен `libclang.dll` в каталоге для `LIBCLANG_PATH`.
   - Через Visual Studio Installer → Изменить → отдельные компоненты → **C++ Clang Compiler for Windows** (в составе идёт LLVM). Типичный путь:  
     `C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin`  
     (у VS 2026 часто папка `18`; издание — Community / Professional / Enterprise). Поиск:  
     `where /R "C:\Program Files\Microsoft Visual Studio" libclang.dll`
   - Отдельно: установщик с [релизов LLVM](https://github.com/llvm/llvm-project/releases) (для Windows — артефакт вида `LLVM-*-win64.exe`). После установки `libclang.dll` обычно в `C:\Program Files\LLVM\bin`.
3. Rust: [rustup](https://rustup.rs/), `rustup default stable-x86_64-pc-windows-msvc`.
4. vcpkg — менеджер библиотек C/C++ от Microsoft: репозиторий [github.com/microsoft/vcpkg](https://github.com/microsoft/vcpkg). Клонируется весь репозиторий (исходники и скрипты инструмента, не отдельный установщик):

   ```bat
   git clone https://github.com/microsoft/vcpkg.git C:\path\to\vcpkg
   cd /d C:\path\to\vcpkg
   bootstrap-vcpkg.bat
   vcpkg.exe install ffmpeg[webp]:x64-windows-static-md
   ```

   Фича **`[webp]`** нужна для **анимированного WebP** (в FFmpeg подключается libwebp). Без неё GIF работает, WebP — нет. Если FFmpeg уже ставился без `webp`, переустанови с этой фичей (при необходимости `vcpkg remove ffmpeg:x64-windows-static-md`, затем команда выше).

   Каталог клона — корень установки; в `VCPKG_ROOT` задаётся этот путь (внутри будут `vcpkg.exe`, `ports`, `installed`, `downloads` и т.д.).

   Triplet `x64-windows-static-md` и таргет `x86_64-pc-windows-msvc` заданы в `.cargo/config.toml`.

   **Манифест в репозитории (`vcpkg.json`):** из **корня gifcap** выполни  
   `%VCPKG_ROOT%\vcpkg.exe install --triplet x64-windows-static-md`  
   — FFmpeg попадёт в `vcpkg_installed/`. В `.cargo/config.toml` задан **`FFMPEG_DIR`** на этот префикс (manifest-дерево не совпадает с тем, что ждёт `vcpkg-rs`). Собирай из корня репо; **не задавай** для Cargo `VCPKG_ROOT=.../gifcap/vcpkg_installed` (будет ошибка про `.vcpkg-root` / `pkg-config`). Если конфиг не подхватился: `cargo clean` и явно  
   `set FFMPEG_DIR=C:\path\to\gifcap\vcpkg_installed\x64-windows-static-md`.

Перед сборкой в сессии терминала (пути заменить на свои):

```bat
set VCPKG_ROOT=C:\path\to\vcpkg
set LIBCLANG_PATH=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
```

Сборка: `ffmpeg-sys-next` + bindgen (заголовки FFmpeg через libclang). Линковка FFmpeg статическая; отдельные `avutil-*.dll` к exe не нужны.

## Сборка

```bat
cd C:\path\to\gifcap
cargo build --release -p gifcap
```

Артефакт: `target\x86_64-pc-windows-msvc\release\gifcap.exe`.

После смены `VCPKG_ROOT` / `LIBCLANG_PATH` или triplet: `cargo clean`, затем снова `cargo build`.

## Использование

1. Запустить `gifcap.exe`.
2. Подогнать размер окна: под верхней панелью видна область захвата (рабочий стол); панель при записи остаётся на месте.
3. FPS, формат GIF / WebP / MP4, Record; Pause / Resume; Stop & save.
4. Итоговый файл: `%USERPROFILE%\Pictures\gifcap\`. Во время записи поток — в `%USERPROFILE%\.gifcap\active\` (`recording.gif`, `recording.webp` или `recording.mp4`).

Лог: `%USERPROFILE%\.gifcap\logs\gifcap.log` (ротация по размеру, до четырёх файлов `gifcap.log.1` … `gifcap.log.4`).
