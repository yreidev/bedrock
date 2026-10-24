#!/usr/bin/env bash
# 用 bedrock 的骨架建一个新项目：just new <名字> <目录>
#
# 复制骨架，把 bedrock、Bedrock、BEDROCK 换成新名字的三种写法，顶层文档从模板写，
# 在 AGENTS.md 记下建自 bedrock 的哪个提交，重新排版。只建目录和 git 仓库，不提交、不装 cargo 工具。
set -euo pipefail

usage() {
    echo "用法：just new <名字> <目录>" >&2
    exit 2
}

[[ $# -eq 2 ]] || usage
name=$1
dest=$2

if [[ ! $name =~ ^[a-z][a-z0-9]*$ ]]; then
    echo "项目名只能是一个小写单词：小写字母开头，只有字母和数字（比如 notes）" >&2
    exit 2
fi
if [[ $name == bedrock ]]; then
    echo "新项目不能也叫 bedrock" >&2
    exit 2
fi
src=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
dest=$(realpath -m "$dest")
case $dest/ in
    "$src"/*)
        echo "新项目不能放在 bedrock 仓库里面" >&2
        exit 2
        ;;
esac
if [[ -e $dest && -n $(ls -A "$dest") ]]; then
    echo "$dest 已经存在而且不是空的" >&2
    exit 2
fi

Name=${name^}
NAME=${name^^}
commit=$(git -C "$src" rev-parse HEAD)
dirty=
if [[ -n $(git -C "$src" status --porcelain) ]]; then
    dirty=yes
fi

mkdir -p "$dest"

# 只有 bedrock 自己有的：规范、技能、检查技能和 just new 的工作流；顶层文档后面从模板写
own='^(docs/conventions/|skill/|\.github/workflows/bedrock\.yml$|README\.md$|AGENTS\.md$|CLAUDE\.md$|CHANGELOG\.md$|docs/PLAN\.md$)'

# 按工作区里的样子复制（包括还没提交的改动），路径里的名字一起换
while IFS= read -r -d '' path; do
    [[ -f $src/$path ]] || continue
    [[ $path =~ $own ]] && continue
    to=$dest/${path//bedrock/$name}
    mkdir -p "$(dirname "$to")"
    cp -p "$src/$path" "$to"
done < <(git -C "$src" ls-files -z --cached --others --exclude-standard)

cd "$dest"

# justfile 末尾引入 bedrock 自己命令的那两行，新项目里没有
sed -i '/^# 只有 bedrock 仓库自己有的命令/,/^import? .skill\/bedrock\.just.$/d' justfile

# 换名字：只动文本文件
while IFS= read -r -d '' file; do
    if grep -Iq . "$file" && grep -q -e bedrock -e Bedrock -e BEDROCK "$file"; then
        sed -i -e "s/bedrock/$name/g" -e "s/Bedrock/$Name/g" -e "s/BEDROCK/$NAME/g" "$file"
    fi
done < <(find . -type f -print0)

# 版本号从 0.1.0 起
old=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
if [[ $old != 0.1.0 ]]; then
    sed -i "0,/^version = \"$old\"$/s//version = \"0.1.0\"/" Cargo.toml
    sed -i 's/^version: .*/version: 0.1.0+1/' apps/app/pubspec.yaml
    sed -i "s/^  \"version\": \"$old\",$/  \"version\": \"0.1.0\",/" schema/ops.json
    sed -i -e "s/（版本 $old）/（版本 0.1.0）/" -e "s/^const opsVersion = '$old';$/const opsVersion = '0.1.0';/" apps/app/lib/gen/ops.dart
    # Cargo.lock 里工作区自己的 crate 跟着改
    awk -v n="$name" -v old="$old" '
        /^name = / { mine = index($0, "name = \"" n "-") == 1 }
        mine && $0 == "version = \"" old "\"" { $0 = "version = \"0.1.0\""; mine = 0 }
        { print }
    ' Cargo.lock >Cargo.lock.new
    mv Cargo.lock.new Cargo.lock
fi

# 顶层文档从模板写，{{项目}} 换成名字，{{bedrock 提交}} 换成建自的提交
note=
if [[ -n $dirty ]]; then
    note="（建的时候 bedrock 还有没提交的改动）"
fi
templates=$src/docs/conventions/templates
fill() {
    sed -e "s/{{项目}}/$name/g" -e "s/{{bedrock 提交}}/$commit/g" -e "s/{{bedrock 提交备注}}/$note/g" "$templates/$1" >"$2"
}
fill README.md README.md
fill AGENTS.md AGENTS.md
fill CLAUDE.md CLAUDE.md
fill CHANGELOG.md CHANGELOG.md
fill PLAN.md docs/PLAN.md

# 名字的长短变了，rustfmt 的换行和 use 的顺序也跟着变：换完重新排版。
# Dart 按应用自己的语言版本排版，要先装好 Flutter、拉好依赖
if command -v fvm >/dev/null; then
    just deps-app >/dev/null
    just fmt >/dev/null
else
    cargo fmt --all
    echo "没找到 fvm：装好以后先 just deps 再 just fmt，不然 Dart 的排版检查过不了" >&2
fi

git init -q -b main

echo "建好了：$dest，建自 bedrock ${commit:0:7}${dirty:+，当时 bedrock 还有没提交的改动}"
echo "接下来："
echo "  cd $dest"
echo "  just deps   # 装工具、拉依赖、装提交钩子"
echo "  just ci     # 会在桌面上弹出应用跑集成测试"
echo "再填 README 和 docs/PLAN.md 里的空位。"
