# bedrock-core

业务规则和操作面。每块业务一个模块（现在只有 `note`），模块里分领域类型、端口、操作、错误；所有操作在 `lib.rs` 的 `operations!` 里登记，各端经 `Core::call` 按名字调用。

## 不管什么

- 存储、网络、时间、编号的实现：写成端口，由适配器 crate 实现。
- HTTP 状态码、命令行输出、界面文字以外的展示：在各端。
- 读配置：各端读好，作为参数传进来。

## 不变量

- 不依赖数据库、HTTP、Web 框架、异步运行时、flutter_rust_bridge、anyhow。名单在本目录的 `deny.toml`，`just check` 单独查。
- 不直接读时间、环境变量、文件，不起线程、不睡眠。本目录的 `clippy.toml` 用 `disallowed-methods` 拦着；它也要抄上根目录 `clippy.toml` 的设置。
- 操作的输入用普通类型（`String`），在操作里构造领域类型，校验失败报这个操作自己的错误码。领域类型不派生 `Deserialize`，没有绕过校验的口子。
- 错误枚举用 `op_error!` 实现 `OpError`，每种情况只写一次 `code`。
- `fake` 特性打开内存版端口（`fake`、`note::MemoryNotes`）和约定测试（`note::contract`）。真的存储实现要跑同一组约定测试。
