import 'package:bedrock/transport/transport.dart';

/// 给人看的错误说明：意料之外的错误不展示技术细节。
String describeError(Object error) => switch (error) {
  OpException(isInternal: true) => '出了点问题，请稍后再试',
  OpException(:final message) => message,
  _ => '出了点问题，请稍后再试',
};
