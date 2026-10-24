# 加端、加平台、去掉应用

先读 `docs/conventions/apps.md`，它的“加一个新端”一节是总步骤，这里补上具体要改的文件。

## 加一个新端

新端是 `apps/` 下的一个新目录，名字是一个英文小写词。

1. 只写组装和界面：在本机跑核心就用 `<项目>_local::open`（Rust 写的端），连别的设备就走 `<项目> serve` 的 HTTP。
2. 需要别的语言的类型时，从 `schema/ops.json` 生成（规矩见 `docs/conventions/codegen.md`），生成命令加进 `justfile` 的 `gen`，过期检查加进 `check-gen`。
3. `justfile`：运行、构建、测试的命令；`check`、`test` 或 `e2e` 里带上它，`just ci` 才会跑。
4. CI：`.github/workflows/ci.yml` 装它要的环境；要发版的话 `.github/workflows/release.yml` 的构建矩阵和 `justfile` 的 `package` 加上它。
5. 提交钩子 `lefthook.yml` 加它的格式检查；新语言的工具版本写进 `justfile` 开头。
6. 至少一条端到端测试。
7. 文档：`README.md` 的安装和上手，`docs/ARCHITECTURE.md` 的模块关系，`docs/development.md` 的环境和命令，`AGENTS.md` 的命令表，`CHANGELOG.md`。

## 给应用加一个平台

Flutter 应用已经带着 Windows、macOS、Linux、安卓、iOS 的工程。要开始构建某个平台（比如 iOS）：

1. `crates/<项目>-ffi/rust-toolchain.toml` 的 `targets` 加上这个平台的 Rust 目标，没列的平台构建时直接报错。
2. `justfile` 的 `build-app`、`package` 认这个目标，`.github/workflows/release.yml` 的构建矩阵加一行，`docs/release.md` 的产物表加一行。
3. 在那个平台上真跑一遍集成测试（手机用 `just e2e-device <设备>`）。

## 去掉应用，只留命令行

命令行不能去：`just gen` 靠它导出操作清单，`serve` 也在它里面。去掉应用时这些一起改：

- 删目录和文件：`apps/app`、`crates/<项目>-ffi`、`.fvmrc`；`.gitignore` 里的 `.fvm/`；根 `Cargo.toml` 工作区依赖里的 `flutter_rust_bridge`。
- `justfile`：删变量 `frb`、`fvm`、`app`、`dart_dirs`，命令 `deps-app`、`e2e`、`e2e-device`、`app`、`build-app`；`deps` 里的 `flutter_rust_bridge_codegen`；`gen`、`fmt`、`check`、`check-gen`、`test` 里 Dart 和 Flutter 的那几行；`ci` 只剩 `check test`；`package` 只打命令行；`ci-tools` 不变。
- `lefthook.yml`：删“Dart 格式”。`_typos.toml`：删 Xcode 文件的排除。
- `.github/workflows/ci.yml`：删系统依赖里 Flutter 和 xvfb 要的包、fvm 的缓存和安装，`xvfb-run -a just ci` 改成 `just ci`，`rust-cache` 的 `cache-directories` 删掉。
- `.github/workflows/release.yml`：构建矩阵只留三个系统的命令行，删 apk、fvm、安卓签名和“构建应用”这几步。
- 文档：`README.md`、`AGENTS.md`、`docs/ARCHITECTURE.md`、`docs/development.md`、`docs/release.md`、`CHANGELOG.md` 里应用的部分。

删完 `git grep -niE 'fvm|flutter|dart|apps/app|-ffi'` 应该什么都找不到，`just ci` 通过。
