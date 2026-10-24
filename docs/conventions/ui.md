# 界面

桌面（Windows、macOS、Linux）和手机（安卓、iOS）是同一个 Flutter 应用 `apps/app`，界面只写一份。

## 技术栈

- Flutter 稳定版，版本固定在仓库根目录的 `.fvmrc`，用 fvm 装对应的版本，命令写成 `fvm flutter …`。升级 Flutter 单独提交。
- 状态管理用 Riverpod 3（`flutter_riverpod`），provider 直接手写；查询和动作的 provider 由 `just gen` 从操作清单生成。
- 路由用 go_router，所有路由写在 `lib/app.dart` 的一个 `GoRouter` 里。页面按路径打开，以后的深链接、桌面多窗口都靠路径。
- 组件用 shadcn_ui，主题是 zinc 的浅色和深色，跟随系统。它还在 0.x，小版本也可能改接口，所以版本写死（`shadcn_ui: 0.57.3`，不带 `^`），升级单独提交。
- 图标用 shadcn_ui 带的 Lucide（`LucideIcons`）。

## 取数据

- 界面里不直接碰传输（本机调用或 HTTP）。两种传输都实现 `lib/transport/transport.dart` 的 `Transport`，启动时在 `main.dart` 选一种，经 `opsProvider.overrideWithValue(Ops(transport))` 注入：同一套界面既能在本机跑核心，也能连别的设备上的 `<项目> serve`。测试里注入别的传输。
- 每个查询操作生成一个 `<名字>Query`（`note.list` 是 `noteListQuery`），按输入缓存，页面上 `ref.watch(noteListQuery(const ListNotesInput()))`，按 `AsyncValue` 的三种状态分别显示。没有页面在看时自动释放。
- 动作经 `ref.read(opActionsProvider).noteCreate(…)` 调。成功后它按业务前缀让相关查询失效（`note.create` 成功后刷新所有 `note.` 开头的查询），页面里不手动刷新。
- 查询失败时只重试可能自己好起来的错误：`unreachable`（连不上）和 `internal`，最多三次，在 `ProviderScope` 的 `retry` 里统一设。动作失败后不自动重试。
- 失败一律是 `OpException`，带 `code` 和 `message`。按 `code` 判断，界面上显示 `message`；`internal` 统一显示“出了点问题”，不展示技术细节（`lib/errors.dart` 的 `describeError`）。
- 每个操作可能报的错误码生成成常量（`NoteCreateErrors.emptyTitle`，名字按操作名 `note.create` 来），和 `code` 比较时用常量，不手写字符串。

## 界面

- 先按手机宽度（390 逻辑像素）设计，再放宽到桌面。任何宽度都不能横向滚动整页。
- 浅色和深色都要好看，颜色从主题里取，不在控件里写死色值。
- 界面文字一律中文，按钮写动作（“新建备忘”），不写“确定”“提交”。
- 可点的东西有中文的语义标签，读屏和集成测试都按它找控件。有文字的按钮用按钮文字；只有图标的按钮包一层 `Semantics(container: true, button: true, label: '删除“…”', excludeSemantics: true, child: …)`。
- 字体、图标都打进包里，不从外网加载：桌面和手机离线也要能打开。图标统一从一个模块取。
- 桌面和手机的差别按平台判断分开（托盘、多窗口只在桌面，后台常驻、推送只在手机），不复制两份应用。只在一种平台出现的代码放在单独的文件里，别在页面里到处写平台判断。

## 代码风格

- `dart format` 的默认格式，`fvm flutter analyze` 零警告。
- lint 用 flutter_lints，另外打开：
  - `strict-casts`、`strict-inference`、`strict-raw-types`：类型推不出来、隐式转换都报错。
  - `unawaited_futures`：忘了 `await` 的异步调用报错，故意不等的写 `unawaited(…)`。
  - `avoid_dynamic_calls`：不在 `dynamic` 上调方法，JSON 先转成具体类型再用。
- flutter_rust_bridge 生成的 `lib/ffi/` 不查。`lib/gen/` 照样查，生成器写出的代码也要过 lint。
- 纯逻辑（格式化、解析、筛选）写成不依赖 Flutter 的 Dart 函数，旁边放单元测试。
- 每个业务一个目录（`lib/notes/`），页面和它的纯逻辑放在一起。

## 测试

- `test/` 放不起界面的单元测试：纯逻辑、生成的类型、HTTP 传输（用 `http` 包的 `MockClient`），`just test` 跑。
- `integration_test/` 在真窗口里跑整个应用，`just e2e` 在 Linux 桌面上跑：一遍在本机跑核心，一遍起 `<项目> serve` 再连过去，走同一套步骤。
- 集成测试按文字和语义标签找控件、点、输入，不按控件类型或内部的 key 找，测的就是用户看到的样子。
- **坑**：按语义标签找控件前要 `tester.ensureSemantics()` 打开语义树，并且在测试函数结束前关掉（`try`/`finally` 里 `dispose()`）。用 `addTearDown` 关就太晚了，测试会报还有没关的语义句柄。
