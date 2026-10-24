# bedrock 应用

桌面（Windows、macOS、Linux）和手机（安卓、iOS）共用的 Flutter 应用。界面用 shadcn_ui，状态用 Riverpod，路由用 go_router。

## 管什么

- 界面和组装：启动时选传输、建 `Ops`，交给 Riverpod；页面经生成的 provider 和 `OpActions` 调操作。
- 两种传输（`lib/transport/`）：本机经 `crates/bedrock-ffi` 直接跑核心；编译时给 `--dart-define=BEDROCK_SERVER=http://地址:端口` 就连那台电脑上的 `bedrock serve`。

## 不管什么

- 业务规则：在核心里，这里只按 `code` 显示错误。
- 操作的类型：`lib/gen/ops.dart` 由 `just gen` 从 `schema/ops.json` 生成，`lib/ffi/` 由 flutter_rust_bridge 生成，都不手改。

## 不变量

- 界面不直接碰传输：查询用生成的 `<操作>Query`，动作用 `opActionsProvider`，动作成功后同一业务的查询自动重新取。
- 构建时 `hook/build.dart` 按 `crates/bedrock-ffi/rust-toolchain.toml` 编 Rust，那份文件的版本要和仓库根目录的一样，要编的平台都列在它的 `targets` 里。
