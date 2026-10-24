# 类型生成

Flutter 应用（以及以后别的语言写的端）用到的类型，全部从 Rust 生成，只有一份来源：核心导出的操作清单加 JSON Schema。手写的类型会和 Rust 慢慢对不上，生成的不会。

## 流程

```text
crates/<项目>-core     操作的输入、输出、错误派生 JsonSchema
        │  just gen
        ▼
schema/ops.json        操作清单 + 全部类型的 JSON Schema
        │
        ├─▶ apps/app/lib/gen/   Dart 类型和操作表
        ├─▶ 其他语言             要用时再加，从同一份生成
        └─▶ MCP 工具定义        直接读这份清单
```

- 输入输出派生 schemars 的 `JsonSchema`。字段的文档注释会进 Schema 的 `description`，再进生成的类型的注释，字段的规则（“1 到 200 个字”）写在那里。
- 可选字段的 `null` 在导出时去掉（见[操作面](operations.md)的线上格式）。
- `just gen` 做三件事：用命令行导出 `schema/ops.json`（`<项目> ops --json`）；从清单生成 `apps/app/lib/gen/ops.dart`；让 flutter_rust_bridge 重新生成 `open`、`call` 两头的胶水代码。

## 清单的样子

```json
{
  "version": "0.1.0",
  "operations": [
    {
      "name": "note.create",
      "kind": "action",
      "input": { "$ref": "#/$defs/CreateNoteInput" },
      "output": { "$ref": "#/$defs/NoteView" },
      "errors": ["empty_title", "title_too_long", "body_too_long", "internal"]
    }
  ],
  "$defs": { "…": "…" }
}
```

## Dart 这一侧

生成器是仓库里自己写的 `apps/app/tool/gen_ops.dart`，用 code_builder 拼代码，写完由 `just gen` 接着跑 `dart format`。它生成：

- 每个输入输出一个 `final class`：`const` 构造函数、`fromJson`、`toJson`、按值比较的 `==` 和 `hashCode`（查询按输入缓存，输入要按值比较）。
  - 可选字段是可空类型，没值时 `toJson` 不写这个键。
  - 时间是 `DateTime`，写出时转成 UTC。
  - Schema 的 `description` 变成类和字段的文档注释。
- 每个操作一组错误码常量（`note.create` → `NoteCreateErrors.emptyTitle`）。
- 操作表 `Ops`：每个操作一个有类型的异步方法（`noteCreate(CreateNoteInput)` 返回 `Future<NoteView>`），名字写错、少传字段，编译就报错。
- 给界面用的 Riverpod provider：每个查询一个 `<名字>Query`，动作都在 `OpActions` 里（用法见[界面](ui.md)的取数据）。

生成器只认核心会导出的那几种 Schema 写法：对象、字符串、时间、整数、小数、布尔、列表、引用。遇到别的直接报错，先在生成器里补上对应的写法，再在核心里用。

底下仍然是“按名字调用”，本机经 `<项目>-ffi` 的 `call`，连别的设备时经 HTTP，两种传输共用这一套类型。

flutter_rust_bridge 只导出 `open` 和 `call`，配置在 `apps/app/flutter_rust_bridge.yaml`，生成 `crates/<项目>-ffi/src/frb_generated.rs` 和 `apps/app/lib/ffi/`。类型从清单来，HTTP 传输才能和本机调用用同一套。

- **坑**：桌面会话里的 `RUST_LOG` 常带模块名（`模块=info`），`flutter_rust_bridge_codegen` 认不得会直接退出，`just gen` 里用 `env -u RUST_LOG` 跑它。
- 运行时提示缺 `cargo expand` 可以忽略，生成照样成功。

## 其他语言

以后有别的语言写的端，同样从 `schema/ops.json` 生成到那个端的目录里，命令加进 `just gen`。

## 规矩

- 生成的文件开头写“自动生成，不要手改”，改了也会被下一次 `just gen` 覆盖。
- 生成的文件入库，看提交就知道接口变了什么。
- `just check` 会重新导出清单、重新生成 Dart 类型，和仓库里的比较，有差别就失败：改了 Rust 类型忘了 `just gen`，提交前就会被拦下。
- flutter_rust_bridge 的胶水代码两头各记着一个内容哈希，对不上时应用启动就报错，`just e2e` 会失败。
- 清单里的 `version` 跟核心的版本走，应用可以据此提示“电脑上的版本太旧”。
