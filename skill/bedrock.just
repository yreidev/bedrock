# 只有 bedrock 仓库自己有的命令，根目录的 justfile 末尾引入。just new 建新项目时不带这个文件。

# 用骨架建一个新项目，名字只许一个小写单词：just new notes ~/github/notes
[no-cd]
[positional-arguments]
new name dir:
    @bash {{ quote(source_directory() / "new.sh") }} "$@"

# 技能里提到的文件都还在。路径里的 <项目> 按 bedrock、<业务> 按 note 找
check-skill:
    #!/usr/bin/env bash
    set -euo pipefail
    cd {{ quote(justfile_directory()) }}
    missing=0
    while read -r p; do
        q=${p//<项目>/bedrock}
        q=${q//<业务>/note}
        [[ $q == *'<'* || $q == *…* ]] && continue
        [[ $q == references/* ]] && q=skill/$q
        if [[ ! -e $q ]]; then
            echo "技能里提到的 $p 不存在" >&2
            missing=1
        fi
    done < <(grep -ohE '`[^` ]+`' skill/SKILL.md skill/references/*.md | tr -d '`' | grep -E '^(docs|crates|apps|schema|skill|references|\.github)/' | sort -u)
    exit $missing
