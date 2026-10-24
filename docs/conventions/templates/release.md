# 发版

## 步骤

1. `just ci` 通过。
2. 把 `CHANGELOG.md` 的“未发布”改成版本号和日期。
3. 改版本号（根 `Cargo.toml` 的 `[workspace.package]`、`apps/app/pubspec.yaml` 的 `version`），再 `just gen`（操作清单里带着版本号）。
4. 提交 `chore: release <版本>`，打标签 `v<版本>`，推送。
5. 等 CI 构建完各端，在 GitHub Releases 核对产物。

## 回滚

<!-- 发错了怎么撤：删标签、撤 Release、旧版本怎么装回去。 -->
