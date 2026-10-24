# 开发

## 环境

- Rust：装好 rustup 就行，进仓库目录时按 `rust-toolchain.toml` 自动装对应版本。构建应用时还会按 `crates/bedrock-ffi/rust-toolchain.toml` 装安卓的编译目标。
- just：任务入口，用系统的包管理器装。
- fvm：`dart pub global activate fvm`，再把 `~/.pub-cache/bin` 加进 `PATH`。装好后先 `fvm config --cache-path ~/.local/share/fvm`，免得 SDK 装进家目录下的 `fvm/`。
- lefthook、gitleaks：提交钩子用，用系统的包管理器装。
- 第一次克隆后运行一次 `just deps`：按 `justfile` 开头写的版本装 cargo 工具（cargo-deny、typos、cargo-machete、flutter_rust_bridge_codegen），按 `.fvmrc` 装 Flutter，拉应用的依赖，装上提交钩子。
- Linux 桌面：Flutter 要的 GTK 开发包、clang、cmake、ninja，`fvm flutter doctor` 会列出缺什么。
- 安卓：安卓 SDK（含 NDK）和 JDK 21，`JAVA_HOME` 指到它。要完整的 JDK：只装了运行时（没有 `javac`）时，Gradle 报“does not provide the required capabilities: [JAVA_COMPILER]”。

## 常用命令

```sh
just                       # 列出所有命令
just dev note list         # 跑命令行，后面是命令行的参数
just dev serve             # 开服务，默认 http://127.0.0.1:7400
just app                   # 在本机桌面跑应用，核心跑在应用里
just check                 # 只检查不改文件：格式、拼写、clippy、依赖、Dart 的 lint、生成的文件是否过期
just test                  # 不起界面的测试：Rust 全部，加上应用的单元测试
just e2e                   # 在桌面上起应用跑集成测试，会弹出窗口
just gen                   # 改了操作或它的输入输出后，重新导出清单、生成 Dart 类型
just ci                    # 提交前跑一遍，和 CI 一样
just build-app apk         # 构建安卓包，在 apps/app/build/app/outputs/flutter-apk/
```

## 调试

- 命令行的数据目录默认是系统的应用数据目录下的 `bedrock`（Linux 上是 `~/.local/share/bedrock`），用 `BEDROCK_DATA=<目录>` 换一个，试东西时指到临时目录，不弄乱自己的数据。
- 应用的数据放在系统给它的数据目录（Linux 上是 `~/.local/share/app.bedrock`），和命令行的分开。
- 让桌面应用连命令行开的服务：先 `just dev serve`，再 `just app --dart-define=BEDROCK_SERVER=http://127.0.0.1:7400`。
- 日志级别用 `BEDROCK_LOG`，写法同 `tracing` 的过滤规则：`BEDROCK_LOG=debug`、`BEDROCK_LOG=bedrock_core=debug,info`。都打到标准错误，`serve` 默认 `info`，其余命令默认 `warn`。
- 直接调一个操作看原始结果：`just dev op note.create '{"title":"试试"}'`；看有哪些操作：`just dev ops`。
- 只跑某一部分测试：`cargo test -p bedrock-core`、`cargo test -p bedrock-http`、`cargo test -p bedrock-cli creates`；应用里的：`cd apps/app && fvm flutter test test/format_test.dart`。
- 改了 `crates/bedrock-ffi` 导出的函数，要 `just gen` 重新生成两头的胶水代码，不然应用启动时报内容哈希不一致。

## 在手机上测

- 手机和电脑连同一个局域网，手机打开开发者选项里的“无线调试”。第一次要配对：在“无线调试”里点“使用配对码配对设备”，电脑上运行 `adb pair <手机地址>:<配对端口>`，输入手机上显示的配对码。以后同一个网络里会自动连上，`adb devices` 能看到。
- 跑集成测试：`just e2e-device <设备名>`，设备名看 `fvm flutter devices`。它会编一份调试包装上去自动点一遍；连 `bedrock serve` 那条要在电脑上起进程，手机上会跳过。测试时别碰手机。
- 装正式包自己试：`just build-app apk`，再 `adb install -r apps/app/build/app/outputs/flutter-apk/app-release.apk`。
- 打包时 Gradle 会留下一个守护进程，占一两个 G 内存，下一次构建接着用它。内存紧时用完就停：`cd apps/app/android && ./gradlew --stop`。
