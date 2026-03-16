import 'dart:convert';
import 'dart:io';

import 'package:hooks/hooks.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    // Build the Rust crate
    final rustCrateDir = input.packageRoot.resolve('rust');

    final cargoNightly =
        Platform.environment['FRB_SIMPLE_BUILD_CARGO_NIGHTLY'] == '1';
    final cargoExtraArgs =
        Platform.environment['FRB_SIMPLE_BUILD_CARGO_EXTRA_ARGS']?.split(' ') ??
            const <String>[];
    final skip = Platform.environment['FRB_SIMPLE_BUILD_SKIP'] == '1';
    final rustflags = Platform.environment['RUSTFLAGS'];

    if (skip) {
      print(
          'frb_utils::simpleBuild SKIP BUILD since environment variable requires this');
    } else {
      final featureArgs = [];
      await runCommand(
        'cargo',
        [
          if (cargoNightly) '+nightly',
          'build',
          '--release',
          ...cargoExtraArgs,
          ...featureArgs,
        ],
        pwd: rustCrateDir.toFilePath(),
        printCommandInStderr: true,
        env: {
          // Though runCommand auto pass environment variable to commands,
          // we do this to explicitly show this important flag
          if (rustflags != null) 'RUSTFLAGS': rustflags,
        },
      );
    }

    final dependencies = {
      rustCrateDir,
      input.packageRoot.resolve('build.rs'),
    };
    output.dependencies.addAll(dependencies);

    // Build the Java crate
    final javaDir = input.packageRoot.resolve('../../zmanim-modern');
    await runCommand(
      'mvn',
      [
        '-q',
        '-o',
        '-Dmaven.test.skip=true',
        '-Dmaven.javadoc.skip=true',
        '-Dmaven.source.skip=true',
        'package'
      ],
      pwd: javaDir.toFilePath(),
      printCommandInStderr: true,
    );

    output.dependencies.addAll({javaDir});
  });
}

/// {@macro flutter_rust_bridge.internal}
class RunCommandOutput {
  /// {@macro flutter_rust_bridge.internal}
  final String stdout;

  /// {@macro flutter_rust_bridge.internal}
  final String stderr;

  /// {@macro flutter_rust_bridge.internal}
  final int exitCode;

  /// {@macro flutter_rust_bridge.internal}
  const RunCommandOutput({
    required this.stdout,
    required this.stderr,
    required this.exitCode,
  });
}

/// {@macro flutter_rust_bridge.internal}
Future<RunCommandOutput> runCommand(
  String command,
  List<String> arguments, {
  String? pwd,
  Map<String, String>? env,
  bool shell = true,
  bool silent = false,
  bool? checkExitCode,
  bool printCommandInStderr = false,
}) async {
  // ignore: avoid_print
  (printCommandInStderr ? stderr : stdout).writeAndFlush(
      '\x1B[1m> $command ${arguments.join(' ')}\x1B[0m (pwd: $pwd, env: $env)\n');

  final process = await Process.start(
    command,
    arguments,
    runInShell: shell,
    workingDirectory: pwd,
    environment: env,
    includeParentEnvironment: true,
  );

  final stdoutText = <String>[];
  final stderrText = <String>[];

  process.stdout.transform(utf8.decoder).listen((line) {
    if (!silent) stdout.writeAndFlush(line);
    stdoutText.add(line);
  });

  process.stderr.transform(utf8.decoder).listen((line) {
    if (!silent) stderr.writeAndFlush(line);
    stderrText.add(line);
  });

  final exitCode = await process.exitCode;
  if ((checkExitCode ?? true) && (exitCode != 0)) {
    const envKey = 'FRB_DART_RUN_COMMAND_STDERR';
    final enableStderr = Platform.environment[envKey] == '1';
    final info = enableStderr
        ? 'stderr=${stderrText.join("")}'
        : 'If you want to see extra information, set $envKey=1';
    throw ProcessException(
        command, arguments, 'Bad exit code ($exitCode). $info', exitCode);
  }

  return RunCommandOutput(
    stdout: stdoutText.join(''),
    stderr: stderrText.join(''),
    exitCode: exitCode,
  );
}

extension on IOSink {
  void writeAndFlush(String message) {
    write(message);
    flush();
  }
}
