# 架构

不随代码同步改，一年翻两次。刚建好的项目写的是骨架本身（示例业务“备忘”），加了真的业务后照实改。

## 要解决的问题

同一套业务规则要在好几个地方跑：命令行、桌面和手机应用（核心直接跑在应用里），手机还要能连电脑上开着的服务，以后还会有别的端。难点是规则只写一份，各端不各自重写；加一个操作时，各端和生成的类型自动跟上。

## 模块关系

- `crates/bedrock-core`：业务规则和操作面。每块业务一个模块（示例是 `note`），所有操作登记在一张表里，`Core::call` 按名字调用，`manifest()` 给出操作清单和 JSON Schema。
- `crates/bedrock-store-sqlite`：存储端口的 SQLite 实现，表结构和迁移也在这里。
- `crates/bedrock-local`：在本机组装核心：SQLite 存储、系统时间、随机编号。要在本机直接跑核心的端都用它。
- `crates/bedrock-http`：把操作面开成 `POST /api/op/<名字>`，挡住浏览器跨站和 DNS 重绑定。
- `crates/bedrock-ffi`：给 Flutter 应用的接口出口，用 flutter_rust_bridge 只导出 `open` 和 `call`，里面用 `bedrock-local` 组装核心。
- `apps/cli`：命令行，在本机直接用核心；`bedrock serve` 用 `bedrock-http` 开服务，`bedrock ops --json` 是导出操作清单的入口。
- `apps/app`：桌面和手机共用的 Flutter 应用。本机经 `bedrock-ffi` 跑核心，也能经 HTTP 连别的设备上的 `bedrock serve`，两种传输对界面是同一个接口。构建时由 Dart 的原生构建钩子编译 `bedrock-ffi` 打进包里。
- `schema/ops.json`：操作清单，`just gen` 从核心导出，Dart 和其他语言的类型从它生成。

## 边界和不变量

- **不变量：** `bedrock-core` 不依赖数据库、HTTP、Web 框架、异步运行时、平台框架和 anyhow，也不直接读时间、环境变量、文件。依赖由核心目录的 `deny.toml` 查，函数由核心目录的 `clippy.toml` 查。
- **不变量：** 业务规则只在核心里。各端只做组装（读配置、建核心）和传输（把调用送进来、把结果送回去）。
- **不变量：** 对外只有操作面。各端经 `Core::call`（或 Rust 里的 `Core::run`）调用，不直接碰端口和领域类型。
- **不变量：** 只有 `bedrock-http` 知道 HTTP：状态码按错误的大类（`ErrorKind`）映射，核心不知道状态码。
- **不变量：** 不写 `unsafe`（工作区的 lint 设成 deny）。给 Flutter 的接口出口要用时，只在那一个 crate 放开。
- **不变量：** `schema/ops.json`、应用里生成的 Dart 类型和核心的类型永远一致，过期了 `just check` 失败。
- **不变量：** 应用的界面不直接碰传输，只经生成的操作表和 provider 调操作。

## 跨模块的约定

- 配置只在各端读：环境变量以 `BEDROCK_` 开头（`BEDROCK_DATA` 数据目录、`BEDROCK_LISTEN` 监听地址、`BEDROCK_LOG` 日志级别），读好后作为参数传给 `bedrock-local` 和核心。
- 意料之外的错误由核心记日志（`tracing`），对外只报 `internal`。各端负责装日志的输出。
- 内存版端口（核心的 `fake` 特性）给测试和演示模式用；每个真的存储都跑核心里的约定测试，保证和内存版行为一致。
