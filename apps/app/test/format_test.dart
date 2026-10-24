import 'package:bedrock/errors.dart';
import 'package:bedrock/main.dart';
import 'package:bedrock/notes/format.dart';
import 'package:bedrock/transport/transport.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('时间按本地时区显示到分钟', () {
    final time = DateTime(2026, 3, 4, 5, 6, 7);
    expect(formatTime(time), '2026-03-04 05:06');
  });

  test('意料之外的错误不展示技术细节', () {
    expect(
      describeError(const OpException('internal', 'database is locked')),
      '出了点问题，请稍后再试',
    );
    expect(describeError(const OpException('empty_title', '标题不能为空')), '标题不能为空');
  });

  test('查询只在连不上和意料之外的错误时重试，最多三次', () {
    const unreachable = OpException(OpException.unreachable, '连不上');
    expect(retryTransient(0, unreachable), isNotNull);
    expect(retryTransient(3, unreachable), isNull);
    expect(retryTransient(0, const OpException('not_found', '没有')), isNull);
  });
}
