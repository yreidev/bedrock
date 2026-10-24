import 'package:bedrock/notes/notes_page.dart';
import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:go_router/go_router.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

final _router = GoRouter(
  routes: [GoRoute(path: '/', builder: (context, state) => const NotesPage())],
);

/// 应用的外壳：主题、语言、路由。用哪种传输由外面的 ProviderScope 决定。
class BedrockApp extends StatelessWidget {
  const BedrockApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ShadApp.router(
      title: 'bedrock',
      routerConfig: _router,
      themeMode: ThemeMode.system,
      theme: ShadThemeData(
        brightness: Brightness.light,
        colorScheme: ShadColorScheme.fromName('zinc'),
      ),
      darkTheme: ShadThemeData(
        brightness: Brightness.dark,
        colorScheme: ShadColorScheme.fromName(
          'zinc',
          brightness: Brightness.dark,
        ),
      ),
      locale: const Locale('zh', 'CN'),
      supportedLocales: const [Locale('zh', 'CN')],
      localizationsDelegates: GlobalMaterialLocalizations.delegates,
    );
  }
}
