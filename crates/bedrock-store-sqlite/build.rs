fn main() {
    // 迁移文件编进程序里，改了要重新编译。
    // 目录要带结尾的斜杠：Flutter 的原生构建钩子读 cargo 的依赖文件时，不带斜杠的路径会被
    // 当成文件，“文件不存在”就算刚改过，每次构建都会重新跑一遍钩子。
    println!("cargo:rerun-if-changed=migrations/");
}
