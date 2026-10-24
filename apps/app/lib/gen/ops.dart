// 自动生成，不要手改：改了操作的输入输出后运行 just gen。
// 来源：schema/ops.json（版本 0.1.0）

import 'package:bedrock/transport/transport.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// 操作清单的版本，跟核心的版本走。
const opsVersion = '0.1.0';

final class CreateNoteInput {
  const CreateNoteInput({required this.title, this.body});

  factory CreateNoteInput.fromJson(Map<String, Object?> json) =>
      CreateNoteInput(
        title: json['title'] as String,
        body: json['body'] as String?,
      );

  /// 去掉首尾空白后 1 到 200 个字
  final String title;

  /// 最多 10000 个字，可以不写
  final String? body;

  Map<String, Object?> toJson() => {'title': title, 'body': ?body};

  @override
  bool operator ==(Object other) =>
      other is CreateNoteInput && other.title == title && other.body == body;

  @override
  int get hashCode => Object.hashAll(['CreateNoteInput', title, body]);
}

/// 一条备忘，给调用方看的样子。
final class NoteView {
  const NoteView({
    required this.id,
    required this.title,
    this.body,
    required this.createdAt,
  });

  factory NoteView.fromJson(Map<String, Object?> json) => NoteView(
    id: json['id'] as String,
    title: json['title'] as String,
    body: json['body'] as String?,
    createdAt: DateTime.parse(json['created_at'] as String),
  );

  final String id;

  final String title;

  /// 没有正文时不出现
  final String? body;

  final DateTime createdAt;

  Map<String, Object?> toJson() => {
    'id': id,
    'title': title,
    'body': ?body,
    'created_at': createdAt.toUtc().toIso8601String(),
  };

  @override
  bool operator ==(Object other) =>
      other is NoteView &&
      other.id == id &&
      other.title == title &&
      other.body == body &&
      other.createdAt == createdAt;

  @override
  int get hashCode => Object.hashAll(['NoteView', id, title, body, createdAt]);
}

final class ListNotesInput {
  const ListNotesInput();

  factory ListNotesInput.fromJson(Map<String, Object?> _) =>
      const ListNotesInput();

  Map<String, Object?> toJson() => const {};

  @override
  bool operator ==(Object other) => other is ListNotesInput;

  @override
  int get hashCode => Object.hashAll(['ListNotesInput']);
}

final class NoteList {
  const NoteList({required this.notes});

  factory NoteList.fromJson(Map<String, Object?> json) => NoteList(
    notes: [
      for (final e0 in json['notes'] as List<Object?>)
        NoteView.fromJson(e0 as Map<String, Object?>),
    ],
  );

  final List<NoteView> notes;

  Map<String, Object?> toJson() => {
    'notes': [for (final e0 in notes) e0.toJson()],
  };

  @override
  bool operator ==(Object other) =>
      other is NoteList && _listEquals(other.notes, notes);

  @override
  int get hashCode => Object.hashAll(['NoteList', Object.hashAll(notes)]);
}

final class DeleteNoteInput {
  const DeleteNoteInput({required this.id});

  factory DeleteNoteInput.fromJson(Map<String, Object?> json) =>
      DeleteNoteInput(id: json['id'] as String);

  final String id;

  Map<String, Object?> toJson() => {'id': id};

  @override
  bool operator ==(Object other) => other is DeleteNoteInput && other.id == id;

  @override
  int get hashCode => Object.hashAll(['DeleteNoteInput', id]);
}

final class DeleteNoteOutput {
  const DeleteNoteOutput();

  factory DeleteNoteOutput.fromJson(Map<String, Object?> _) =>
      const DeleteNoteOutput();

  Map<String, Object?> toJson() => const {};

  @override
  bool operator ==(Object other) => other is DeleteNoteOutput;

  @override
  int get hashCode => Object.hashAll(['DeleteNoteOutput']);
}

/// `note.create` 可能报的错误码，和 [OpException.code] 比较。
abstract final class NoteCreateErrors {
  static const emptyTitle = 'empty_title';

  static const titleTooLong = 'title_too_long';

  static const bodyTooLong = 'body_too_long';

  static const internal = 'internal';
}

/// `note.list` 可能报的错误码，和 [OpException.code] 比较。
abstract final class NoteListErrors {
  static const internal = 'internal';
}

/// `note.delete` 可能报的错误码，和 [OpException.code] 比较。
abstract final class NoteDeleteErrors {
  static const notFound = 'not_found';

  static const internal = 'internal';
}

/// 按名字调用核心的操作，输入输出都有类型；底下是哪种传输由 [transport] 决定。
final class Ops {
  const Ops(this.transport);

  final Transport transport;

  /// `note.create`（动作）。可能的错误码：`empty_title`、`title_too_long`、`body_too_long`、`internal`，见 [NoteCreateErrors]。
  Future<NoteView> noteCreate(CreateNoteInput input) async =>
      NoteView.fromJson(await transport.call('note.create', input.toJson()));

  /// `note.list`（查询）。可能的错误码：`internal`，见 [NoteListErrors]。
  Future<NoteList> noteList(ListNotesInput input) async =>
      NoteList.fromJson(await transport.call('note.list', input.toJson()));

  /// `note.delete`（动作）。可能的错误码：`not_found`、`internal`，见 [NoteDeleteErrors]。
  Future<DeleteNoteOutput> noteDelete(DeleteNoteInput input) async =>
      DeleteNoteOutput.fromJson(
        await transport.call('note.delete', input.toJson()),
      );
}

/// 当前的操作入口。启动时换成用真的传输建的：
/// `opsProvider.overrideWithValue(Ops(transport))`。
final opsProvider = Provider<Ops>(
  (ref) => throw StateError('启动时要给 opsProvider 指定传输'),
);

/// `note.list`（查询）。可能的错误码：`internal`，见 [NoteListErrors]。
///
/// 按输入缓存，没人用时释放。
final noteListQuery = FutureProvider.autoDispose
    .family<NoteList, ListNotesInput>(
      (ref, input) => ref.watch(opsProvider).noteList(input),
    );

/// 界面调动作都经它，成功后同一业务的查询会重新取。
final opActionsProvider = Provider<OpActions>(OpActions.new);

/// 所有动作。成功后让同一业务（操作名的第一段）的查询都失效，不用在页面里手动刷新。
final class OpActions {
  OpActions(this._ref);

  final Ref _ref;

  /// `note.create`（动作）。可能的错误码：`empty_title`、`title_too_long`、`body_too_long`、`internal`，见 [NoteCreateErrors]。
  Future<NoteView> noteCreate(CreateNoteInput input) async {
    final output = await _ref.read(opsProvider).noteCreate(input);
    _ref.invalidate(noteListQuery);
    return output;
  }

  /// `note.delete`（动作）。可能的错误码：`not_found`、`internal`，见 [NoteDeleteErrors]。
  Future<DeleteNoteOutput> noteDelete(DeleteNoteInput input) async {
    final output = await _ref.read(opsProvider).noteDelete(input);
    _ref.invalidate(noteListQuery);
    return output;
  }
}

bool _listEquals<T>(List<T>? a, List<T>? b) {
  if (identical(a, b)) return true;
  if (a == null || b == null || a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
