# 给已有的业务加一个操作

先读 `docs/conventions/operations.md`。照着备忘的 `crates/<项目>-core/src/<业务>/ops.rs` 写。

1. 定名字和种类：名字是 `<业务>.<动作>`，全小写，多个词用下划线。只读不改的是查询（`Kind::Query`，用 `get`、`list`、`search` 这类词），会改状态的是动作（`Kind::Action`，用动词）。
2. 在 `<业务>/ops.rs` 写输入、输出和操作：
   - 没有参数也写一个空的输入结构体。字段用普通类型，规则写在文档注释里，校验在操作里构造领域类型时做。
   - 输出用专门的视图类型，列表也包一层对象。
   - 可选字段写 `Option<T>` 加 `#[serde(default, skip_serializing_if = "Option::is_none")]`。
3. 在 `<业务>/error.rs` 用 `op_error!` 写这个操作的错误枚举，每种情况一个稳定的错误码，声明它属于哪一大类（决定 HTTP 状态码）。
4. 要新的外部能力时，在 `<业务>/ports.rs` 的端口上加方法，`fake.rs` 里的假实现、`contract.rs` 的约定测试、各适配器的真实现一起加。
5. 登记进 `crates/<项目>-core/src/lib.rs` 的 `operations!`。没登记的操作不进清单，各端都用不了。
6. 核心里用假端口测：正常路径和每个错误码。
7. `just gen`，生成的文件一起提交。各端按需要接：命令行加子命令（`apps/cli/src/main.rs`），应用的页面用生成的 provider。
8. 操作发布过以后要向后兼容：不改名、不删、不改种类；输入只加可选字段，输出只加字段，错误码只加不改。要换就加新操作，旧的标弃用。
9. `CHANGELOG.md` 的“未发布”写一条，`just ci`。
