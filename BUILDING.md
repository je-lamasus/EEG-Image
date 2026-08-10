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

Workflow сначала проверяет версию, форматирование, тесты и Clippy, затем создаёт пять архивов. Архивы загружаются как direct artifacts без дополнительной ZIP-обёртки GitHub Actions. После успешной сборки GitHub CLI создаёт тег `v<версия>`, публикует GitHub Release и прикрепляет все архивы. Для release job запрошено минимально необходимое разрешение `contents: write`.

Общая мастер-иконка находится в `packaging/icons/app-icon.png`. Формат `.icns` копируется в macOS app bundle, а `.ico` встраивается в Windows executable во время работы `build.rs`. Уменьшенная PNG-версия также задаётся как иконка окна Iced.

### Подпись и notarization macOS

macOS-версия упаковывается как `EEG Image.app`. Чтобы Gatekeeper разрешал запуск загруженного приложения без ручного обхода защиты, добавьте в `Settings → Secrets and variables → Actions` следующие repository secrets:

- `APPLE_CERTIFICATE_BASE64` — экспортированный сертификат `Developer ID Application` в формате PKCS#12 (`.p12`), закодированный в Base64;
- `APPLE_CERTIFICATE_PASSWORD` — пароль от `.p12`;
- `APPLE_ID` — Apple Account участника Apple Developer Program;
- `APPLE_TEAM_ID` — Team ID из Apple Developer Account;
- `APPLE_APP_SPECIFIC_PASSWORD` — app-specific password для `notarytool`.

Если заданы все пять секретов, workflow импортирует сертификат во временный keychain, включает Hardened Runtime при подписи, отправляет приложение в Apple Notary Service и прикрепляет полученный ticket к `.app`. Временный сертификат и keychain удаляются после сборки. Если секреты отсутствуют полностью, создаётся неподписанная сборка с предупреждением; если заполнена только часть секретов, macOS job завершается ошибкой, чтобы случайно не опубликовать некорректно настроенный релиз.

Каждый push в `main`, кроме первого push в пустой репозиторий, должен повышать `[package].version` в `Cargo.toml`; после изменения нужно обновить `Cargo.lock` командой `cargo check`. Если политика организации запрещает запись через `GITHUB_TOKEN`, разрешите `Read and write permissions` в `Settings → Actions → General → Workflow permissions`.
