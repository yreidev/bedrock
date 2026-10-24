import 'package:bedrock/errors.dart';
import 'package:bedrock/gen/ops.dart';
import 'package:bedrock/notes/format.dart';
import 'package:bedrock/transport/transport.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

/// 备忘：新建、列出、删除。
class NotesPage extends ConsumerStatefulWidget {
  const NotesPage({super.key});

  @override
  ConsumerState<NotesPage> createState() => _NotesPageState();
}

class _NotesPageState extends ConsumerState<NotesPage> {
  final _title = TextEditingController();
  bool _saving = false;

  @override
  void dispose() {
    _title.dispose();
    super.dispose();
  }

  Future<void> _create() async {
    setState(() => _saving = true);
    try {
      await ref
          .read(opActionsProvider)
          .noteCreate(CreateNoteInput(title: _title.text));
      _title.clear();
    } on OpException catch (err) {
      _showError(err);
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _delete(NoteView note) async {
    try {
      await ref
          .read(opActionsProvider)
          .noteDelete(DeleteNoteInput(id: note.id));
    } on OpException catch (err) {
      _showError(err);
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ShadToaster.of(context)
        .show(ShadToast.destructive(title: Text(describeError(error))));
  }

  @override
  Widget build(BuildContext context) {
    final theme = ShadTheme.of(context);
    final notes = ref.watch(noteListQuery(const ListNotesInput()));
    return ColoredBox(
      color: theme.colorScheme.background,
      child: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 640),
            child: ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Text('备忘', style: theme.textTheme.h3),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Expanded(
                      child: ShadInput(
                        controller: _title,
                        placeholder: const Text('写下要记的事'),
                        textInputAction: TextInputAction.done,
                        onSubmitted: (_) => _create(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    ShadButton(
                      enabled: !_saving,
                      onPressed: _create,
                      leading: const Icon(LucideIcons.plus, size: 16),
                      child: const Text('新建备忘'),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                ...switch (notes) {
                  AsyncData(value: NoteList(notes: [])) => [
                    Text('还没有备忘', style: theme.textTheme.muted),
                  ],
                  AsyncData(value: NoteList(:final notes)) => [
                    for (final note in notes)
                      _NoteCard(note: note, onDelete: () => _delete(note)),
                  ],
                  AsyncError(:final error) => [
                    Text(describeError(error), style: theme.textTheme.muted),
                    const SizedBox(height: 8),
                    Align(
                      alignment: Alignment.centerLeft,
                      child: ShadButton.outline(
                        onPressed: () => ref.invalidate(noteListQuery),
                        child: const Text('重新加载'),
                      ),
                    ),
                  ],
                  _ => [Text('正在加载…', style: theme.textTheme.muted)],
                },
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _NoteCard extends StatelessWidget {
  const _NoteCard({required this.note, required this.onDelete});

  final NoteView note;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final theme = ShadTheme.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: ShadCard(
        padding: const EdgeInsets.fromLTRB(16, 12, 8, 12),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(note.title, style: theme.textTheme.large),
                  if (note.body case final body?)
                    Text(body, style: theme.textTheme.p),
                  Text(
                    formatTime(note.createdAt),
                    style: theme.textTheme.muted,
                  ),
                ],
              ),
            ),
            Semantics(
              container: true,
              button: true,
              label: '删除“${note.title}”',
              excludeSemantics: true,
              child: ShadIconButton.ghost(
                icon: const Icon(LucideIcons.trash2, size: 16),
                onPressed: onDelete,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
