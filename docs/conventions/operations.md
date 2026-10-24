# 操作面

核心对外只有一个面：一组带名字的操作，加一个“按名字调用”的入口。各端只管把调用送进来、把结果送回去，不各自重写业务规则。加一个操作只改核心，所有端和生成的类型自动跟上。

## 操作

```rust
pub struct CreateNote;

impl Operation for CreateNote {
    const NAME: &'static str = "note.create";
    const KIND: Kind = Kind::Action;
    type Input = CreateNoteInput;
    type Output = NoteView;
    type Error = CreateNoteError;

    fn run<P: Ports>(core: &Core<P>, ctx: &Ctx, input: Self::Input)
        -> impl Future<Output = Result<Self::Output, Self::Error>> + Send { … }
}
```

- **名字**：`<业务>.<动作>`，全小写，多个词用下划线：`note.create`、`note.list`、`device.revoke`、`session.send_prompt`。业务就是核心里的模块名。
- **种类**：读的叫查询（`Kind::Query`），不改任何状态，可以重试、可以缓存；改的叫动作（`Kind::Action`），失败后不自动重试。查询用 `get`、`list`、`search` 这类词，动作用动词。
- **输入、输出**：每个操作都有，没有参数也写一个空结构体，和操作写在同一个文件里。都派生 `Serialize`、`Deserialize`、`JsonSchema`，是公开类型。字段用普通类型（`String`、数字、时间），规则写在文档注释里（会进 Schema 和生成的类型），校验在操作里构造领域类型时做。输出用专门的视图类型（`NoteView`），不直接把实体送出去，实体内部可以随便改。
- **输出是对象**：列表也包一层（`NoteList { notes }`），以后要加分页之类的字段不用改类型。
- **错误**：每个操作有自己的错误枚举，每种情况对应一个稳定的 `code`，见[错误和测试](errors-and-tests.md)。
- **上下文** `Ctx`：调用方是谁（本机直接调用，还是经网络进来的；接了认证后带上设备或用户）。由端填好传进来，权限在操作里判断。

## 登记

所有操作在 `core/src/lib.rs` 的一张表里登记一次：

```rust
operations! {
    note::CreateNote,
    note::ListNotes,
    note::DeleteNote,
}
```

这张表生成两样东西：

- `Core::call(ctx, name, input) -> Result<output, ErrorBody>`：输入输出是 `serde_json::Value`。按名字找到操作，解析输入、执行、序列化输出。名字不存在返回 `unknown_operation`，输入解析失败返回 `invalid_input`。
- `manifest()`：操作清单（名字、种类、输入输出的 JSON Schema、可能的错误码），给[类型生成](codegen.md)和 MCP 用。

Rust 写的端也可以不经 JSON：`core.run::<CreateNote>(ctx, input)` 直接拿到有类型的输出和错误，命令行的子命令就这样用。

写了操作没登记，生成的类型里就没有它，应用用不了；登记了再 `just gen`。

## 线上格式

- 输入输出都是 JSON 对象。字段名用 `snake_case`，和 Rust 一致，各端照用生成的类型。
- 时间点用 RFC 3339 字符串（UTC），时长用毫秒整数，编号用字符串。
- 可选字段缺省就是没有，不写 `null`：写成 `Option<T>` 加 `#[serde(default, skip_serializing_if = "Option::is_none")]`。生成 Schema 时会去掉 `null`，字段不在 `required` 里，生成的 Dart 类型里是可空的。
- 错误统一是：

```json
{ "code": "empty_title", "message": "标题不能为空" }
```

## 各端怎么传

| 端 | 怎么调用 | 错误 |
| --- | --- | --- |
| 命令行 | 常用操作有好记的子命令：`<项目> note create "标题"`；任何操作都能直接调：`<项目> op note.create '{"title":"…"}'` | 打印中文说明，退出码 1 |
| HTTP（`<项目> serve`） | `POST /api/op/<名字>`，`Content-Type: application/json`，请求体是输入（空的当作 `{}`），响应体是输出 | HTTP 状态码按下表，响应体是错误 |
| Flutter 应用，本机 | 经 `<项目>-ffi`：`open(数据目录)` 后 `call(name, input_json) -> output_json` | 带回 `{ code, message }` |
| Flutter 应用，连别的设备 | 同 HTTP | 同 HTTP |

HTTP 的状态码：

| `code` | 状态码 |
| --- | --- |
| `invalid_input` 和各种校验失败 | 400 |
| `unauthorized` | 401 |
| `forbidden` | 403 |
| `not_found`、`unknown_operation` | 404 |
| `conflict` | 409 |
| `internal` | 500 |

具体错误码映射到哪一类，由错误类型自己声明（见[错误和测试](errors-and-tests.md)），端里不写映射表以外的判断。

## 兼容

手机上的旧版应用会连电脑上新版的 `<项目> serve`，操作面一旦发布就要向后兼容：

- 已发布的操作不改名、不删、不改种类。要换，就加一个新操作，旧的在文档里标弃用，过一个大版本再删。
- 输入只加可选字段；输出只加字段，不删、不改类型。
- 错误码只加不改。

## 推送

需要实时更新的界面（状态变化、新消息）要 `serve` 主动推。推送的写法还没定，第一个要用的项目迁移时定下来，补在这里。
