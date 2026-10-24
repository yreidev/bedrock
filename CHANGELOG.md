# 更新记录

## 未发布

### 新增

- 规范：目录结构、核心分层、操作面、类型生成、错误和测试、各端、界面、检查、文档，以及各文档的模板。
- Rust 骨架：核心（示例业务“备忘”的新建、列出、删除，按名字调用，导出操作清单）、SQLite 存储、本机组装、命令行，`bedrock serve` 把操作面开成 HTTP。
- Flutter 应用：桌面和手机共用，能新建、列出、删除备忘；本机经 flutter_rust_bridge 跑核心，也能连 `bedrock serve`。
- 从操作清单生成 Dart 的类型、错误码、操作表和 Riverpod provider。
- `just` 命令：`deps`、`gen`、`fmt`、`check`、`test`、`e2e`、`e2e-device`、`ci`、`dev`、`app`、`build`、`build-app`；核心的禁用依赖和禁用函数由 `just check` 查。
- 提交钩子：暂存文件的格式、拼写、密钥，提交说明的格式。
- GitHub Actions：每次推送跑 `just ci`；打标签时构建三个桌面系统的应用和命令行、安卓包，发到 GitHub Releases。
- 许可证：MIT 或 Apache-2.0，任选其一。
- `just new <名字> <目录>`：用骨架建新项目，换好名字，顶层文档从模板写，记下建自 bedrock 的哪个提交。
- 全局技能 `bedrock`（`skill/`）：建项目、加业务、加操作、加端或去掉应用、跟上新规矩时照着做。
