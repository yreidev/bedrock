import 'dart:convert';

import 'package:bedrock/transport/transport.dart';
import 'package:http/http.dart' as http;

/// 经网络调另一台电脑上的 `bedrock serve`。
final class HttpTransport implements Transport {
  /// [server] 写成 `http://地址:端口`。
  HttpTransport(String server, {http.Client? client})
    : _server = server.endsWith('/')
          ? server.substring(0, server.length - 1)
          : server,
      _client = client ?? http.Client();

  final String _server;
  final http.Client _client;

  @override
  Future<Map<String, Object?>> call(
    String name,
    Map<String, Object?> input,
  ) async {
    final http.Response response;
    try {
      response = await _client.post(
        Uri.parse('$_server/api/op/$name'),
        headers: const {'content-type': 'application/json'},
        body: jsonEncode(input),
      );
    } on http.ClientException catch (err) {
      throw OpException(OpException.unreachable, '连不上 $_server：${err.message}');
    }
    final Object? body;
    try {
      body = jsonDecode(utf8.decode(response.bodyBytes));
    } on FormatException {
      throw OpException('internal', '服务回的不是 JSON（状态码 ${response.statusCode}）');
    }
    return switch (body) {
      final Map<String, Object?> output when response.statusCode == 200 =>
        output,
      {'code': final String code, 'message': final String message} =>
        throw OpException(code, message),
      _ => throw OpException(
        'internal',
        '服务回了看不懂的内容（状态码 ${response.statusCode}）',
      ),
    };
  }
}
