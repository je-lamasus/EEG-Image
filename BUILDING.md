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

Для Windows runner нужны Visual Studio Build Tools с компонентами C++ для x86-64 и ARM64. Для Linux runner нужны системные библиотеки X11/Wayland, перечисленные в `.gitlab-ci.yml`.

## GitLab Runner

Pipeline запускается только при push в `main` и ожидает runner-ы с тегами:

- `linux-x86-64` — Docker executor на x86-64;
- `linux-arm64` — Docker executor на ARM64;
- `windows-x86-64` — Shell executor с PowerShell, Rust и Visual Studio Build Tools;
- `macos-arm64` — Shell executor на Apple Silicon с Rust и Xcode Command Line Tools.

Имена тегов можно переопределить переменными GitLab CI/CD `RUNNER_TAG_LINUX_X86_64`, `RUNNER_TAG_LINUX_ARM64`, `RUNNER_TAG_WINDOWS_X86_64` и `RUNNER_TAG_MACOS_ARM64`.

Каждый push в `main` должен повышать `[package].version` в `Cargo.toml`; после изменения нужно обновить `Cargo.lock` командой `cargo check`. Успешный pipeline публикует GitLab Release с тегом `v<версия>` и пятью архивами.
