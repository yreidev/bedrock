# {{项目}}

<!-- 每个会话都会读这份文件，保持短而准。只写命令、质量关卡和要避开的坑；目录和文件是什么，读代码就知道，不写。 -->

按 bedrock 的规范写，建自 bedrock 提交 `{{bedrock 提交}}`{{bedrock 提交备注}}。规范全文在全局技能 `bedrock` 里，网上看 https://github.com/yreidev/bedrock/tree/{{bedrock 提交}}/docs/conventions 。bedrock 的规矩改了、项目跟着改过以后，把这里的提交换成新的。项目自己的决定在 `docs/PLAN.md`，边界和不变量在 `docs/ARCHITECTURE.md`。

## 命令

| 命令 | 做什么 |
| --- | --- |
| `just check` | 只检查不改文件 |
| `just test` | 不起界面的测试：Rust 全部，加上应用的单元测试 |
| `just e2e` | 在 Linux 桌面上起应用跑集成测试（会弹窗） |
| `just gen` | 改了操作的输入输出、或 `crates/{{项目}}-ffi` 导出的函数后，重新导出清单、生成 Dart 代码 |
| `just dev [参数]` | 开发模式跑命令行：`just dev note list`、`just dev serve` |
| `just app [参数]` | 开发模式在本机桌面跑应用 |

Flutter 的命令一律经 fvm（`fvm flutter …`、`fvm dart …`）。

## 质量关卡

提交前 `just ci` 必须通过。改了操作的输入输出要 `just gen` 并提交生成的文件。

提交钩子（lefthook）查暂存文件的格式、拼写、密钥和提交说明，被拦下就改好再提交，别用 `--no-verify` 跳过钩子。

## 要避开的坑

<!-- 每条写成：什么情况下、会出什么错、该怎么做。新加一条要满足：不明显、反复遇到、具体到能照做。 -->
