import 'dart:io';

void main(List<String> args) async {
  // Build the Rust crate
  final rustCrateDir = Platform.script.resolve('rust');
  final rustflags = Platform.environment['RUSTFLAGS'];

  stderr.writeln("Building Rust crate...");
  final rustProcess = await Process.start(
    'cargo',
    [
      'build',
      '--release',
    ],
    workingDirectory: rustCrateDir.toFilePath(),
    environment: {
      // Though runCommand auto pass environment variable to commands,
      // we do this to explicitly show this important flag
      if (rustflags != null) 'RUSTFLAGS': rustflags,
    },
    mode: ProcessStartMode.inheritStdio,
  );
  final rustExitCode = await rustProcess.exitCode;
  if (rustExitCode != 0) {
    stderr.writeln("Rust build failed with exit code $rustExitCode");
    exit(rustExitCode);
  }

  // Build the Java library
  final javaDir = Platform.script.resolve('java');

  stderr.writeln("Building Java library...");
  final javaProcess = await Process.start(
    'mvn',
    [
      '-q',
      '-o',
      '-Dmaven.test.skip=true',
      '-Dmaven.javadoc.skip=true',
      '-Dmaven.source.skip=true',
      'package'
    ],
    includeParentEnvironment: true,
    runInShell: true,
    workingDirectory: javaDir.toFilePath(),
    mode: ProcessStartMode.inheritStdio,
  );
  final javaExitCode = await javaProcess.exitCode;

  if (javaExitCode != 0) {
    stderr.writeln("Java build failed with exit code $javaExitCode");
    exit(javaExitCode);
  }
  stderr.writeln("Build complete");
}
