import 'package:bedrock/gen/ops.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('可选字段没值时不写这个键', () {
    expect(const CreateNoteInput(title: '买牛奶').toJson(), {'title': '买牛奶'});
    expect(const CreateNoteInput(title: '交房租', body: '月底前').toJson(), {
      'title': '交房租',
      'body': '月底前',
    });
  });

  test('读核心的输出：时间转成 DateTime，没有的字段是 null', () {
    final list = NoteList.fromJson({
      'notes': [
        {'id': 'id-1', 'title': '买牛奶', 'created_at': '2026-01-01T00:00:01Z'},
      ],
    });
    final note = list.notes.single;
    expect(note.body, isNull);
    expect(note.createdAt, DateTime.utc(2026, 1, 1, 0, 0, 1));
    expect(NoteList.fromJson(list.toJson()), list);
  });

  test('输入按值比较，查询才能按输入缓存', () {
    expect(const ListNotesInput(), const ListNotesInput());
    expect(
      const CreateNoteInput(title: 'a').hashCode,
      const CreateNoteInput(title: 'a').hashCode,
    );
    expect(
      const CreateNoteInput(title: 'a'),
      isNot(const CreateNoteInput(title: 'a', body: 'b')),
    );
  });
}
