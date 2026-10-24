//! 给 Flutter 应用的接口出口：打开数据目录得到核心，再按名字调操作，输入输出都是 JSON 字符串。
//!
//! 操作的类型不经这里生成：Dart 那边的类型从 `schema/ops.json` 来，和 HTTP 传输共用一套。

pub mod api;

// flutter_rust_bridge 生成的胶水代码，要用 unsafe，也不按本仓库的 lint 写
#[allow(unsafe_code, clippy::all, clippy::unwrap_used, clippy::expect_used)]
mod frb_generated;
