---
name: bedrock
description: 按 bedrock 的规矩建新项目、加业务、加操作、加端或去掉端，以及让已有项目跟上 bedrock 新改的规矩。用户要新建一个 Rust 写核心的项目（命令行、桌面或手机应用），或者在 AGENTS.md 写着“按 bedrock 的规范写”的项目里加功能、改结构时用。
---

# bedrock

bedrock 是规范加骨架：`docs/conventions/` 是规矩，仓库根目录是按规矩写的参考实现，示例业务是“备忘”（`note`）。这个技能目录是 bedrock 仓库里的 `skill/`，仓库在：

```sh
BEDROCK=$(dirname "$(readlink -f ~/.claude/skills/bedrock)")
```

下面写到的 `docs/conventions/…` 都在 `$BEDROCK` 里；`crates/<项目>-core/…` 这类路径在项目里，对应到骨架就是把 `<项目>` 换成 `bedrock`。

## 先找对任务

| 要做的事 | 照着做 |
| --- | --- |
| 建一个新项目 | `references/new-project.md` |
| 加一块业务，或者删掉示例业务“备忘” | `references/add-business.md` |
| 给已有的业务加一个操作 | `references/add-operation.md` |
| 加一个端、给应用加一个平台、去掉应用只留命令行 | `references/add-end.md` |
| bedrock 的规矩改了，项目跟着改 | `references/align.md` |

动手前读要动的那几篇规范（`docs/conventions/README.md` 有目录），再看骨架里对应的代码怎么写，照着写。规范和骨架对不上时以骨架为准，回头告诉用户要改 bedrock 的文字。

## 必须守住的

- 业务规则只写在核心里。核心不碰外部世界（文件、网络、时间、环境变量），要用就写成端口，由适配器实现。依赖只能是端 → 适配器 → 核心。
- 各端只经操作面调核心。加了操作要登记、`just gen`，生成的文件一起提交。
- 每做完一步 `just ci` 通过再汇报；提交、建 GitHub 仓库、推送都等用户说。
- 项目里发现规矩不合适：先在项目里按实际情况做，记进项目的 `docs/PLAN.md`，汇报时提出来，用户同意后再回 bedrock 改规矩（规矩和骨架一起改，单独提交）。
- 项目名只许一个小写单词：小写字母开头，只有字母和数字。
