import 'dart:io';

import 'package:bedrock/app.dart';
import 'package:bedrock/gen/ops.dart';
import 'package:bedrock/transport/http.dart';
import 'package:bedrock/transport/local.dart';
import 'package:bedrock/transport/transport.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:xdg_directories/xdg_directories.dart' as xdg;

/// 编译时给 `--dart-define=BEDROCK_SERVER=http://地址:端口` 就连那台电脑上的
/// `bedrock serve`，不给就在本机跑核心。
const _server = String.fromEnvironment('BEDROCK_SERVER');

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final Transport transport = _server.isEmpty
      ? await LocalTransport.open(await _dataDir())
      : HttpTransport(_server);
  runApp(
    ProviderScope(
      overrides: [opsProvider.overrideWithValue(Ops(transport))],
      retry: retryTransient,
      child: const BedrockApp(),
    ),
  );
}

/// 应用的数据目录，放在系统给这个应用的地方。
///
/// Linux 上直接用 `$XDG_DATA_HOME/app.bedrock`：path_provider 在这个目录还不存在、
/// 而 `$XDG_DATA_HOME/bedrock` 存在时会退回用后者，那是命令行的数据目录。
Future<String> _dataDir() async => Platform.isLinux
    ? '${xdg.dataHome.path}/app.bedrock'
    : (await getApplicationSupportDirectory()).path;

/// 查询失败时只重试可能自己好起来的：连不上、意料之外的错误。最多三次，越等越久。
Duration? retryTransient(int retryCount, Object error) => switch (error) {
  OpException(:final code)
      when retryCount < 3 &&
          (code == OpException.unreachable || code == 'internal') =>
    Duration(milliseconds: 200 << retryCount),
  _ => null,
};
