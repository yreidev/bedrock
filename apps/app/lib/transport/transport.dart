/// 把一次“按名字调用”送进核心，再把结果拿回来。本机直接调和经网络调都实现它。
abstract interface class Transport {
  /// 调 [name] 这个操作，失败时抛 [OpException]。
  Future<Map<String, Object?>> call(String name, Map<String, Object?> input);
}

/// 操作失败。[code] 给程序判断，[message] 给人看。
final class OpException implements Exception {
  const OpException(this.code, this.message);

  /// 连不上另一台电脑上的服务，不是核心报的错
  static const unreachable = 'unreachable';

  final String code;
  final String message;

  /// 意料之外的错误：界面上统一说“出了点问题”，详情在核心的日志里。
  bool get isInternal => code == 'internal';

  @override
  String toString() => '$message（$code）';
}
