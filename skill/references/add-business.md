# 加一块业务

先读 `docs/conventions/layering.md`、`docs/conventions/operations.md`、`docs/conventions/errors-and-tests.md`，要做界面再读 `docs/conventions/ui.md`。照着备忘写：每个文件的分工和备忘里同名的文件一样，先打开备忘的那个文件再写自己的。

业务名是单数名词（`device`、`session`），也是模块名和操作名的前缀（`device.list`）。

## 步骤

1. **核心** `crates/<项目>-core/src/<业务>/`：
   - `model.rs` 领域类型：有规则的值用 newtype，构造时校验；实体的改动写成它自己的方法。
   - `ports.rs` 这块业务要的外部能力，按能力命名（`DeviceStore`），每个端口有自己的错误枚举。
   - `ops.rs` 操作和它们的输入输出，`error.rs` 每个操作的错误（`op_error!`，错误码是稳定的英文短词，说明写中文）。
   - `fake.rs` 内存版的端口实现，`contract.rs` 端口的约定测试，`mod.rs` 导出。
   - 在 `crates/<项目>-core/src/lib.rs` 加 `pub mod <业务>;`，操作登记进 `operations!`。
   - 新端口加进 `crates/<项目>-core/src/ports.rs` 的 `Ports` 总表，假实现接进 `crates/<项目>-core/src/fake.rs` 的那一整套假端口。
   - 测试用假端口：每个操作的正常路径和每个错误码至少一条。
2. **适配器**：
   - 存数据的：在 `crates/<项目>-store-sqlite` 实现新端口，表结构写成 `crates/<项目>-store-sqlite/migrations/` 里的新迁移（编号接着往下排，发布过的迁移不改），测试里跑核心的约定测试（照 `crates/<项目>-store-sqlite/tests/store.rs`）。
   - 别的外部能力（网络服务、系统功能）：新建 `crates/<项目>-<实现>`，带 `README.md`（模板 `docs/conventions/templates/crate-README.md`），对本地假服务器测。
3. **本机组装** `crates/<项目>-local/src/lib.rs`：把真的实现接进 `Ports`。
4. **命令行** `apps/cli/src/main.rs`：常用的操作加好记的子命令（`<项目> <业务> <动作>`），其余靠 `<项目> op <名字>`；测试加在 `apps/cli/tests/cli.rs`。
5. `just gen`：导出 `schema/ops.json`，生成 `apps/app/lib/gen/ops.dart`。
6. **应用**：页面放在 `apps/app/lib/` 下按业务分的目录（备忘的是 `apps/app/lib/notes/`），只经生成的 provider 和操作表调操作，路由加在 `apps/app/lib/app.dart`。纯逻辑（格式化、排序）写成函数放在同一个目录，单元测试放 `apps/app/test/`；集成测试 `apps/app/integration_test/app_test.dart` 里加一条走通这块业务的。
7. **文档**：`CHANGELOG.md` 的“未发布”；加了 crate 时更新 `docs/ARCHITECTURE.md` 的模块关系；有新的边界就补一条不变量。
8. `just ci`。

## 删掉示例业务“备忘”

第一块业务写完、`just ci` 通过后就删，别留到以后。要删的：

- 核心：`crates/<项目>-core/src/note/` 整个目录，`lib.rs` 里的模块和三个操作的登记，`ports.rs` 里的 `Notes`，`fake.rs` 里的内存备忘。
- SQLite 存储：备忘的实现和测试，`crates/<项目>-store-sqlite/migrations/0001_notes.sql`。项目还没发布过就直接删，迁移从 `0001` 重新编号；发布过就加一个删表的新迁移。
- 本机组装：`crates/<项目>-local/src/lib.rs` 里接备忘存储的那几行，`crates/<项目>-local/tests/local.rs` 换成新业务的操作。
- 命令行：`note` 子命令和 `apps/cli/tests/cli.rs` 里用它的测试。
- HTTP：`crates/<项目>-http/tests/http.rs` 用的是备忘的操作，换成新业务的。
- 应用：`apps/app/lib/notes/`、路由、`apps/app/test/format_test.dart`，集成测试里备忘的那条。
- 文档：`AGENTS.md` 和 `docs/development.md` 里 `just dev note list` 这类例子，`docs/ARCHITECTURE.md` 里提到备忘的地方，各 crate 的 `README.md`。

删完 `just gen`，再 `git grep -niE '\bnotes?\b|备忘'` 查漏，`just ci` 通过。
