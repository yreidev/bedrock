# bedrock

这个仓库是规范加骨架：`docs/conventions/` 是规矩，仓库根目录是按规矩写的参考实现（示例业务“备忘”），`skill/` 是全局技能和建新项目的脚本。决定和进度在 `docs/PLAN.md`。

## 命令

| 命令 | 做什么 |
| --- | --- |
| `just check` | 只检查不改文件 |
| `just test` | 不起界面的测试：Rust 全部，加上应用的单元测试 |
| `just e2e` | 在 Linux 桌面上起应用跑集成测试（会弹窗） |
| `just gen` | 改了操作的输入输出、或 `crates/bedrock-ffi` 导出的函数后，重新导出清单、生成 Dart 代码 |
| `just dev [参数]` | 开发模式跑命令行：`just dev note list`、`just dev serve` |
| `just app [参数]` | 开发模式在本机桌面跑应用 |
| `just new <名字> <目录>` | 用骨架建一个新项目 |
| `just check-skill` | 查技能里提到的文件都还在 |

Flutter 的命令一律经 fvm（`fvm flutter …`、`fvm dart …`），fvm 在 `~/.pub-cache/bin`。

## 质量关卡

提交前 `just ci` 必须通过；改了 `skill/` 再跑 `just check-skill`。改了操作的输入输出要 `just gen` 并提交生成的文件。推送后 GitHub 上还有一个工作流用 `just new` 建项目再跑 `just ci`，它挂了说明骨架改坏了建新项目。

提交钩子（lefthook）查暂存文件的格式、拼写、密钥和提交说明，被拦下就改好再提交，别用 `--no-verify` 跳过钩子。提交说明的写法见 `docs/conventions/checks.md`。

## 要避开的坑

- 规矩和骨架必须一起改：改了 `docs/conventions/` 里的做法，骨架里对应的代码要同时改；反过来也一样。两边对不上时以骨架为准，并回头改文字。
- 新规矩先过“规矩怎么改”那三条（不明显、反复遇到、具体到能照做），一次性的观察不写进规矩。
- 仓库里不写参考了哪些项目、网站，只写结论；不写个人信息（路径、地址、邮箱、用量数字）。
- 示例业务保持小：够演示校验、端口、存储、错误码和各端调用就行，别为了演示加业务功能。
- 骨架里的项目名只用 `bedrock`、`Bedrock`、`BEDROCK` 三种写法（`bedrock_core` 这类下划线写法也算），`just new` 靠替换这三种写法建新项目，别用别的办法拼出项目名。
- 只有 bedrock 自己有的东西（规范、技能、`bedrock.yml` 工作流、顶层的 README、AGENTS、PLAN、CHANGELOG）列在 `skill/new.sh` 的 `own` 里，新加这类文件要一起加进去，不然会被复制进新项目。
- 技能只写怎么做、去哪看，规矩本身留在 `docs/conventions/`，别在技能里再抄一遍。
