# 所有命令的入口。`just` 列出全部。
#
# Flutter 的命令都经 fvm，版本按仓库根目录的 .fvmrc。

set shell := ["bash", "-euo", "pipefail", "-c"]

# cargo 工具的版本，升级就改这里再跑 just deps
cargo_deny := "0.20.2"
typos := "1.51.1"
cargo_machete := "0.9.2"
# 要和 crates/bedrock-ffi、apps/app 里 flutter_rust_bridge 的版本一样
frb := "2.13.0"
# CI 按这个版本装 fvm；本机用 dart pub global activate fvm 装
fvm := "4.3.2"

core := "crates/bedrock-core"
app := "apps/app"
dart_dirs := "lib test integration_test tool hook"

# 列出所有命令
default:
    @just --list --unsorted

# 按固定版本装 cargo 工具和 Flutter，拉应用的依赖，装上提交钩子
deps: deps-app
    cargo install --locked cargo-deny@{{ cargo_deny }} typos-cli@{{ typos }} cargo-machete@{{ cargo_machete }} flutter_rust_bridge_codegen@{{ frb }}
    lefthook install

# 按 .fvmrc 装 Flutter，拉应用的依赖
deps-app:
    fvm install
    cd {{ app }} && fvm flutter pub get

# CI 要装的检查工具和版本，给 taiki-e/install-action 用
[private]
ci-tools:
    @echo "cargo-deny@{{ cargo_deny }},typos@{{ typos }},cargo-machete@{{ cargo_machete }}"

# 导出操作清单，生成 Dart 类型和给 Dart 的接口
gen:
    cargo run -q -p bedrock-cli -- ops --json > schema/ops.json
    cd {{ app }} && fvm dart run tool/gen_ops.dart ../../schema/ops.json lib/gen/ops.dart && fvm dart format lib/gen/ops.dart
    # 桌面会话里的 RUST_LOG 常带模块名，flutter_rust_bridge_codegen 认不得会直接退出
    cd {{ app }} && env -u RUST_LOG flutter_rust_bridge_codegen generate

# 格式化全部代码
fmt:
    cargo fmt --all
    cd {{ app }} && fvm dart format {{ dart_dirs }}

# 只检查、不改文件
check:
    cargo fmt --all --check
    typos
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    cargo deny --all-features --locked check --hide-inclusion-graph
    cargo deny --locked --manifest-path {{ core }}/Cargo.toml --config {{ core }}/deny.toml check --hide-inclusion-graph bans
    cargo machete
    cd {{ app }} && fvm dart format --output=none --set-exit-if-changed {{ dart_dirs }}
    cd {{ app }} && fvm flutter analyze
    @just check-gen

# 生成的文件要和 Rust 类型对得上。给 Dart 的接口过期时，应用启动就会报内容不一致，集成测试会失败
[private]
check-gen:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo run -q --locked -p bedrock-cli -- ops --json | diff -u schema/ops.json -; then
        echo "schema/ops.json 过期了：运行 just gen 再提交" >&2
        exit 1
    fi
    # 放在应用自己的 .dart_tool 里，dart format 才按这个包的语言版本排版
    out=$(mktemp -p "{{ justfile_directory() }}/{{ app }}/.dart_tool" --suffix=.dart)
    trap 'rm -f "$out"' EXIT
    (cd {{ app }} && fvm dart run tool/gen_ops.dart ../../schema/ops.json "$out" && fvm dart format "$out" >/dev/null)
    if ! diff -u {{ app }}/lib/gen/ops.dart "$out"; then
        echo "{{ app }}/lib/gen/ops.dart 过期了：运行 just gen 再提交" >&2
        exit 1
    fi

# 全部单元测试：Rust，加上 Flutter 应用里不起界面的
test:
    cargo test --workspace --all-features --locked
    cd {{ app }} && fvm flutter test

# 端到端：在 Linux 桌面上起应用，本机跑核心和连 bedrock serve 各走一遍
e2e:
    cargo build --locked -p bedrock-cli
    cd {{ app }} && fvm flutter test integration_test -d linux --dart-define=BEDROCK_CLI={{ justfile_directory() }}/target/debug/bedrock

# 在连着的手机上跑集成测试，只跑本机跑核心那条。设备名看 fvm flutter devices
e2e-device device:
    cd {{ app }} && fvm flutter test integration_test -d {{ device }}

# 和 CI 完全一样
ci: check test e2e

# 开发模式跑命令行：just dev note list、just dev serve
[positional-arguments]
dev *args:
    cargo run -p bedrock-cli -- "$@"

# 开发模式在本机桌面跑 Flutter 应用，后面的参数交给 flutter run
[positional-arguments]
app *args:
    cd {{ app }} && fvm flutter run -d linux "$@"

# 构建命令行的发行版
build:
    cargo build --release --locked -p bedrock-cli

# 构建 Flutter 应用的发行版：linux、macos、windows、apk
build-app target:
    cd {{ app }} && fvm flutter build {{ target }} --release

# 把 build 和 build-app 的产物打包到 dist/，发版的 CI 用。目标同 build-app，apk 不带命令行
[private]
package target version:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p dist
    name=bedrock-{{ version }}
    out={{ app }}/build
    case {{ target }} in
        linux)
            tar -czf dist/$name-linux-x64.tar.gz -C $out/linux/x64/release/bundle .
            tar -czf dist/$name-cli-linux-x64.tar.gz -C target/release bedrock ;;
        macos)
            ditto -c -k --keepParent $out/macos/Build/Products/Release/bedrock.app dist/$name-macos.zip
            tar -czf dist/$name-cli-macos-$(uname -m).tar.gz -C target/release bedrock ;;
        windows)
            (cd $out/windows/x64/runner/Release && 7z a -tzip "$OLDPWD/dist/$name-windows-x64.zip" . >/dev/null)
            (cd target/release && 7z a -tzip "$OLDPWD/dist/$name-cli-windows-x64.zip" bedrock.exe >/dev/null) ;;
        apk)
            cp $out/app/outputs/flutter-apk/app-release.apk dist/$name-android.apk ;;
        *)
            echo "不认识的目标：{{ target }}" >&2; exit 2 ;;
    esac
    ls -l dist

# 从 CHANGELOG.md 取出某个版本的说明，发 GitHub Release 时用
[private]
release-notes version:
    @awk -v v="{{ version }}" '/^## / { on = index($0, "## " v) == 1; next } on' CHANGELOG.md

# 检查提交说明，提交钩子调它：首行 `类型: 说明`、英文、不超过 72 个字符，不带 AI 工具的署名
[private]
check-commit-msg file:
    #!/usr/bin/env bash
    set -euo pipefail
    # 钩子拿到的说明还带着 git 的注释行；剪刀线以下是 git commit -v 附上的差异
    msg=$(sed '/^# -\{24\} >8 -\{24\}$/,$d' "{{ file }}" | grep -v '^#' || true)
    first=$(printf '%s\n' "$msg" | sed -n 1p)
    fail() { printf '提交说明：%s\n  首行：%s\n' "$1" "$first" >&2; exit 1; }
    case $first in
        "Merge "* | "fixup! "* | "squash! "* | "amend! "*) exit 0 ;;
    esac
    types='feat|fix|docs|refactor|test|build|ci|perf|chore|revert'
    if printf '%s' "$first" | LC_ALL=C grep -q '[^ -~]'; then fail "首行用英文写"; fi
    [[ $first =~ ^($types):\ [^[:space:]] ]] || fail "首行写成“类型: 说明”，类型是 ${types//|/、} 之一"
    (( ${#first} <= 72 )) || fail "首行 ${#first} 个字符，不能超过 72 个"
    [[ -z $(printf '%s\n' "$msg" | sed -n 2p) ]] || fail "首行和正文之间空一行"
    if printf '%s\n' "$msg" | grep -qiE '^co-authored-by:.*(claude|anthropic|openai|codex|copilot|cursor|gemini)|generated with .*(claude|codex|copilot|cursor)|🤖'; then
        fail "不带 AI 工具的署名行"
    fi

# 只有 bedrock 仓库自己有的命令：just new、just check-skill。建新项目时连这两行一起去掉
import? 'skill/bedrock.just'
