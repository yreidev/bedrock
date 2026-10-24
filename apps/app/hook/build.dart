// Flutter 构建时调它：用 rustup 按 crates/bedrock-ffi/rust-toolchain.toml 编出 Rust 核心，打进应用。
import 'package:flutter_rust_bridge_hooks/flutter_rust_bridge_hooks.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    await const FlutterRustBridgeNativeAssetsBuilder(
      cratePath: '../../crates/bedrock-ffi',
    ).run(input: input, output: output);
  });
}
