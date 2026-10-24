# bedrock

以后所有项目的地基：一套规范、一个能编译运行的多端骨架，加一个让代理照着做的全局技能。

- 核心逻辑用 Rust 写，一个仓库放所有端：命令行（也能开服务给别的设备用）、桌面和手机（同一个 Flutter 应用），以后的端照同样的办法接进来。
- 业务规则只写一份：各端经“按名字调用”的操作面使用核心，Dart 和其他语言的类型从 Rust 生成。
- 核心按业务分模块，外部能力走端口，依赖方向和禁用依赖由检查自动把关。
- 骨架本身由 CI 验证，规范和骨架对不上时以骨架为准。

不适合：只有一个端、一下午能写完的小工具，只取规范里需要的部分就够了。

## 现状

规范和骨架已经能用：核心（示例业务“备忘”）、SQLite 存储、命令行（含 `serve`），以及在 Linux 桌面和安卓上能新建、列出、删除备忘的 Flutter 应用。提交钩子和 CI 也已写好。进度见[计划](docs/PLAN.md)。全部搭好后，用 `just new <名字> <目录>` 从它建新项目。

## 试一下

要先装好 rustup、just 和 fvm（见[开发](docs/development.md)的环境）。

```sh
just deps                                   # 装检查用的 cargo 工具和 Flutter
just dev note create 买牛奶 --body 低脂的
just dev note list
just dev serve                              # http://127.0.0.1:7400，Ctrl+C 退出
curl -X POST -H 'Content-Type: application/json' -d '{}' http://127.0.0.1:7400/api/op/note.list
just app                                    # 在本机桌面跑应用
just build-app apk                          # 构建安卓包
just ci                                     # 检查、测试、桌面上的集成测试
```

数据默认放在系统的应用数据目录下，用 `BEDROCK_DATA=<目录>` 换到别处。

## 文档

- [规范](docs/conventions/README.md)：目录结构、核心分层、操作面、类型生成、错误和测试、各端、界面、检查、文档
- [文档模板](docs/conventions/templates/)
- [架构](docs/ARCHITECTURE.md)
- [开发](docs/development.md)：环境、命令、调试
- [发版](docs/release.md)：发版步骤、安卓签名、回滚
- [计划和进度](docs/PLAN.md)

许可证：[MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)，任选其一。
