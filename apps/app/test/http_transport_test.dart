import 'dart:convert';

import 'package:bedrock/transport/http.dart';
import 'package:bedrock/transport/transport.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('发 JSON 到 /api/op/<名字>，拿回输出', () async {
    late http.Request sent;
    final transport = HttpTransport(
      'http://127.0.0.1:7400/',
      client: MockClient((request) async {
        sent = request;
        return http.Response(jsonEncode({'notes': <Object?>[]}), 200);
      }),
    );
    expect(await transport.call('note.list', {}), {'notes': <Object?>[]});
    expect(sent.url.toString(), 'http://127.0.0.1:7400/api/op/note.list');
    expect(sent.headers['content-type'], startsWith('application/json'));
  });

  test('错误回应变成带 code 的 OpException', () async {
    final transport = HttpTransport(
      'http://127.0.0.1:7400',
      client: MockClient(
        (_) async => http.Response(
          jsonEncode({'code': 'empty_title', 'message': '标题不能为空'}),
          400,
          headers: {'content-type': 'application/json; charset=utf-8'},
        ),
      ),
    );
    await expectLater(
      transport.call('note.create', {'title': ' '}),
      throwsA(
        isA<OpException>()
            .having((e) => e.code, 'code', 'empty_title')
            .having((e) => e.message, 'message', '标题不能为空'),
      ),
    );
  });

  test('连不上时报 unreachable', () async {
    final transport = HttpTransport(
      'http://127.0.0.1:7400',
      client: MockClient((_) async => throw http.ClientException('拒绝连接')),
    );
    await expectLater(
      transport.call('note.list', {}),
      throwsA(
        isA<OpException>().having(
          (e) => e.code,
          'code',
          OpException.unreachable,
        ),
      ),
    );
  });
}
