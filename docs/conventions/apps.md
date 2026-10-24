# 各端

每个端只做组装和界面：读配置、建适配器和核心、把操作面接到自己的界面或协议上。业务规则一律在核心里。

## 命令行 `apps/cli`

- 用 clap 的 derive 写法，程序名就是项目名。
- 常用操作有好记的子命令，按业务分组：`<项目> note list`、`<项目> note create "标题"`。
- 任何操作都能直接调：`<项目> op <名字> '<输入 JSON>'`，输入写 `-` 时从标准输入读，不写就是 `{}`，输出固定是 JSON。`<项目> ops` 列出所有操作，加 `--json` 输出完整的操作清单（`just gen` 就用它导出）。
- 子命令默认输出给人看的中文；加 `--json` 输出操作的原始结果，方便脚本处理，出错时标准错误里也是 JSON 的 `{ code, message }`。
- 退出码：成功 0，操作失败 1，参数写错 2。错误说明打到标准错误。
- 在本机直接用核心（`<项目>_local::open`），数据目录用 `--data` 或 `<项目大写>_DATA` 指定。
- 常年不变的选项（数据目录、监听地址）也能用环境变量设，名字是 `<项目大写>_` 加选项名（`--listen` 对应 `_LISTEN`）；日志级别用 `_LOG`。
- 平时用单线程的异步运行时，做完一件事就退出；只有 `serve` 用多线程。

## 开服务 `<项目> serve`

让别的设备（比如手机上的应用）经 HTTP 调操作。路由在 `crates/<项目>-http`，命令行只负责监听、组装和退出。

- 用 axum。操作面走 `POST /api/op/<名字>`，另有 `GET /api/health`；别的路径一律返回 JSON 的 `not_found`。
- 只收 `Content-Type: application/json` 的请求：浏览器跨站发 JSON 要先预检，服务不放行跨站，别的网站就没法借用户的浏览器发请求。
- 只监听本机地址时，只认本机的主机名（`localhost`、`127.0.0.1`、`[::1]`），挡住 DNS 重绑定：别的网站把自己的域名解析到 127.0.0.1，再从用户的浏览器里调进来。
- 回应都是 JSON，每个回应都带安全头：内容安全策略 `default-src 'none'`、`nosniff`、`no-referrer`、禁止被嵌入。
- 默认只监听本机（`--listen`，默认 `127.0.0.1:7400`）。还没有认证，监听别的地址时打一条警告；要给别的设备用之前先加认证。
- 准备好后往标准输出打一行 `在 http://<地址> 上提供服务`，端口写 0 时这里是系统分配的端口，测试和脚本读这一行。
- 收到退出信号（Ctrl+C、SIGTERM）时停止接新请求，等进行中的请求做完再退出，退出码 0。
- 路由写成 `router(core, 监听地址)`、启动写成 `serve(listener, core, 退出信号)`，放在 `<项目>-http` 的 `lib.rs`，它的测试直接调这两个函数。

## Flutter 应用 `apps/app`

桌面（Windows、macOS、Linux）、安卓、iOS 是同一个 Flutter 应用，界面的写法见[界面](ui.md)。

- 核心直接跑在应用进程里，经 `crates/<项目>-ffi` 调用，数据放在系统给这个应用的数据目录。
- 也能连别的设备上的 `<项目> serve`，走 HTTP。两种传输对界面是同一个接口。
- 官方和社区插件没有的平台能力（后台常驻、推送等），用 Kotlin 或 Swift 写成 Flutter 插件。插件只做平台相关的事，业务规则仍在核心里。
- 启动时选传输：编译时给了 `--dart-define=<项目大写>_SERVER=http://地址:端口` 就连那台电脑，没给就在本机跑核心。
- 安卓的 applicationId 和各平台的包名是 `app.<项目>`，不带个人信息。
- 本机的数据放在系统给这个应用的目录，和命令行的分开。**坑**：Linux 上 path_provider 在 `$XDG_DATA_HOME/app.<项目>` 还不存在、而 `$XDG_DATA_HOME/<项目>`（命令行的数据目录）存在时，会退回用后者，两边就混在一起了。所以 Linux 上直接用 `$XDG_DATA_HOME/app.<项目>`，见 `main.dart` 的 `_dataDir`。

```text
apps/app/
├── lib/
│   ├── main.dart          选传输、注入、启动
│   ├── app.dart           主题、语言、路由
│   ├── errors.dart        错误怎么显示
│   ├── transport/         Transport 接口，本机和 HTTP 两种实现
│   ├── gen/ops.dart       生成的类型、操作表、provider（just gen）
│   ├── ffi/               flutter_rust_bridge 生成的 open、call（just gen）
│   └── <业务>/            每个业务一个目录：页面和它的纯逻辑
├── hook/build.dart        构建应用时编译 <项目>-ffi，打进包里
├── tool/gen_ops.dart      从 schema/ops.json 生成 lib/gen/ops.dart
├── test/                  不起界面的单元测试
├── integration_test/      在真窗口里跑整个应用
└── android/ ios/ linux/ macos/ windows/   各平台的工程，flutter create 生成
```

### Rust 怎么进到应用里

- 用 Dart 的原生构建钩子：`hook/build.dart` 交给 flutter_rust_bridge_hooks，`flutter run`、`flutter build`、`flutter test` 时自动用 rustup 编译 `crates/<项目>-ffi`，按目标平台交叉编译，产物打进应用。编译 Rust 只靠这个钩子，各平台的工程里没有相关的脚本。
- 编译用的 Rust 版本和目标平台写在 `crates/<项目>-ffi/rust-toolchain.toml`：`channel` 和仓库根目录那份一样（升级时两份一起改），`targets` 列出要打包的平台（Linux、macOS 两种 CPU、Windows 桌面，安卓的三种 CPU；iOS 要构建时再加），rustup 缺什么自动装。
- 安卓交叉编译要安卓 SDK 里的 NDK，路径由 Flutter 传给钩子。
- 钩子按发行版编 Rust，打调试包时也一样；每种目标平台、每种构建配置第一次构建时要把全部依赖编一遍，之后只编改了的。
- **坑**：钩子只把少数几个环境变量（`PATH`、`HOME`、`TMPDIR` 这类）传给 cargo，`CARGO_BUILD_JOBS`、`RUSTFLAGS` 都传不进去。本机要限编译的并行数，写在 `~/.cargo/config.toml` 的 `[build] jobs`。
- **坑**：cargo 的依赖文件里写到的目录（`build.rs` 里 `rerun-if-changed` 的目录）要带结尾的斜杠，不然钩子把它当成不存在的文件，每次构建都判定过期，重跑一遍还报“File modified during build”。

### 安卓

- 构建用 JDK 21（要完整的 JDK，只有运行时不行）。Gradle 的内存在 `android/gradle.properties` 里限住：`org.gradle.jvmargs=-Xmx2g …`、`org.gradle.workers.max=4`。
- 正式包现在只能在本机跑核心：`src/main/AndroidManifest.xml` 里没有网络权限（`flutter create` 只给调试包加了），安卓也默认不让走明文 HTTP。要连别的电脑上的 `<项目> serve`，等服务加了认证和加密传输，再加网络权限。

## 给 Flutter 的接口出口 `crates/<项目>-ffi`

- 用 flutter_rust_bridge，只导出两个函数：`open(数据目录)` 返回一个句柄，`句柄.call(name, input_json) -> output_json`，失败时带回 `{ code, message }`。句柄在 Dart 一侧是不透明的对象，里面是组装好的核心。
- `crate-type` 是 `["staticlib", "cdylib"]`：iOS 和 macOS 要静态库，其他平台要动态库。
- 自己带一个两线程的 tokio 运行时跑核心，和 flutter_rust_bridge 的线程池分开。
- 只导出“按名字调用”：操作的输入输出类型从 `schema/ops.json` 生成（见[类型生成](codegen.md)），和 HTTP 传输共用同一份，不让 flutter_rust_bridge 另生成一套。
- 整个工作区只有这个 crate 允许 `unsafe`（生成的胶水代码要用）。

## 加一个新端

1. 在 `apps/` 下建目录，名字是一个英文小写词。
2. 只写组装和界面：建适配器和核心，或连 `<项目> serve`，把操作面接上。
3. 需要类型时从 `schema/ops.json` 生成，命令加进 `just gen`。
4. 在 `justfile` 里加运行和构建的命令，在 CI 里加构建。
5. 至少写一条端到端测试。
6. 更新 README 的安装和使用、`docs/ARCHITECTURE.md` 的模块关系。
