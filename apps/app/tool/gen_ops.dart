// 从操作清单生成 lib/gen/ops.dart：每个输入输出一个类、每个操作的错误码、带类型的操作表，
// 以及给界面用的 Riverpod provider。`just gen` 调它。
//
// 用法：dart run tool/gen_ops.dart <ops.json> <输出文件>
//
// 输出没排版，`just gen` 接着跑 dart format。这样生成器只依赖 code_builder，
// 每次 dart run 现场编译只要几秒。
//
// 只认核心会导出的那几种 Schema 写法（对象、字符串、时间、数字、布尔、列表、引用），
// 遇到别的直接报错：先在这里补上对应的写法，再在核心里用它。

import 'dart:convert';
import 'dart:io';

import 'package:code_builder/code_builder.dart';

void main(List<String> args) {
  if (args.length != 2) {
    stderr.writeln('用法：dart run tool/gen_ops.dart <ops.json> <输出文件>');
    exit(2);
  }
  final String code;
  try {
    final manifest = jsonDecode(File(args[0]).readAsStringSync());
    code = generate(_object(manifest, '清单'));
  } on FormatException catch (e) {
    stderr.writeln('生成不了：${e.message}');
    exit(1);
  }
  File(args[1])
    ..createSync(recursive: true)
    ..writeAsStringSync(code);
}

String generate(Map<String, Object?> manifest) {
  final defs = _object(manifest[r'$defs'], r'$defs');
  final operations = [
    for (final (i, op) in _list(manifest['operations'], 'operations').indexed)
      _Operation.parse(_object(op, 'operations[$i]')),
  ];
  final types = [
    for (final MapEntry(:key, :value) in defs.entries)
      _TypeDef.parse(key, _object(value, '\$defs.$key')),
  ];
  for (final op in operations) {
    for (final name in [op.input, op.output]) {
      if (!defs.containsKey(name)) {
        throw FormatException('${op.name} 引用的 $name 不在 \$defs 里');
      }
    }
  }

  final library = Library(
    (l) => l
      ..comments.addAll([
        '自动生成，不要手改：改了操作的输入输出后运行 just gen。',
        '来源：schema/ops.json（版本 ${manifest['version']}）',
      ])
      ..directives.addAll([
        Directive.import('package:flutter_riverpod/flutter_riverpod.dart'),
        Directive.import('package:bedrock/transport/transport.dart'),
      ])
      ..body.addAll([
        Field(
          (f) => f
            ..docs.add('/// 操作清单的版本，跟核心的版本走。')
            ..modifier = FieldModifier.constant
            ..name = 'opsVersion'
            ..assignment = Code(_string('${manifest['version']}')),
        ),
        for (final type in types) type.toClass(),
        for (final op in operations) op.errorsClass(),
        _opsClass(operations),
        ..._providers(operations),
        if (types.any((t) => t.fields.any((f) => f.type is _List)))
          Code(_listEqualsHelper),
      ]),
  );
  return library
      .accept(DartEmitter(useNullSafetySyntax: true, orderDirectives: true))
      .toString();
}

/// 一个操作：名字、种类、输入输出的类型名、可能的错误码。
final class _Operation {
  _Operation(this.name, this.kind, this.input, this.output, this.errors);

  factory _Operation.parse(Map<String, Object?> json) {
    final name = _string_(json['name'], '操作名');
    if (!RegExp(r'^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$').hasMatch(name)) {
      throw FormatException('操作名要写成 <业务>.<动作>：$name');
    }
    final kind = switch (json['kind']) {
      'query' => _Kind.query,
      'action' => _Kind.action,
      final other => throw FormatException('$name 的种类不认识：$other'),
    };
    return _Operation(
      name,
      kind,
      _refName(_object(json['input'], '$name.input'), '$name.input'),
      _refName(_object(json['output'], '$name.output'), '$name.output'),
      [
        for (final code in _list(json['errors'], '$name.errors'))
          _string_(code, '$name 的错误码'),
      ],
    );
  }

  final String name;
  final _Kind kind;
  final String input;
  final String output;
  final List<String> errors;

  /// `note.create` → `noteCreate`
  String get method => _camel(name.replaceAll('.', '_'));

  /// 操作名的第一段，动作成功后按它刷新查询。
  String get business => name.split('.').first;

  String get errorsClassName => '${_pascal(name.replaceAll('.', '_'))}Errors';

  String get doc {
    final kindName = kind == _Kind.query ? '查询' : '动作';
    final codes = errors.map((e) => '`$e`').join('、');
    return '/// `$name`（$kindName）。可能的错误码：$codes，见 [$errorsClassName]。';
  }

  Class errorsClass() => Class(
    (c) => c
      ..docs.add('/// `$name` 可能报的错误码，和 [OpException.code] 比较。')
      ..abstract = true
      ..modifier = ClassModifier.final$
      ..name = errorsClassName
      ..fields.addAll([
        for (final code in errors)
          Field(
            (f) => f
              ..static = true
              ..modifier = FieldModifier.constant
              ..name = _identifier(_camel(code))
              ..assignment = Code(_string(code)),
          ),
      ]),
  );
}

enum _Kind { query, action }

/// 一个输入或输出的类型：只能是对象。
final class _TypeDef {
  _TypeDef(this.name, this.doc, this.fields);

  factory _TypeDef.parse(String name, Map<String, Object?> json) {
    _onlyKeys(json, const {
      'type',
      'properties',
      'required',
      'description',
      'title',
      'additionalProperties',
    }, '\$defs.$name');
    if (json['type'] != 'object') {
      throw FormatException('\$defs.$name 要是对象，现在是 ${json['type']}');
    }
    final properties = json['properties'] == null
        ? const <String, Object?>{}
        : _object(json['properties'], '\$defs.$name.properties');
    final required = {
      for (final key in _list(json['required'] ?? const [], '$name.required'))
        _string_(key, '$name.required'),
    };
    return _TypeDef(name, json['description'] as String?, [
      for (final MapEntry(:key, :value) in properties.entries)
        _FieldDef.parse(
          key,
          _object(value, '$name.$key'),
          required.contains(key),
          '$name.$key',
        ),
    ]);
  }

  final String name;
  final String? doc;
  final List<_FieldDef> fields;

  Class toClass() => Class(
    (c) => c
      ..docs.addAll(_docs(doc))
      ..modifier = ClassModifier.final$
      ..name = name
      ..constructors.addAll([
        Constructor(
          (k) => k
            ..constant = true
            ..optionalParameters.addAll([
              for (final field in fields)
                Parameter(
                  (p) => p
                    ..named = true
                    ..required = field.required
                    ..toThis = true
                    ..name = field.dartName,
                ),
            ]),
        ),
        Constructor(
          (k) => k
            ..factory = true
            ..name = 'fromJson'
            ..requiredParameters.add(
              Parameter(
                (p) => p
                  ..name = fields.isEmpty ? '_' : 'json'
                  ..type = refer('Map<String, Object?>'),
              ),
            )
            ..lambda = true
            ..body = Code(
              fields.isEmpty
                  ? 'const $name()'
                  : '$name(${fields.map((f) => '${f.dartName}: ${f.decode()},').join()})',
            ),
        ),
      ])
      ..fields.addAll([
        for (final field in fields)
          Field(
            (f) => f
              ..docs.addAll(_docs(field.doc))
              ..modifier = FieldModifier.final$
              ..type = refer(field.type.dart + (field.required ? '' : '?'))
              ..name = field.dartName,
          ),
      ])
      ..methods.addAll([
        Method(
          (m) => m
            ..returns = refer('Map<String, Object?>')
            ..name = 'toJson'
            ..lambda = true
            ..body = Code(
              fields.isEmpty
                  ? 'const {}'
                  : '{${fields.map((f) => '${f.encode()},').join()}}',
            ),
        ),
        Method(
          (m) => m
            ..annotations.add(refer('override'))
            ..returns = refer('bool')
            ..name = 'operator =='
            ..requiredParameters.add(
              Parameter(
                (p) => p
                  ..name = 'other'
                  ..type = refer('Object'),
              ),
            )
            ..lambda = true
            ..body = Code(
              [
                'other is $name',
                for (final f in fields) f.equals(),
              ].join(' && '),
            ),
        ),
        Method(
          (m) => m
            ..annotations.add(refer('override'))
            ..returns = refer('int')
            ..type = MethodType.getter
            ..name = 'hashCode'
            ..lambda = true
            ..body = Code(
              'Object.hashAll([${_string(name)}, ${fields.map((f) => '${f.hash()},').join()}])',
            ),
        ),
      ]),
  );
}

/// 对象里的一个字段。
final class _FieldDef {
  _FieldDef(this.jsonName, this.type, this.required, this.doc);

  factory _FieldDef.parse(
    String jsonName,
    Map<String, Object?> json,
    bool required,
    String where,
  ) {
    if (!RegExp(r'^[a-z][a-z0-9_]*$').hasMatch(jsonName)) {
      throw FormatException('$where：字段名要是 snake_case');
    }
    return _FieldDef(
      jsonName,
      _parseType(json, where),
      required,
      json['description'] as String?,
    );
  }

  final String jsonName;
  final _Type type;
  final bool required;
  final String? doc;

  String get dartName => _identifier(_camel(jsonName));

  String get _value => "json[${_string(jsonName)}]";

  String decode() => switch (type) {
    _ when required => type.decode(_value, 0),
    _Scalar(dart: final dart) when dart != 'double' => '$_value as $dart?',
    _ => '$_value == null ? null : ${type.decode(_value, 0)}',
  };

  /// 可选字段没值时不写这个键。
  String encode() {
    final key = _string(jsonName);
    if (required) return '$key: ${type.encode(dartName, 0)}';
    return switch (type.encodeNullable(dartName)) {
      final value? => '$key: ?$value',
      null =>
        'if ($dartName case final $dartName?) $key: ${type.encode(dartName, 0)}',
    };
  }

  String equals() => type is _List
      ? '_listEquals(other.$dartName, $dartName)'
      : 'other.$dartName == $dartName';

  String hash() => type is _List
      ? (required
            ? 'Object.hashAll($dartName)'
            : 'Object.hashAll($dartName ?? const [])')
      : dartName;
}

/// 字段的类型，以及它和 JSON 之间怎么转。`depth` 用来给嵌套列表的循环变量起不同的名字。
sealed class _Type {
  const _Type();

  String get dart;
  String decode(String value, int depth);
  String encode(String value, int depth);

  /// 值可能是 null 时怎么写，配合 `'键': ?值` 只在有值时写这个键；写不成一个表达式时给 null。
  String? encodeNullable(String value);
}

final class _Scalar extends _Type {
  const _Scalar(this.dart);

  @override
  final String dart;

  @override
  String decode(String value, int depth) => switch (dart) {
    'double' => '($value as num).toDouble()',
    _ => '$value as $dart',
  };

  @override
  String encode(String value, int depth) => value;

  @override
  String encodeNullable(String value) => value;
}

final class _Time extends _Type {
  const _Time();

  @override
  String get dart => 'DateTime';

  @override
  String decode(String value, int depth) => 'DateTime.parse($value as String)';

  @override
  String encode(String value, int depth) => '$value.toUtc().toIso8601String()';

  @override
  String encodeNullable(String value) => '$value?.toUtc().toIso8601String()';
}

final class _Ref extends _Type {
  const _Ref(this.dart);

  @override
  final String dart;

  @override
  String decode(String value, int depth) =>
      '$dart.fromJson($value as Map<String, Object?>)';

  @override
  String encode(String value, int depth) => '$value.toJson()';

  @override
  String encodeNullable(String value) => '$value?.toJson()';
}

final class _List extends _Type {
  const _List(this.item);

  final _Type item;

  @override
  String get dart => 'List<${item.dart}>';

  @override
  String decode(String value, int depth) =>
      '[for (final e$depth in $value as List<Object?>) ${item.decode('e$depth', depth + 1)}]';

  @override
  String encode(String value, int depth) =>
      '[for (final e$depth in $value) ${item.encode('e$depth', depth + 1)}]';

  @override
  String? encodeNullable(String value) => null;
}

_Type _parseType(Map<String, Object?> json, String where) {
  _onlyKeys(json, const {
    r'$ref',
    'type',
    'format',
    'items',
    'description',
    'minimum',
    'maximum',
  }, where);
  if (json[r'$ref'] != null) {
    return _Ref(_refName(json, where));
  }
  return switch (json['type']) {
    'string' => switch (json['format']) {
      null => const _Scalar('String'),
      'date-time' => const _Time(),
      final other => throw FormatException('$where：不认识的字符串格式 $other'),
    },
    'integer' => const _Scalar('int'),
    'number' => const _Scalar('double'),
    'boolean' => const _Scalar('bool'),
    'array' => _List(
      _parseType(_object(json['items'], '$where[]'), '$where[]'),
    ),
    final other => throw FormatException('$where：不认识的类型 $other'),
  };
}

Class _opsClass(List<_Operation> operations) => Class(
  (c) => c
    ..docs.add('/// 按名字调用核心的操作，输入输出都有类型；底下是哪种传输由 [transport] 决定。')
    ..modifier = ClassModifier.final$
    ..name = 'Ops'
    ..constructors.add(
      Constructor(
        (k) => k
          ..constant = true
          ..requiredParameters.add(
            Parameter(
              (p) => p
                ..toThis = true
                ..name = 'transport',
            ),
          ),
      ),
    )
    ..fields.add(
      Field(
        (f) => f
          ..modifier = FieldModifier.final$
          ..type = refer('Transport')
          ..name = 'transport',
      ),
    )
    ..methods.addAll([
      for (final op in operations)
        Method(
          (m) => m
            ..docs.add(op.doc)
            ..returns = refer('Future<${op.output}>')
            ..name = op.method
            ..requiredParameters.add(
              Parameter(
                (p) => p
                  ..name = 'input'
                  ..type = refer(op.input),
              ),
            )
            ..modifier = MethodModifier.async
            ..lambda = true
            ..body = Code(
              '${op.output}.fromJson(await transport.call(${_string(op.name)}, input.toJson()))',
            ),
        ),
    ]),
);

/// 查询各一个 provider，按输入缓存；动作放进 [OpActions]，成功后让同一业务的查询失效。
List<Spec> _providers(List<_Operation> operations) {
  final queries = operations.where((op) => op.kind == _Kind.query).toList();
  final actions = operations.where((op) => op.kind == _Kind.action).toList();
  return [
    Field(
      (f) => f
        ..docs.addAll([
          '/// 当前的操作入口。启动时换成用真的传输建的：',
          '/// `opsProvider.overrideWithValue(Ops(transport))`。',
        ])
        ..modifier = FieldModifier.final$
        ..name = 'opsProvider'
        ..assignment = const Code(
          "Provider<Ops>((ref) => throw StateError('启动时要给 opsProvider 指定传输'))",
        ),
    ),
    for (final op in queries)
      Field(
        (f) => f
          ..docs.addAll([op.doc, '///', '/// 按输入缓存，没人用时释放。'])
          ..modifier = FieldModifier.final$
          ..name = '${op.method}Query'
          ..assignment = Code(
            'FutureProvider.autoDispose.family<${op.output}, ${op.input}>('
            '(ref, input) => ref.watch(opsProvider).${op.method}(input),)',
          ),
      ),
    Field(
      (f) => f
        ..docs.add('/// 界面调动作都经它，成功后同一业务的查询会重新取。')
        ..modifier = FieldModifier.final$
        ..name = 'opActionsProvider'
        ..assignment = const Code('Provider<OpActions>(OpActions.new)'),
    ),
    Class(
      (c) => c
        ..docs.add('/// 所有动作。成功后让同一业务（操作名的第一段）的查询都失效，不用在页面里手动刷新。')
        ..modifier = ClassModifier.final$
        ..name = 'OpActions'
        ..constructors.add(
          Constructor(
            (k) => k
              ..requiredParameters.add(
                Parameter(
                  (p) => p
                    ..toThis = true
                    ..name = '_ref',
                ),
              ),
          ),
        )
        ..fields.add(
          Field(
            (f) => f
              ..modifier = FieldModifier.final$
              ..type = refer('Ref')
              ..name = '_ref',
          ),
        )
        ..methods.addAll([
          for (final op in actions)
            Method((m) {
              final stale = queries.where((q) => q.business == op.business);
              m
                ..docs.add(op.doc)
                ..returns = refer('Future<${op.output}>')
                ..name = op.method
                ..requiredParameters.add(
                  Parameter(
                    (p) => p
                      ..name = 'input'
                      ..type = refer(op.input),
                  ),
                )
                ..modifier = MethodModifier.async
                ..body = Code(
                  'final output = await _ref.read(opsProvider).${op.method}(input);'
                  '${stale.map((q) => '_ref.invalidate(${q.method}Query);').join()}'
                  'return output;',
                );
            }),
        ]),
    ),
  ];
}

const _listEqualsHelper = '''
bool _listEquals<T>(List<T>? a, List<T>? b) {
  if (identical(a, b)) return true;
  if (a == null || b == null || a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
''';

Map<String, Object?> _object(Object? value, String where) => switch (value) {
  final Map<String, Object?> map => map,
  _ => throw FormatException('$where 要是对象'),
};

List<Object?> _list(Object? value, String where) => switch (value) {
  final List<Object?> list => list,
  _ => throw FormatException('$where 要是列表'),
};

String _string_(Object? value, String where) => switch (value) {
  final String s => s,
  _ => throw FormatException('$where 要是字符串'),
};

String _refName(Map<String, Object?> json, String where) {
  const prefix = r'#/$defs/';
  final ref = _string_(json[r'$ref'], '$where 的引用');
  if (!ref.startsWith(prefix)) {
    throw FormatException('$where：只认 $prefix 下的引用，遇到 $ref');
  }
  return ref.substring(prefix.length);
}

void _onlyKeys(Map<String, Object?> json, Set<String> allowed, String where) {
  for (final key in json.keys) {
    if (!allowed.contains(key)) {
      throw FormatException('$where：不认识的写法 “$key”，先在生成器里补上');
    }
  }
}

/// 写成 Dart 的单引号字符串。
String _string(String value) =>
    "'${value.replaceAll(r'\', r'\\').replaceAll("'", r"\'").replaceAll(r'$', r'\$')}'";

List<String> _docs(String? doc) =>
    doc == null ? const [] : [for (final line in doc.split('\n')) '/// $line'];

/// `created_at` → `createdAt`
String _camel(String snake) {
  final parts = snake.split('_');
  return parts.first + parts.skip(1).map(_capitalize).join();
}

/// `note_create` → `NoteCreate`
String _pascal(String snake) => snake.split('_').map(_capitalize).join();

String _capitalize(String s) =>
    s.isEmpty ? s : s[0].toUpperCase() + s.substring(1);

/// 撞上 Dart 关键字时在后面加下划线。
String _identifier(String name) => _reserved.contains(name) ? '${name}_' : name;

const _reserved = {
  'assert',
  'break',
  'case',
  'catch',
  'class',
  'const',
  'continue',
  'default',
  'do',
  'else',
  'enum',
  'extends',
  'false',
  'final',
  'finally',
  'for',
  'if',
  'in',
  'is',
  'new',
  'null',
  'rethrow',
  'return',
  'super',
  'switch',
  'this',
  'throw',
  'true',
  'try',
  'var',
  'void',
  'while',
  'with',
  'hashCode',
  'runtimeType',
  'toString',
  'toJson',
};
