# bedrock 计划

bedrock 是以后所有项目的地基：规范文档、一个能直接编译运行的多端骨架，加一个全局技能，让每个会话建项目、加功能时照着做。

## 目标项目的样子

- 核心逻辑用 Rust 写。
- 一个仓库放所有端：无界面的命令行（也负责起服务，让别的设备经网络连进来），和一个 Flutter 客户端应用（Windows、macOS、Linux、安卓、iOS），以后还会加别的端。各端共用同一套核心，不各自重写业务规则。没有网页端。
- 仓库用 Cargo 工作区管 Rust，Flutter 应用在 `apps/app`。

## 已定

| # | 问题 | 结论 | 日期 |
| --- | --- | --- | --- |
| 1 | 规矩放在哪、做成什么形式 | 独立的规范仓库，放规范文档和能编译运行的多端骨架（各目录都有最小示例），骨架由 CI 验证；另做一个全局技能，让会话建项目、加功能时按它来；全局 CLAUDE.md 只加一行指向技能 | 2026-10-24 |
| 2 | 仓库名 | `bedrock`，放在 GitHub 上，全局技能同名。（2026-10-24 改：原来是私有仓库，改为公开，MIT 或 Apache-2.0 双许可） | 2026-10-24 |
| 3 | 顶层目录 | `apps/` 每个端一个目录：`cli`（命令行）、`app`（Flutter 客户端），只做组装和界面；`crates/` 放 Rust 库，核心也在这里（`<项目>-core`）；`docs/` 放文档。（2026-10-24 改：去掉 `server`、`web` 和放前端包的 `packages/`，因为不做网页端，服务并进命令行，见第 6、7 题） | 2026-10-24 |
| 4 | 任务入口 | 根目录 `justfile`：`just check / test / build / dev <端>` 等。工具版本写在各自的文件里：Rust 用 `rust-toolchain.toml`，Flutter 用 `.fvmrc`，cargo 工具由 `just deps` 按固定版本装。（2026-10-24 改：原来写的是 Node 和 pnpm 用 `package.json`，不做网页后改成 Flutter） | 2026-10-24 |
| 5 | 核心分层 | 先按业务分：`<项目>-core` 里每块业务一个模块，模块里分领域类型（构造时校验，非法值造不出来）、端口（需要外部提供的能力，写成 trait）、用例（查询和动作）、错误。数据库、网络、第三方服务的实现放单独的适配器 crate。依赖方向只能是 `apps` → 适配器 → core；core 不依赖任何适配器，也不依赖数据库、HTTP、Web 框架、界面框架这类库，`just check` 自动查。先只有一个 core，某块业务长大了再拆成自己的 crate | 2026-10-24 |
| 6 | 各端怎么调用核心 | 统一操作面：核心把每个用例登记成带名字的操作（如 `session.list`、`device.revoke`），输入输出是公开类型，另有一个按名字调用的入口。各端只管传输：命令行既有好记的子命令，也能 `op <名字>` 直接调，`serve` 子命令把操作面经 HTTP 开给别的设备（路由在 `crates/<项目>-http`）；Flutter 应用经 flutter_rust_bridge 在本机直接调，或经网络连另一台电脑上的 `serve`。加一个操作只改核心，各端和 Dart 类型自动跟上；以后给 AI 代理（MCP）也用同一套名字和结构。（2026-10-24 改：服务端不再是单独的端，并进命令行，项目只剩命令行和客户端应用两个端） | 2026-10-24 |
| 7 | 带界面的端 | 桌面（Windows、macOS、Linux）、安卓、iOS 是同一个 Flutter 应用 `apps/app`，一套界面、一种语言（Dart）；桌面和手机的差别用布局和平台判断分开。Rust 核心经 `crates/<项目>-ffi`（flutter_rust_bridge）跑在应用进程里，只导出 `open` 和 `call(名字, 输入 JSON)`；平台缺的能力用 Flutter 插件补，没有现成的就用 Kotlin / Swift 写。原因：手机体验很重要，要原生的流畅度和手势；一个人维护，桌面和手机共用一个应用最省事；Rust 核心加 Flutter 在桌面和手机上都有成熟产品在用。桌面的已知短板：稳定版没有公开的多窗口接口，托盘、菜单栏靠社区插件，Linux 上的中文输入法要实测（B3 做）。（2026-10-24 改：原来是 Tauri 应用加 UniFFI 出口；定了不做网页、手机用 Flutter，桌面也随之用 Flutter） | 2026-10-24 |
| 8 | 类型生成 | 操作的输入输出派生 schemars，`just gen` 导出操作清单加 JSON Schema（一份来源）。Flutter 应用的 Dart 类型和带类型的调用封装从它生成；MCP 的工具定义也直接用它。生成的文件入库，CI 重新生成后有差别就失败。Dart 类型用什么工具生成，B4 前定（定了，见第 17 题）。（2026-10-24 改：原来生成 TS 给网页和 Tauri） | 2026-10-24 |
| 9 | 文档 | 文档一律中文，各管一件事，不写逐文件地图（目录和文件是什么，读代码就知道，这种地图很快过时）。`README.md` 给用户：一句话、亮点、安装、上手、文档目录、许可证，约一屏半；`AGENTS.md` 给代理：命令、质量关卡、要避开的坑；`CLAUDE.md` 只有一行 `@AGENTS.md`；`CHANGELOG.md` 记每个版本改了什么；`docs/ARCHITECTURE.md` 写模块关系、边界和不变量，不随代码同步改，一年看两次；`docs/PLAN.md` 记决定（带日期和原因）和进度；`docs/development.md` 写环境、命令、调试；`docs/release.md` 写发版步骤；每个 crate 的 `README.md` 写它管什么、不管什么 | 2026-10-24 |
| 10 | 检查力度 | 标准档。Rust：clippy 默认规则零警告，另开禁止 `dbg!`、`todo!`、正式代码里的 `unwrap`、core 里直接打印；`unsafe` 只允许在 ffi crate 里；cargo-deny 查许可证、已知漏洞和 core 的禁用依赖；typos 查拼写；cargo-machete 查没用的依赖。Flutter：`dart format`、`flutter analyze` 零警告，lint 规则集 B4 前定（2026-10-24 改：原来是 TypeScript、ESLint、Prettier）。lefthook 提交钩子只查暂存的文件，gitleaks 扫密钥；提交说明首行写成“类型: 说明”，不带 AI 署名。CI 跑 `just ci`，和本机一致 | 2026-10-24 |
| 11 | 错误和测试 | 错误：每个操作返回有类型的错误（thiserror），对外是 `code` 加 `message`，`code` 是稳定的英文短词给程序判断，`message` 是中文给人看；意料之外的统一报 `internal` 并记日志，不把内部细节发给前端；anyhow 只在各端的启动和组装代码里用；预期内的失败不用 panic。测试：core 用手写的内存版假适配器测（实现同样的端口，也能给演示模式用）；适配器对真东西测（临时 SQLite、本地假服务器）；每个端至少一条端到端（命令行跑编出来的程序；Flutter 应用用 integration_test 在桌面上跑）；输出多的用快照（insta）；测试不连外网、可重复 | 2026-10-24 |
| 12 | 先试哪个项目 | bedrock 的骨架和技能做完、CI 跑通后，挑一个手上的 Rust 项目按新规矩重组，一步到位：原来的 Rust 代码拆成 `crates/<项目>-core`、适配器和 `apps/cli`（`<项目> serve`），同时做 Flutter 客户端应用，原来的网页端删掉。它是 Rust、有服务、要做手机端，正好检验这套规矩；还没提交过，重组没有历史包袱。不合适的地方回头改 bedrock。（2026-10-24 改：定了直接换成 Flutter，不保留网页；做完之前手机上没有客户端可用） | 2026-10-24 |
| 13 | 前端技术栈 | Flutter（Dart），桌面和手机共用。状态管理、路由、组件库、Dart 类型生成和 lint 规则集 B4 前逐题定（2026-10-24 定了，见第 14–18 题）。（2026-10-24 改：原来是 React + TypeScript + HeroUI + TanStack，给网页和 Tauri 用；不做网页、改用 Flutter 后作废） | 2026-10-24 |
| 14 | Flutter 的状态管理 | Riverpod 3（flutter_riverpod），先不用它的代码生成。查询的缓存、出错重试、按输入分实例、失效重取都是它自带的，正好对上规范里“查询可缓存可重试、动作成功后按业务前缀刷新”。选它时是 3.4.3（2026-09-03 发布），这几个候选里下载最多，MIT 许可证；Bloc 超过一年没发版，signals 用的人少，只用 Flutter 自带的等于自己再造一个 | 2026-10-24 |
| 15 | Flutter 的路由 | go_router，Flutter 官方团队维护，先不用它的代码生成。点通知直接打开某一页、宽屏时左栏不动右边换页的嵌套布局都是现成的。选它时是 18.1.1（2026-10-20 发布）；auto_route 必须用代码生成，只用自带的 Navigator 要自己补这些 | 2026-10-24 |
| 16 | Flutter 的组件库 | shadcn_ui，和 HeroUI 同一类简洁风格。HeroUI 没有官方的 Flutter 版（只有网页和 React Native），pub.dev 上的个人移植几乎没人用；Material 也能调成那个样子，但想要现成就接近的。选它时是 0.57.3（2026-10-22 发布，MIT），还没到 1.0、小版本常有不兼容改动，所以版本锁死，升级单独提交并看它的更新记录 | 2026-10-24 |
| 17 | Dart 类型怎么生成 | 自己写一个小生成器：`apps/app` 里的 Dart 脚本读 `schema/ops.json`，用 dart-lang 官方的 code_builder 拼代码、dart_style 格式化，`just gen` 调它（2026-10-24 改为生成器只拼代码、由 `just gen` 接着跑 SDK 自带的 `dart format`，原因：dart_style 带着整个 analyzer，每次 `dart run` 现场编译生成器要多花二十秒，排版结果一样）。生成每个输入输出的类（字段转驼峰、时间转 `DateTime`、可选字段没值就不写）、每个操作的错误码、一张带类型的操作表；只认核心会导出的那几种 Schema 写法，遇到别的直接报错。实测 quicktype 时它把 `NoteView` 起名成 `All`、空结构变成 `Map`、可选字段写出 `null`，也不生成操作表 | 2026-10-24 |
| 18 | Dart 的 lint | flutter_lints（`flutter create` 自带的官方规则）零警告，另开分析器的严格模式（`strict-casts`、`strict-inference`、`strict-raw-types`，相当于 TS 的 strict），再加几条真会出错的规则（漏写 `await`、对 `dynamic` 调方法）。和 Rust 那边“默认规则零警告再加几条”同一档；very_good_analysis 太严，写应用时要关掉一批 | 2026-10-24 |
| 19 | 全局技能放哪 | 技能的文件放在 bedrock 仓库的 `skill/`，`~/.claude/skills/bedrock` 是指向它的符号链接。技能和规范、骨架一起改、一起提交，bedrock 自己的 CI 查技能里提到的文件还在不在；换电脑时克隆仓库再链接一次 | 2026-10-24 |
| 20 | 新项目的名字 | 只许一个小写单词：小写字母开头，只有字母和数字（`crook`、`tillbox`、`relaypocket`）。`just new` 把 `bedrock`、`Bedrock`、`BEDROCK` 直接换成新名字的三种写法，crate 名、Rust 标识符、Dart 包名、安卓 applicationId、类型名、环境变量都合法；带连字符的名字要按位置换成好几种写法，骨架里写错一处新项目就编不过 | 2026-10-24 |
| 21 | 新项目带不带规范 | 不带。规范只在 bedrock 一处，新项目的 `AGENTS.md` 写明按 bedrock 的规矩，并记下建项目时 bedrock 的提交；本机会话经全局技能读规范，别人看 GitHub 上那个提交的 `docs/conventions/`。bedrock 的规矩改了以后，技能对比这个提交列出项目要跟着改的地方 | 2026-10-24 |
| 22 | 新项目里的示例业务和端 | 全都留着：`just new` 建出来就是完整骨架（命令行、Flutter 应用、示例业务“备忘”），当场能过 `just ci`。加第一块业务时照着备忘写，写完删掉备忘；不要某个端（比如只要命令行）时按技能里的步骤删掉 | 2026-10-24 |

## 顺带定下的小事

问答里没单独问、按推荐做法定的，不合适随时改：

- bedrock 仓库根目录本身就是骨架，CI 直接验证它；规范文档放 `docs/conventions/`。新项目用 `just new <名字> <目录>` 复制骨架并把 `bedrock` 换成新名字。
- 骨架里的示例业务是“备忘”（新建、列出、删除），够演示领域类型的校验、端口、存储适配器、错误码和各端调用，又不会让示例喧宾夺主。

B2 写骨架时定下的（2026-10-24）：

- Rust 工具链固定 1.99.1（当天最新的稳定版），最低版本也写 1.99。
- 多了一个 `crates/bedrock-local`：把 SQLite 存储、系统时间、随机编号组装成核心。命令行、服务端、Tauri 应用、原生接口出口都要在本机跑核心，组装只写一份。（B3 后用它的是命令行和给 Flutter 的接口出口）
- 操作的输入用普通类型（`title: String`），在操作里构造领域类型。这样校验失败能报具体的错误码（`empty_title`），领域类型也不用派生 `Deserialize`。规范里原来写的“领域类型用 `serde(try_from)`”改掉了。
- 错误码由 `op_error!` 宏一处写好，同时给出 `code`、大类和清单里的错误码列表；清单的错误码包括 `internal`。
- 列表操作的输出也是对象（`{ notes: [...] }`），以后加分页不改类型。
- 可选字段在导出 Schema 时去掉 `null`，字段不在 `required` 里。（B3 改：原来写的是生成的 TS 是 `body?: string`，现在生成 Dart）
- 编号用 UUID v7，时间精确到毫秒，SQLite 里存 UTC 毫秒数。
- 服务端只收 `Content-Type: application/json`（挡跨站请求伪造）；只监听本机时只认本机的主机名（挡 DNS 重绑定）；默认端口 7400。（B3 后这些都在 `bedrock-http`，由 `bedrock serve` 用）
- 命令行直调操作写成 `bedrock op <名字> '<输入>'`，`--json` 只表示“输出原始 JSON”；`bedrock ops --json` 导出操作清单，`just gen` 就用它。
- 工作区没写许可证：bedrock 是私有仓库，新项目在 `just new` 时自己定。（2026-10-24 改：bedrock 要公开，写了 MIT 或 Apache-2.0 双许可，新项目也默认用它，见 `docs/conventions/layout.md`）
- 原生接口出口导出一个对象 `Bedrock`（`open` 加 `call`），绑定生成到 `target/ffi/`，不入库。（B3 删掉了：改用 flutter_rust_bridge，B4 重写）
- `basic-toml` 不再维护的通报（RUSTSEC-2026-0374）先放过：只经 UniFFI 生成绑定的工具引入，不进发行版。（B3 去掉 UniFFI 后这条放行也删了）

B3 定下的（2026-10-24）：

- `bedrock serve` 的监听地址用 `--listen` 或 `BEDROCK_LISTEN`（原来服务端用的是 `BEDROCK_ADDR`），选项和环境变量同名好记。
- `serve` 准备好后往标准输出打一行 `在 http://<地址> 上提供服务`，端口写 0 时是系统分配的端口，测试读这一行。监听的不是本机地址时打一条警告：还没有认证。
- 命令行平时用单线程的异步运行时，只有 `serve` 用多线程；`serve` 的日志默认 `info`，其余命令默认 `warn`。
- 不托管网页后，任何未知路径都回 JSON 的 `not_found`；内容安全策略收紧成 `default-src 'none'; frame-ancestors 'none'`。
- 实测 Flutter 桌面：发行版的包 25 MB，渲染用 Impeller。窗口用 window_manager 0.5.2（最小尺寸、最小化、最大化、关窗口改成藏到托盘都能做），托盘用 tray_manager 0.8.0（底层的原生库随包带源码在本机编译）。**坑**：tray_manager 的新接口默认不自动弹菜单，Linux 上托盘项这时报告“没有菜单”，右键没反应，要显式设成右键弹出；退出前先摘掉窗口监听，不然销毁窗口时的关窗事件会去动已经没了的窗口。这两个库是不是正式用，B4 定组件时再说。
- Flutter 用 fvm 固定版本，`.fvmrc` 放仓库根目录，`.fvm/` 不入库。fvm 默认把 SDK 放在家目录下的 `fvm/`，换到 `~/.local/share/fvm`。版本是 3.47.8（当天最新的稳定版，带 Dart 3.13.7）。

B4 定下的（2026-10-24）：

- Rust 用 Dart 的原生构建钩子打进应用（`hook/build.dart` 加 flutter_rust_bridge_hooks 2.13.0，底下是 native_toolchain_rust），编译 Rust 只靠这个钩子。ffi crate 自带 `rust-toolchain.toml`，写明 Rust 版本和要打包的平台，rustup 缺什么自动装。
- 给 Dart 的接口是一个不透明的句柄：`Bedrock.open(dataDir)` 拿到句柄，`句柄.call(name, inputJson)`。ffi crate 自己带两线程的 tokio 运行时。
- 应用启动时选传输：编译时给 `BEDROCK_SERVER` 就连那台电脑的 `bedrock serve`，不给就在本机跑核心，数据放系统给应用的数据目录。
- 查询只重试 `unreachable` 和 `internal`，最多三次。
- 集成测试在 Linux 桌面上跑两遍同样的步骤：一遍本机跑核心，一遍起 `bedrock serve` 再连过去（`just e2e`，进 `just ci`）。
- 安卓正式包先只在本机跑核心：没有网络权限，安卓也默认不让走明文 HTTP；等服务加了认证和加密传输再开。
- 托盘、窗口管理（B3 实测的 tray_manager、window_manager）这一步还没用上，等有项目要用时再加进骨架。
- **坑**：桌面会话里的 `RUST_LOG` 带模块名时，`flutter_rust_bridge_codegen` 直接退出，`just gen` 里去掉这个变量再跑。
- **坑**：cargo 依赖文件里的目录不带结尾斜杠时，原生构建钩子把它当成不存在的文件，每次构建都重跑。SQLite 存储的 `build.rs` 改成 `rerun-if-changed=migrations/`。
- **坑**：集成测试按语义标签找控件要先 `ensureSemantics()`，并且在测试函数结束前关掉，`addTearDown` 太晚。
- **坑**：原生构建钩子只把白名单里的环境变量传给 cargo，`CARGO_BUILD_JOBS` 进不去，rustc 按 CPU 核数一起开，内存小的机器会卡住。要限就在本机的 `~/.cargo/config.toml` 写 `[build] jobs`，仓库不写死。
- **坑**：Linux 上 path_provider 在应用的目录还不存在时，会退回用可执行文件名的目录，和命令行的数据目录撞。应用里直接用 `$XDG_DATA_HOME/app.bedrock`。
- 安卓在真机上测（无线 ADB），加了 `just e2e-device <设备>`。
- 加了 `cupertino_icons`（版本跟 Flutter 模板走，1.x）：Flutter 自带的 Cupertino 控件引用了这套图标，不带的话每次构建都报缺字体。

B5 定下的（2026-10-24）：

- 提交钩子用 lefthook，`just deps` 顺带装上。提交前并行查暂存文件：Rust 和 Dart 的格式、拼写、gitleaks 查密钥；写提交说明时跑 `just check-commit-msg`。钩子只查不改，被拦下自己改好再提交。
- 提交说明：第一行 `类型: 说明`，类型从 feat、fix、docs、refactor、test、build、ci、perf、chore、revert 里选，不带范围；第一行只用英文字符、最长 72 个字符，第二行空着；不许带 AI 署名行。合并、fixup 这类 git 自己生成的说明放过。
- `rustfmt.toml` 写明 `edition`：钩子直接调 rustfmt，不经 cargo，读不到工作区的版次。
- CI（`ci.yml`）每次推送在 Linux 上跑 `xvfb-run just ci`，工作流里只装环境，检查一律走 just。工具版本只写一处：just 的版本写在工作流里，其余从 justfile 读（`just ci-tools`、`fvm` 变量）。第三方 Actions 钉到提交哈希，后面注明版本。
- 发版（`release.yml`）：打 `v` 开头的标签时在 Linux、macOS、Windows 上构建应用和命令行，在 Linux 上构建安卓包，用 `CHANGELOG.md` 里这个版本的一节发 Release；手动运行只构建不发。打包写在 `just package` 里，工作流不另写脚本。
- 安卓签名：`android/key.properties` 在就用正式密钥库，不在就用调试密钥；CI 从仓库的四个密钥写出这个文件。ffi crate 的 `rust-toolchain.toml` 补上 macOS 两种 CPU 和 Windows。
- 工作流改完用 actionlint 查一遍。

B6 定下的（2026-10-24）：

- `just new` 写在 `skill/new.sh`，命令在 `skill/bedrock.just`，根 `justfile` 末尾用 `import?` 引入；新项目里去掉这两行，也不带 `skill/`。它按 bedrock 工作区里的样子复制（包括没提交的改动，AGENTS.md 里注明），不提交、不装 cargo 工具。**坑**：名字长短变了，rustfmt 的换行和 `use` 的顺序、Dart 生成文件的换行都会跟着变，原样替换过不了格式检查，所以换完名字要装 Flutter、拉依赖、`just fmt`。
- 新项目的 `README.md`、`AGENTS.md`、`CLAUDE.md`、`CHANGELOG.md`、`docs/PLAN.md` 从模板写；`docs/ARCHITECTURE.md`、`docs/development.md`、`docs/release.md` 和各 crate 的 `README.md` 用骨架的，换好名字。AGENTS 模板改成和骨架一样的命令表，开头记着 bedrock 的提交和那个提交的规范网址。
- 版本号从 0.1.0 起：bedrock 自己的版本不是 0.1.0 时，`Cargo.toml`、`pubspec.yaml`、操作清单、生成的 Dart 和 `Cargo.lock` 里工作区自己的 crate 一起改回去。
- 技能分成入口 `skill/SKILL.md` 和 `skill/references/` 下五篇：建项目、加业务（含删备忘）、加操作、加端（含加平台、去掉应用）、跟上新规矩。技能只写步骤和去哪看，规矩留在规范里。`just check-skill` 查技能里提到的路径都在（`<项目>` 按 bedrock、`<业务>` 按 note 找）。
- bedrock 多一个工作流 `bedrock.yml`：查技能，用 `just new` 建一个项目，在里面跑 `just ci`。

## 开发步骤

每步做完都要 `just ci` 通过，停下来确认后再做下一步。

| 步骤 | 内容 | 验收 |
| --- | --- | --- |
| B1 规范文档（2026-10-24 完成：9 篇规范、9 个模板，以及本仓库的 README、AGENTS；还没有代码，`just ci` 从 B2 起才有） | `docs/conventions/` 下按主题写规矩：目录、分层、操作面、类型生成、文档、检查、错误和测试、前端、带界面的端；README、AGENTS 的模板 | 每条规矩写清楚做什么、为什么，能直接照做 |
| B2 Rust 骨架（2026-10-24 完成：核心、SQLite 存储、本机组装、命令行、服务端、原生接口出口，27 个测试；`just ci` 通过；故意让核心依赖 sqlx、在核心里读时间或打印，`just check` 都会失败；改了类型没 `just gen` 也会失败） | 工作区配置（版次、统一 lint、依赖、工具链、rustfmt、clippy、deny、typos）；`crates/bedrock-core`（备忘业务、操作登记、按名字调用、导出 Schema）；`crates/bedrock-store-sqlite`；`crates/bedrock-ffi`（UniFFI 出口）；`apps/cli`；`apps/server`（操作面走 HTTP，托管网页）；`justfile`；core 禁用依赖的检查 | `just check`、`just test` 通过；命令行和服务端都能新建、列出、删除备忘；故意让 core 依赖 sqlx 时 `just check` 失败 |
| B3 调整 Rust 骨架，实测 Flutter 桌面（2026-10-24 完成：`bedrock serve` 并进命令行，`apps/server` 和 UniFFI 出口删掉，26 个测试，`just ci` 通过；Flutter 3.47.8 的最小应用在 Linux 桌面上试过中文输入、窗口、托盘） | 服务并进命令行：HTTP 路由挪到 `crates/bedrock-http`，命令行加 `bedrock serve`，去掉 `apps/server` 和托管网页；删掉写了一半的前端（pnpm 工作区、`packages/`、`apps/web`）；规范里网页、TS、Tauri 的部分改掉。装 Flutter（版本固定在仓库里），在本机 Linux 桌面跑一个最小应用，实测中文输入法、窗口、托盘 | `just ci` 通过；`bedrock serve` 能增删查备忘；在 Linux 桌面上试过 Flutter 应用里的中文输入，没问题 |
| B4 Flutter 应用（2026-10-24 完成：逐题定了第 14–18 题；`apps/app` 在 Linux 桌面和安卓真机（无线 ADB）上都能新建、列出、删除备忘，安卓正式包只编 arm64-v8a 时 23.5 MB；`just ci` 通过，Rust 27 个测试、Dart 9 个单元测试、桌面集成测试 2 条，手机上集成测试通过；改了 Rust 类型不 `just gen`，清单和 Dart 两段检查都会失败；iOS、macOS、Windows 只生成了工程，还没构建） | 先逐题定状态管理、路由、组件库、Dart 类型生成、lint 规则集。`apps/app`：桌面（Linux 本机构建）、安卓（本机构建 APK）、iOS 工程（在 macOS 上构建，先只生成工程）；`crates/bedrock-ffi` 换成 flutter_rust_bridge（`open` 加 `call`）；Dart 类型从 `schema/ops.json` 生成；两种传输：本机直接调核心、经网络连 `bedrock serve` | 桌面和安卓都能增删查备忘；改了 Rust 类型不重新生成时检查失败；集成测试通过 |
| B5 提交钩子和 CI（2026-10-24 完成：钩子在临时副本里试过，正常的提交能过，中文第一行、假的 GitHub 令牌、没排版的 Rust 文件都被拦下；两个工作流过了 actionlint；推到 GitHub 后 CI 通过，一次约 10 分钟，Rust 27 个测试和桌面集成测试 2 条都跑了；发版工作流还没实际跑过） | lefthook、提交说明检查、gitleaks；GitHub Actions 跑 `just ci`，发版时打包三个桌面平台和安卓 | 推送后 CI 通过 |
| B6 全局技能（2026-10-24 完成：逐题定了第 19–22 题；写了 `just new`、技能和 `bedrock.yml`；本机 `just new demo` 建出的项目从头跑 `just ci` 通过，Rust 27 个测试、Dart 9 个、桌面集成测试 2 条） | `~/.claude/skills/bedrock/`：建项目、加业务、加操作、加端时怎么做；全局 CLAUDE.md 加一行指向它 | 新会话里让代理建一个项目，按规矩建出来（2026-10-24 改：用户定了不在新会话里验收，以 `just new` 建出的项目在本机和 CI 里都过 `just ci` 为准） |
| B7 迁移第一个项目 | 按第 12 题重组，同时做 Flutter 客户端，删掉网页端 | 原有的 Rust 测试全部通过；手机上的 Flutter 应用能做到网页端现在能做的事 |
