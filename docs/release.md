# 发版

打 `v<版本>` 标签后，`.github/workflows/release.yml` 在三个桌面系统上构建应用和命令行、在 Linux 上构建安卓包，发到 GitHub Releases。

## 步骤

1. `just ci` 通过。
2. 把 `CHANGELOG.md` 的“未发布”改成 `<版本> - <日期>`，下面再开一个空的“未发布”。发 Release 时取的就是这一节，没有这一节发版会失败。
3. 改版本号：根 `Cargo.toml` 的 `[workspace.package] version`；`apps/app/pubspec.yaml` 的 `version`，加号后面的构建号也要加一（`0.2.0+2`），安卓靠它判断能不能覆盖安装。然后 `just gen`，操作清单里带着版本号。
4. 提交 `chore: release <版本>`，打标签 `v<版本>`，推送提交和标签。
5. 等发版工作流跑完，在 GitHub Releases 核对产物：

   | 文件 | 是什么 |
   | --- | --- |
   | `bedrock-<版本>-linux-x64.tar.gz` | Linux 桌面应用，解开后运行 `bedrock` |
   | `bedrock-<版本>-macos.zip` | macOS 桌面应用，两种 CPU 通用 |
   | `bedrock-<版本>-windows-x64.zip` | Windows 桌面应用，解开后运行 `bedrock.exe` |
   | `bedrock-<版本>-android.apk` | 安卓包 |
   | `bedrock-<版本>-cli-*` | 各系统的命令行 |

只想试构建、不发版：在 GitHub 的 Actions 页面手动运行“发版”，产物在那次运行的附件里，版本号写成 `dev-<提交>`。macOS 的运行器按十倍计费时长，别频繁试。

## 安卓签名

安卓只认同一个密钥签的新版本，所以正式包要一直用同一个密钥库。在仓库的 Settings → Secrets and variables → Actions 里加四个密钥：

| 名字 | 内容 |
| --- | --- |
| `ANDROID_KEYSTORE_BASE64` | 密钥库文件的 base64：`base64 -w0 release.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | 密钥库密码 |
| `ANDROID_KEY_ALIAS` | 密钥别名 |
| `ANDROID_KEY_PASSWORD` | 密钥密码 |

生成密钥库：`keytool -genkey -v -keystore release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias release`。密钥库和密码自己另存一份，丢了就再也发不了能覆盖安装的新版本。

没配这四个密钥时，工作流用调试密钥签名并打一条警告。本机构建同理：`apps/app/android/key.properties` 在就用正式密钥，不在就用调试密钥。调试密钥签的包和正式包互相不能覆盖安装，要先卸载。

## 没签名的地方

- macOS 应用没有签名和公证，第一次打开要在“系统设置 → 隐私与安全性”里点“仍要打开”。
- Windows 应用没有签名，SmartScreen 会提示，点“更多信息 → 仍要运行”。

## 回滚

- 发错了：在 GitHub Releases 删掉这个 Release，再删标签（`git push --delete origin v<版本>`、`git tag -d v<版本>`）。
- 修好后发一个新的补丁版本，不复用已经发出去的版本号。
- 用户要装回旧版本：从 Releases 下载旧版本的包。安卓要先卸载新版本才能装旧版本，卸载会清掉应用的数据。
