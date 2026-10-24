# 目录结构

一个项目一个仓库，所有端都在里面，共用同一个核心。Rust 用 Cargo 工作区，客户端应用是 `apps/app` 下的一个 Flutter 工程。

## 顶层

```text
<项目>/
├─ apps/                 每个端一个目录，只做组装和界面
│  ├─ cli/               命令行，`<项目> serve` 也在这里
│  └─ app/               Flutter 应用：桌面、安卓、iOS
├─ crates/               Rust 库
│  ├─ <项目>-core/       核心：业务规则、操作面
│  ├─ <项目>-store-sqlite/   适配器：实现核心的存储端口
│  ├─ <项目>-local/      在本机组装核心：真的存储、时间、编号
│  ├─ <项目>-http/       把操作面开成 HTTP，`serve` 用它
│  └─ <项目>-ffi/        给 Flutter 应用的接口出口
├─ schema/               生成的操作清单和 JSON Schema（入库）
├─ docs/
├─ .github/workflows/    CI（每次推送跑 just ci）和发版
├─ Cargo.toml            Rust 工作区
├─ justfile              所有命令的入口
├─ rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml  _typos.toml
├─ .fvmrc                Flutter 的版本
├─ lefthook.yml          提交钩子
└─ README.md  AGENTS.md  CLAUDE.md  CHANGELOG.md  LICENSE-MIT  LICENSE-APACHE
```

根目录只放工作区配置、工具配置和顶层文档。构建产物、缓存、本机数据不入库（写进 `.gitignore`）。

许可证默认 MIT 或 Apache-2.0 双许可：根目录放 `LICENSE-MIT` 和 `LICENSE-APACHE` 两份原文，`[workspace.package]` 写 `license = "MIT OR Apache-2.0"`，README 末尾写一行许可证。

## apps/：各端

- 一个端一个目录，目录名是一个英文小写词：`cli`、`app`，以后的端照此起名。
- 端里只做三件事：读配置、建适配器和核心、把操作面接到自己的界面或协议上。业务规则一律写在核心里，端里出现 `if` 判断业务状态，就说明有规则漏在了外面。
- 只有这个端用的代码留在这个端里（比如桌面的托盘、命令行的参数解析）。两个以上的端都要用的 Rust 代码，挪到 `crates/`。
- 各端的写法见[各端](apps.md)。

## crates/：Rust 库

- 平铺在 `crates/` 下，不再分子目录。目录名和包名一致。
- 名字是 `<项目>-<用途>`：
  - `<项目>-core`：核心，见[核心分层](layering.md)。
  - 适配器按它实现的东西命名：`<项目>-store-sqlite`、`<项目>-github`、`<项目>-mailer-smtp`。
  - `<项目>-local`：把真端口组装成核心，在本机直接跑核心的端都用它。
  - `<项目>-http`：把操作面开成 HTTP，命令行的 `serve` 用它。
  - `<项目>-ffi`：给 Flutter 应用的接口出口。
- 开始只有一个 core。某块业务的代码多到改它时要在 core 里来回跳，或者有端只需要其中一部分时，把它拆成 `<项目>-<业务>`，依赖方向照旧。
- 每个 crate 有一个 `README.md`，写它管什么、不管什么、有哪些不变量（见[文档](docs.md)）。

## schema/

`just gen` 生成的操作清单和 JSON Schema，入库，CI 检查是否过期。见[类型生成](codegen.md)。

## 命名

| 东西 | 写法 | 例子 |
| --- | --- | --- |
| 项目 | 一个小写单词：小写字母开头，只有字母和数字 | `notes` |
| crate | `<项目>-<用途>`，小写加连字符 | `notes-core` |
| Rust 模块、文件 | 小写加下划线 | `store.rs`、`note/` |
| 业务模块 | 业务的单数名词 | `note`、`device`、`session` |
| 操作名 | `<业务>.<动作>`，见[操作面](operations.md) | `note.create` |
| Dart 文件 | 小写加下划线 | `note_list.dart` |
| Dart 类型 | 大驼峰 | `NoteList` |
| 文档 | 顶层大写（`README.md`），`docs/` 下小写 | `docs/development.md` |
