# bedrock-ffi

给 Flutter 应用的接口出口，用 flutter_rust_bridge 导出：`Bedrock.open(数据目录)` 和 `call(名字, 输入 JSON)`，失败时带回 `CallError { code, message }`。

## 不管什么

- 操作的类型：Dart 那边从 `schema/ops.json` 生成，和 HTTP 传输共用，这里只传 JSON 字符串。
- 数据目录在哪：由应用决定（系统给它的应用数据目录）。

## 不变量

- 整个工作区只有这里的 `frb_generated.rs` 放开 `unsafe`，那是 flutter_rust_bridge 生成的胶水代码。
- 自带一个两个线程的 tokio 运行时：存储要在 tokio 里跑，而 Dart 调进来的线程不归 tokio 管。
- `rust-toolchain.toml` 是给 Flutter 构建钩子看的：版本和仓库根目录的一样，`targets` 列出要编的平台。
- `src/frb_generated.rs` 和 `apps/app/lib/ffi/` 由 `just gen` 生成，改了 `src/api.rs` 要重新生成。
