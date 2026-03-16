import 'dart:io';

void main(List<String> args) async {
  // Build the Rust crate
  final rustCrateDir = Platform.script.resolve('rust');
  final rustflags = Platform.environment['RUSTFLAGS'];

  final rustResult = await Process.run(
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
  );
  if (rustResult.exitCode != 0) {
    stderr.writeln(rustResult.stderr);
    exit(rustResult.exitCode);
  }

  // Build the Java library
  final javaDir = Platform.script.resolve('java');
  final javaResult = await Process.run(
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
  );
  if (javaResult.exitCode != 0) {
    stderr.writeln(javaResult.stderr);
    exit(javaResult.exitCode);
  }
  stderr.writeln("Build complete");
}
