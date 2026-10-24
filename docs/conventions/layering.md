# 核心分层

业务规则只写一份，写在 `crates/<项目>-core` 里，所有端共用。核心按业务分模块；它需要的外部能力（存数据、发请求、看时间）写成端口，由适配器实现；依赖只能从外往里指。

## 和 DDD 的对应

| 这里的叫法 | DDD 里的叫法 | 在哪 |
| --- | --- | --- |
| 业务模块 | 限界上下文 | `core/src/<业务>/` |
| 领域类型 | 实体、值对象 | `<业务>/model.rs` |
| 端口 | 仓储接口、外部服务接口 | `<业务>/ports.rs`、`core/src/ports.rs` |
| 操作（用例） | 应用服务 | `<业务>/ops.rs` |
| 适配器 | 基础设施 | `crates/<项目>-<实现>` |
| 各端 | 接口层 | `apps/<端>` |

## 依赖方向

```text
apps/<端>  ──▶  crates/<项目>-<适配器>  ──▶  crates/<项目>-core
     └────────────────────────────────────────▶  ▲
```

- 端依赖适配器和核心，适配器依赖核心，核心谁也不依赖。
- 核心不依赖这几类库：数据库客户端、HTTP 和网络、Web 框架、异步运行时、界面和平台框架（flutter_rust_bridge）、`anyhow` 这类“什么错误都装”的库。名单写在 `crates/<项目>-core/deny.toml`，`just check` 单独对核心跑一遍 cargo-deny，加了就失败。
- 核心也不直接碰外部世界：不读环境变量、不读写文件、不看系统时间、不起后台任务、不睡眠。`crates/<项目>-core/clippy.toml` 用 `disallowed-methods` 禁掉这些函数。要用时写成端口。
- 好处：核心能在任何地方跑（电脑、手机、以后的端），测试不用起数据库，换存储只换适配器。

## 业务模块

```text
crates/<项目>-core/src/
├─ lib.rs          对外导出、操作登记表
├─ ports.rs        跨业务的端口（时间、编号）和端口总表 Ports
├─ ops.rs          操作面：Operation、Core::call、操作清单
├─ error.rs        对外错误的共同部分（code、message、op_error!）
├─ fake.rs         跨业务端口的假实现和一整套假端口，测试和演示模式用
└─ note/           一块业务
   ├─ mod.rs
   ├─ model.rs     领域类型
   ├─ ports.rs     这块业务要的外部能力
   ├─ ops.rs       这块业务的操作，以及它们的输入输出
   ├─ error.rs     这块业务的错误
   ├─ fake.rs      内存版的端口实现
   └─ contract.rs  端口的约定测试，假的和真的实现都跑
```

`fake.rs`、`contract.rs` 只在测试和 `fake` 特性打开时编译。

- 按业务分，不按层分：改“备忘”只动 `note/`。
- 文件按需要建。小业务可以先都写在 `mod.rs` 里，边界比文件数重要：领域类型不依赖端口和操作，端口不依赖操作。
- 业务模块之间只用对方公开的类型和函数，不伸手进对方内部；不许互相依赖成环。两块业务都要的概念，单独提成一个模块。

## 领域类型

非法的值造不出来，规则写在类型上。

```rust
/// 备忘的标题：去掉首尾空白后 1 到 200 个字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteTitle(String);

impl NoteTitle {
    pub fn new(raw: &str) -> Result<Self, NoteTitleError> { … }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

- 有规则的值用 newtype，字段私有，只能经构造函数得到。
- 领域类型不派生 `Deserialize`、`JsonSchema`，不出现在操作的输入输出里。操作的输入用普通类型（`title: String`），在操作里构造领域类型，校验失败报这个操作自己的错误码（`empty_title`），而不是笼统的 `invalid_input`。这样也没有绕过校验的口子。
- 适配器从存储里读回来时也走构造函数，存储里的脏数据进不了核心。
- 实体（有编号、会变的东西，如 `Note`）的改动写成它自己的方法（`note.rename(title, now)`），方法里守住规则；外面不直接改字段。
- 操作的输入类型和实体分开：`CreateNote` 是“请求建一条备忘”，`Note` 是“一条备忘”，两者会各自变化。
- 没有规则、只是存取的数据，用普通结构体加公开字段就行，不为分层而分层。

## 端口

核心需要外部提供的能力，按能力命名，不按技术命名：`NoteStore`，而不是 `SqliteNotes`。

```rust
pub trait NoteStore: Send + Sync + 'static {
    /// 编号已存在时返回 `StoreError::Conflict`。
    fn insert(&self, note: &Note) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn list(&self) -> impl Future<Output = Result<Vec<Note>, StoreError>> + Send;
}
```

- 方法写成返回 `impl Future<Output = …> + Send`，保证各端能把操作放进多线程运行时。
- 每个端口有自己的错误枚举，列出调用方要分别处理的情况，再加一个兜底的 `Other(Box<dyn Error + Send + Sync>)`。适配器把库的错误翻译成这些情况，认不出的放进兜底。
- 时间、编号、随机数也是端口（`Clock`、`Ids`），测试里能固定住。
- 一个动作要同时改几样东西、必须一起成功时，把“这件事”做成端口上的一个方法，由适配器在一个事务里完成。核心不知道事务。
- 所有端口收在 `ports.rs` 的总表里，核心按总表取：

```rust
pub trait Ports: Send + Sync + 'static {
    type Clock: Clock;
    type Ids: Ids;
    type Notes: NoteStore;
    fn clock(&self) -> &Self::Clock;
    fn ids(&self) -> &Self::Ids;
    fn notes(&self) -> &Self::Notes;
}

pub struct Core<P: Ports> { ports: P }
```

端口用泛型传进核心：没有运行时开销，端口方法可以直接返回 `impl Future`；测试和演示模式换一套总表就行。

## 操作

每个用例是一个操作，写在 `<业务>/ops.rs`：取输入、调领域类型和端口、返回输出或错误。名字、登记和对外的样子见[操作面](operations.md)。

- 操作里只做编排：先校验输入（构造领域类型），再按顺序调端口，最后组装输出。规则本身写在领域类型的方法里。
- 权限在操作里判断（调用方是谁由上下文带进来），不在端里判断。

## 适配器

`crates/<项目>-<实现>` 实现核心的端口，例如 `<项目>-store-sqlite` 实现 `NoteStore`。

在本机跑核心要的一整套真端口（存储、系统时间、随机编号）由 `crates/<项目>-local` 组装好，`<项目>_local::open(数据目录)` 直接得到核心。命令行（含 `serve`）和给 Flutter 的接口出口都用它，各端不各自组装端口；加一个端口时在那里接上真的实现。

- 只做翻译：领域类型和存储格式互转，库的错误转成端口的错误。不写业务规则。
- 存储格式（表结构、迁移）属于适配器，领域类型不带数据库的注解。
- 对真东西测（临时 SQLite、本地假服务器），见[错误和测试](errors-and-tests.md)。

## 什么时候可以简化

DDD 的价值在规则多的业务上。只是存取数据、规则很少的部分可以简化：领域类型用普通结构体，操作直接调端口。但这两条不能省：外部能力走端口，核心不依赖适配器。
