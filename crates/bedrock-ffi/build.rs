fn main() {
    // flutter_rust_bridge 的 #[frb] 宏展开后带 cfg(frb_expand)，生成代码时才打开
    println!("cargo::rustc-check-cfg=cfg(frb_expand)");
}
