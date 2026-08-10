# Сборка eeg-image

Проект использует Rust `1.94.1`. Быстрая оптимизированная сборка создаётся командами-алиасами из `.cargo/config.toml`:

| Платформа | Архитектура | Rust target | Команда |
|---|---|---|---|
| Linux | x86-64 | `x86_64-unknown-linux-gnu` | `cargo release-linux-x86-64` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `cargo release-linux-arm64` |
| Windows | x86-64 | `x86_64-pc-windows-msvc` | `cargo release-windows-x86-64` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `cargo release-windows-arm64` |
| macOS | ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `cargo release-macos-arm64` |

Перед первой сборкой target нужно установить через `rustup target add <target>`. Нативная сборка предпочтительна. Linux ARM64 собирается на ARM64 runner, обе Windows-версии — на Windows x86-64 runner (ARM64 кросс-компиляцией), macOS — на Apple Silicon runner.

Для Windows runner нужны Visual Studio Build Tools с компонентами C++ для x86-64 и ARM64. Для Linux runner нужны системные библиотеки X11/Wayland, перечисленные в `.github/workflows/release.yml`.

## GitHub Actions

Workflow `.github/workflows/release.yml` запускается при каждом push в `main`. Он использует GitHub-hosted runner-ы:

- `ubuntu-22.04` для Linux x86-64;
- `ubuntu-22.04-arm` для Linux ARM64;
- `windows-2025` для Windows x86-64 и кросс-компиляции Windows ARM64;
- `macos-15` для macOS ARM64.

Workflow сначала проверяет версию, форматирование, тесты и Clippy, затем создаёт пять архивов. После успешной сборки GitHub CLI создаёт тег `v<версия>`, публикует GitHub Release и прикрепляет все архивы. Для release job запрошено минимально необходимое разрешение `contents: write`.

Каждый push в `main`, кроме первого push в пустой репозиторий, должен повышать `[package].version` в `Cargo.toml`; после изменения нужно обновить `Cargo.lock` командой `cargo check`. Если политика организации запрещает запись через `GITHUB_TOKEN`, разрешите `Read and write permissions` в `Settings → Actions → General → Workflow permissions`.
