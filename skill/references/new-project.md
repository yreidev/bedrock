# 建一个新项目

1. 跟用户确认三件事，一次问一题：
   - 项目名：一个小写单词，小写字母开头，只有字母和数字。它会成为命令行的程序名、crate 名的前缀（`<项目>-core`）、Dart 包名、安卓的 applicationId（`app.<项目>`）和环境变量的前缀（`<项目大写>_DATA`）。
   - 放在哪个目录。
   - 一句话说清它做什么、给谁用。
2. 在 bedrock 里建：

   ```sh
   cd "$BEDROCK" && just new <项目> <目录>
   ```

   `just new` 复制骨架（按 bedrock 工作区里的样子，包括没提交的改动），把 `bedrock`、`Bedrock`、`BEDROCK` 换成新名字，版本号从 0.1.0 起；`README.md`、`AGENTS.md`、`CLAUDE.md`、`CHANGELOG.md`、`docs/PLAN.md` 从 `docs/conventions/templates/` 写；`AGENTS.md` 里记下建自 bedrock 的哪个提交；换完名字按 `.fvmrc` 装 Flutter、拉应用的依赖、`just fmt` 重新排版（名字长短变了，换行和 `use` 的顺序会跟着变）；最后 `git init`。规范、技能不带过去。它不提交，也不装 cargo 工具和提交钩子。
3. 在新项目里：

   ```sh
   just deps   # 按固定版本装 cargo 工具和 Flutter，拉依赖，装提交钩子
   just ci     # 检查、测试，最后在桌面上弹出应用跑集成测试
   ```

   这时示例业务“备忘”和所有端都在，`just ci` 必须通过；不过就是骨架或 `just new` 有问题，回 bedrock 修。
4. 填文档里的空位：`README.md` 的一句话、亮点、不适合的情况；`docs/PLAN.md` 开头的一句话；`docs/ARCHITECTURE.md` 的“要解决的问题”。别的文档（`docs/development.md`、`docs/release.md`、各 crate 的 `README.md`）已经按骨架写好，加业务时照实改。
5. 停下汇报。用户说提交再提交，首个提交写 `chore: scaffold from bedrock <提交前七位>`。要建 GitHub 仓库时先问用户：哪个账号、公开还是私有。`Cargo.toml` 里的 `repository` 默认指向和 bedrock 同一个账号下的同名仓库，不对就改。
6. 接着按 `references/add-business.md` 加第一块业务，写完删掉备忘。只要命令行、不要应用的项目，按 `references/add-end.md` 去掉应用。
