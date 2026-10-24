import 'dart:convert';

import 'package:bedrock/ffi/api.dart';
import 'package:bedrock/ffi/frb_generated.dart';
import 'package:bedrock/transport/transport.dart';

/// 核心直接跑在应用进程里，经 Rust 的接口出口调用。
final class LocalTransport implements Transport {
  LocalTransport._(this._core);

  final Bedrock _core;

  static Future<void>? _loaded;

  /// 打开 [dataDir] 里的数据，目录不在就建。
  static Future<LocalTransport> open(String dataDir) async {
    // 原生库一个进程只加载一次
    await (_loaded ??= RustLib.init());
    try {
      return LocalTransport._(await Bedrock.open(dataDir: dataDir));
    } on CallError catch (err) {
      throw OpException(err.code, err.message);
    }
  }

  @override
  Future<Map<String, Object?>> call(
    String name,
    Map<String, Object?> input,
  ) async {
    final String output;
    try {
      output = await _core.call(name: name, inputJson: jsonEncode(input));
    } on CallError catch (err) {
      throw OpException(err.code, err.message);
    }
    return jsonDecode(output) as Map<String, Object?>;
  }
}
