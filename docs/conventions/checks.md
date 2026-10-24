# 检查

本机和 CI 跑同样的检查，入口都是 `just`。提交前钩子只查这次暂存的文件，`just check` 查全部。

## just 命令

| 命令 | 做什么 |
| --- | --- |
| `just` | 列出所有命令 |
| `just deps` | 按固定版本装 cargo 工具（含 flutter_rust_bridge_codegen）和 Flutter，拉应用的依赖，装上提交钩子 |
| `just deps-app` | 只装 Flutter、拉应用的依赖（CI 用） |
| `just gen` | 导出操作清单，生成 Dart 类型和给 Dart 的接口 |
| `just fmt` | 格式化全部 Rust 和 Dart 代码 |
| `just check` | 只检查、不改文件：格式、typos、clippy、cargo-deny（全仓库一次，核心单独一次）、cargo-machete、Dart 格式、`flutter analyze`、生成的文件是否过期 |
| `just test` | Rust 的全部测试（含命令行和 HTTP 的端到端测试），加上应用里不起界面的测试 |
| `just e2e` | 在 Linux 桌面上起应用跑集成测试，本机跑核心和连 `<项目> serve` 各一遍，会弹出窗口 |
| `just ci` | `check`、`test`、`e2e`，和 CI 完全一样 |
| `just e2e-device <设备>` | 在连着的手机上跑集成测试，只跑本机跑核心那条 |
| `just dev [参数]` | 开发模式跑命令行：`just dev note list`、`just dev serve` |
| `just app [参数]` | 开发模式在本机桌面跑应用，参数交给 `flutter run` |
| `just build` | 构建命令行的发行版 |
| `just build-app <目标>` | 构建应用的发行版：`linux`、`apk` 等，交给 `flutter build` |

工具的版本写在 `justfile` 开头，升级就改版本号再跑 `just deps`。

## Rust

### 工具链和工作区

- `rust-toolchain.toml` 固定一个稳定版本号，带 `rustfmt`、`clippy`。升级工具链单独提交。
- 版次 2024。根 `Cargo.toml` 统一管：
  - `[workspace.package]`：版次、最低 Rust 版本、许可证、仓库地址，成员写 `edition.workspace = true`。
  - `[workspace.dependencies]`：所有外部依赖的版本只写在这里，成员写 `serde = { workspace = true }`。
  - `[workspace.lints]`：统一的 lint，成员写 `[lints] workspace = true`。

### lint

- clippy 默认规则零警告（`just check` 带 `-D warnings`），检查时打开所有特性（`--all-features`），测试和假端口也查。
- 另外打开：
  - `dbg_macro`、`todo`、`unimplemented`：调试和占位代码不进仓库。
  - `unwrap_used`、`expect_used`：正式代码里处理好失败，测试里允许（`clippy.toml` 的 `allow-unwrap-in-tests`、`allow-expect-in-tests`）。
  - **坑**：这两项只管 `#[test]` 函数和 `#[cfg(test)]` 模块里的代码，`tests/` 下集成测试文件里的辅助函数管不到。每个集成测试文件开头写 `#![allow(clippy::unwrap_used, clippy::expect_used)]`。
  - `unsafe_code = "deny"`：只有 ffi crate 放开。
- 核心另有自己的 `clippy.toml`，用 `disallowed-methods` 禁掉直接读时间、环境变量、文件的函数；核心的 `lib.rs` 开 `print_stdout`、`print_stderr`。clippy 只读离 crate 最近的那份 `clippy.toml`，所以核心那份要把根目录的设置也抄上。
- 格式用 rustfmt 默认规则，`rustfmt.toml` 只写 `style_edition` 和 `edition`（提交钩子直接调 rustfmt 时要用）。

### 依赖审查

- 根目录 `deny.toml`：许可证白名单（只列现有依赖用到的，新依赖带来新许可证时确认后再加）、已知漏洞和不再维护的库、只认 crates.io、同一个库多个版本时警告。放过某条通报要写进 `ignore` 并注明原因。
- `crates/<项目>-core/deny.toml`：核心的禁用依赖（见[核心分层](layering.md)），`just check` 用 `cargo deny --manifest-path crates/<项目>-core/Cargo.toml --config crates/<项目>-core/deny.toml check bans` 单独跑（`--config` 要写在 `check` 前面）。它只看核心的正式依赖，测试用的（比如异步运行时）不算，靠的是那份配置里的 `exclude-dev = true`。
- cargo-machete 查声明了没用的依赖。
- 加新依赖前看它近一年有没有发版、许可证和项目合不合，能用现成的库就不自己写。

## Flutter

- 版本固定在仓库根目录的 `.fvmrc`，用 fvm 装，命令一律写成 `fvm flutter …`、`fvm dart …`，不用系统里装的 `flutter`、`dart`，免得各人版本不一样。
- fvm 默认把 SDK 放在家目录下的 `fvm/`，装好 fvm 后先 `fvm config --cache-path ~/.local/share/fvm` 换到别处。
- 仓库里的 `.fvm/` 是 fvm 生成的链接，不入库。**坑**：`.gitignore` 里要写成 `.fvm/`（不带开头的斜杠），fvm 认得这种写法；写成别的，每次 `fvm use` 都会往文件末尾再追加一段，还不换行。
- fvm 用 `dart pub global activate fvm` 装，装在 `~/.pub-cache/bin`，要把这个目录加进 `PATH`。
- `dart format` 默认格式，`fvm flutter analyze` 零警告，lint 规则见[界面](ui.md)的代码风格。
- 应用依赖的版本范围写在 `pubspec.yaml`，`pubspec.lock` 入库。还在 0.x 的界面库写死版本。

## 拼写

typos 查代码、文档、提交里的英文拼写。真有这个词（缩写、专有名词）就加进 `_typos.toml`。Xcode 的工程文件（`*.pbxproj`、`*.xib`、`*.storyboard`）里全是随机编号，整类排除。

## 提交钩子

lefthook，配置在 `lefthook.yml`，`just deps` 装上。lefthook 和 gitleaks 用系统的包管理器装。只检查暂存的文件，不改文件：

- 提交前（并行）：
  - Rust 格式：`rustfmt --check`。它不经过 cargo，读不到 `Cargo.toml` 里的版次，所以 `rustfmt.toml` 里要写 `edition`。
  - Dart 格式：在 `apps/app` 里跑 `dart format`，按应用的语言版本排版。
  - 拼写：typos 加 `--force-exclude`，不然直接传进去的文件会绕过 `_typos.toml` 的排除。
  - 密钥：`gitleaks git --pre-commit --staged`，扫暂存的改动。
- 写提交说明时：`just check-commit-msg` 查格式（见下）。合并、`fixup!` 这类 git 自己生成的说明放过。
- 本机想加自己的检查（比如一份私有关键词清单），写在 `lefthook-local.yml`，它不入库。
- 改了钩子，在一个临时副本里真提交一次试：`git init` 后 `lefthook install`，提交时用 `git -c commit.gpgsign=false`，免得弹签名授权。

## 提交说明

- 首行用英文写成 `类型: 说明`，不超过 72 个字符。类型是 `feat`、`fix`、`docs`、`refactor`、`test`、`build`、`ci`、`perf`、`chore`、`revert` 之一，不带范围（不写成 `feat(app): …`）。例：`feat: list notes by update time`。
- 需要时空一行写正文，说明为什么改。
- 不带 AI 工具的署名行。
- 一个提交只做一件事；规矩、骨架、依赖升级各自单独提交。

## CI

GitHub Actions，工作流在 `.github/workflows/`：

- `ci.yml`，每次推送：在 Linux 上跑 `just ci`，集成测试在 xvfb 的虚拟屏幕上跑。
- `release.yml`，打 `v` 开头的标签时：在 Windows、macOS、Linux 上构建应用和命令行，在 Linux 上构建安卓包，发到 GitHub Releases。手动运行时只构建不发版。步骤见项目的 `docs/release.md`。
- CI 用的命令就是本机的 `just` 命令，工作流里只装环境。工具版本只写一处：检查工具和 fvm 的版本从 `justfile` 读（`just ci-tools`、`just --evaluate fvm`），Rust 按 `rust-toolchain.toml`，Flutter 按 `.fvmrc`；只有 just 自己的版本写在工作流里。
- 第三方 action 钉到提交哈希，后面注释写版本号（`@<哈希> # v7.0.1`），升级时一起改。
- 缓存：Rust 用 rust-cache，连同 Flutter 构建钩子在 `apps/app/.dart_tool/hooks_runner/shared` 里编的 Rust 一起缓存；Flutter SDK 和 pub 的包按 `.fvmrc`、`pubspec.lock` 缓存。
