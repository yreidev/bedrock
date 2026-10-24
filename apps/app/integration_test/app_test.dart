import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:bedrock/app.dart';
import 'package:bedrock/gen/ops.dart';
import 'package:bedrock/transport/http.dart';
import 'package:bedrock/transport/local.dart';
import 'package:bedrock/transport/transport.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

/// 编出来的命令行，`just e2e` 用 `--dart-define` 传进来；没有就跳过连服务的那一条。
const _cli = String.fromEnvironment('BEDROCK_CLI');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('本机跑核心：新建、列出、删除备忘', (tester) async {
    final data = await Directory.systemTemp.createTemp('bedrock-app-');
    addTearDown(() => data.delete(recursive: true));
    await _createListDelete(tester, await LocalTransport.open(data.path));
  });

  testWidgets('连 bedrock serve：新建、列出、删除备忘', (tester) async {
    final data = await Directory.systemTemp.createTemp('bedrock-serve-');
    final serve = await Process.start(
      _cli,
      ['serve', '--listen', '127.0.0.1:0'],
      environment: {'BEDROCK_DATA': data.path},
    );
    addTearDown(() async {
      serve.kill(ProcessSignal.sigterm);
      await serve.exitCode;
      await data.delete(recursive: true);
    });
    unawaited(serve.stderr.drain<void>());
    final ready = await serve.stdout
        .transform(utf8.decoder)
        .transform(const LineSplitter())
        .first;
    final address = RegExp(r'^在 (http://\S+) 上提供服务$').firstMatch(ready);
    expect(address, isNotNull, reason: ready);
    await _createListDelete(tester, HttpTransport(address!.group(1)!));
  }, skip: _cli.isEmpty);
}

Future<void> _createListDelete(WidgetTester tester, Transport transport) async {
  // 按语义标签找控件要先建语义树（读屏用的也是它），测试结束前要关掉
  final semantics = tester.ensureSemantics();
  try {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [opsProvider.overrideWithValue(Ops(transport))],
        child: const BedrockApp(),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('还没有备忘'), findsOneWidget);

    await tester.tap(find.text('新建备忘'));
    await tester.pumpAndSettle();
    expect(find.text('标题不能为空'), findsOneWidget, reason: '核心的校验错误要显示出来');

    await tester.enterText(find.byType(EditableText), '买牛奶');
    await tester.tap(find.text('新建备忘'));
    await tester.pumpAndSettle();
    expect(find.text('买牛奶'), findsOneWidget);
    expect(find.text('还没有备忘'), findsNothing);

    await tester.tap(find.bySemanticsLabel('删除“买牛奶”'));
    await tester.pumpAndSettle();
    expect(find.text('买牛奶'), findsNothing);
    expect(find.text('还没有备忘'), findsOneWidget);
  } finally {
    semantics.dispose();
  }
}
