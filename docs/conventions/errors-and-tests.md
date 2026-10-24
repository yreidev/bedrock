# 错误和测试

## 错误

### 三种错误，三种处理

| 种类 | 例子 | 怎么处理 |
| --- | --- | --- |
| 调用方能处理的 | 标题为空、记录不存在、没有权限、冲突 | 操作的错误枚举里单独一种情况，带稳定的 `code` 和中文说明 |
| 意料之外的 | 数据库连不上、磁盘满、适配器认不出的错误 | 统一报 `internal`，详情只进日志，不发给调用方 |
| 程序本身的毛病 | 违反了代码里自己保证的前提 | 可以 panic，但要在旁边写注释说明为什么不可能发生 |

### 写法

```rust
#[derive(Debug, thiserror::Error)]
pub enum CreateNoteError {
    #[error(transparent)]
    Title(#[from] NoteTitleError),      // 领域类型的错误直接带过来，说明只写一次
    #[error(transparent)]
    Internal(#[from] Internal),
}

op_error! {
    CreateNoteError {
        Title(NoteTitleError::Empty) => "empty_title", Invalid;
        Title(NoteTitleError::TooLong) => "title_too_long", Invalid;
        Internal(_) => "internal", Internal;
    }
}
```

- 每个操作一个错误枚举，用 thiserror。`code` 是稳定的英文小写短词，给程序判断；`#[error]` 里是中文说明，给人看，写清楚出了什么事、用户能怎么办。
- `OpError` 用 `op_error!` 实现：每种情况写一次 `code` 和大类（大类决定 HTTP 的状态码：`Invalid`、`NotFound`、`Conflict`……），同时生成操作清单里的错误码列表，不会漏写。
- 端口的错误（如 `StoreError`）在操作里翻译成操作的错误：`StoreError::NotFound` 变成 `DeleteNoteError::NotFound`，其他的变成 `Internal`。
- `Internal` 保留原始错误的完整链，记日志时打印出来；对外只给 `code: "internal"` 和一句“出了点问题，详情在日志里”。
- 应用内部的错误枚举不加 `#[non_exhaustive]`：加了新情况，编译器会指出所有没处理的地方。
- `anyhow` 只用在各端的启动和组装代码里（读配置、建适配器失败就退出）。核心和适配器用有类型的错误。
- 正式代码里不用 `unwrap`、`expect` 处理可能失败的事（clippy 会拦）。确实不可能失败的，给一个说得通的退路（`unwrap_or_default()`、`unwrap_or(原值)`），旁边写注释说明为什么不会走到。

### 日志

- 用 `tracing`。核心也可以记日志，但不直接打印到终端。
- 日志写给排查问题的人：带上操作名、关键参数，有请求编号时也带上。不记密码、令牌、密钥、用户内容的全文。

## 测试

### 分层

| 测什么 | 怎么测 | 放在哪 |
| --- | --- | --- |
| 领域类型和规则 | 直接调构造函数和方法 | 同一文件的 `#[cfg(test)] mod tests` |
| 操作 | 用内存版的假适配器组一个核心，按名字调用，检查输出和错误码 | 核心里同一文件，或 `core/tests/` |
| 适配器 | 对真东西测：临时目录里的 SQLite、本地起的假服务器 | 适配器 crate 的 `tests/` |
| 各端 | 端到端：起真的程序，从外面调用 | 每个端至少一条，见下文 |

### 假适配器

- 每个端口都有一个内存版实现，手写，放在业务模块的 `fake.rs`，用 `fake` 特性打开。它和真适配器守同样的约定（比如编号重复返回 `Conflict`），所以测出来的结果可信。
- 核心的 `fake.rs` 把它们和假的时间、编号拼成一整套 `FakePorts`，`fake::core()` 直接得到一个核心。同一套也给演示模式用：不用装数据库就能把程序跑起来看。
- 时间和编号用固定的假实现（每问一次往后走一秒、`id-1`、`id-2`……），测试结果每次都一样，可以直接比较整个输出。
- 每个存储端口有一组约定测试，写在业务模块的 `contract.rs`：一个接收空存储、不守约定就 panic 的函数。内存版在核心里跑它，每个真适配器在自己的测试里也跑它，保证两者行为一致。

### 端到端

- 命令行：跑编出来的程序（`env!("CARGO_BIN_EXE_<名字>")`），数据目录指到临时目录，时区固定成 UTC，检查输出和退出码。
- HTTP：在 `<项目>-http` 的测试里绑一个随机端口起服务，用 HTTP 调操作；只收 JSON、DNS 重绑定的防护、安全头也在这里测。
- `serve`：命令行的测试里起 `<项目> serve --listen 127.0.0.1:0`，从它打的那一行读出端口，调一次操作，再发 SIGTERM，看它正常退出。
- Flutter 应用：用 Flutter 的 integration_test，按手机和桌面两种尺寸、浅色和深色各过一遍。找控件用中文的语义标签，不按控件类型和位置。

### 规矩

- 测试不连外网，不依赖本机装了什么服务，跑几遍结果都一样。
- 测试数据是编出来的，不拿真实用户的数据当样本。
- 输出多、结构复杂的结果用 insta 快照，改了快照要看一眼差异再接受。
- 修一个缺陷，先写一个能复现它的失败测试，再改代码。
- 测试函数名写成一句英文描述行为（`rejects_empty_title`），端到端测试的标题用中文写成一句话。
