import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

const crateName = 'interop';

void main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) return;

    final codeConfig = input.config.code;
    final targetTriple = _targetTriple(codeConfig);
    final workspaceRoot = input.packageRoot.resolve('../../..');
    final libraryFileName = codeConfig.targetOS.dylibFileName(
      crateName.replaceAll('-', '_'),
    );
    final libraryPath = workspaceRoot.resolve(
      'target/$targetTriple/debug/$libraryFileName',
    );

    final cargo = await Process.run(
      'cargo',
      ['build', '-p', crateName, '--target', targetTriple],
      workingDirectory: workspaceRoot.toFilePath(),
      environment: _cargoEnvironment(codeConfig, targetTriple),
    );

    if (cargo.exitCode != 0) {
      throw Exception(
        'Failed to build $crateName for $targetTriple:\n'
        '${cargo.stderr}\n${cargo.stdout}',
      );
    }

    if (!File.fromUri(libraryPath).existsSync()) {
      throw Exception(
        'Expected native library at ${libraryPath.toFilePath()} '
        'after cargo build for $targetTriple.',
      );
    }

    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'lib.g.dart',
        linkMode: DynamicLoadingBundled(),
        file: libraryPath,
      ),
    );

    output.dependencies.add(workspaceRoot.resolve('interop/Cargo.toml'));
  });
}

String _targetTriple(CodeConfig config) {
  return switch ((config.targetOS, config.targetArchitecture)) {
    (OS.android, Architecture.arm64) => 'aarch64-linux-android',
    (OS.android, Architecture.arm) => 'armv7-linux-androideabi',
    (OS.android, Architecture.x64) => 'x86_64-linux-android',
    (OS.iOS, Architecture.arm64) when config.iOS.targetSdk == IOSSdk.iPhoneSimulator =>
      'aarch64-apple-ios-sim',
    (OS.iOS, Architecture.arm64) => 'aarch64-apple-ios',
    (OS.iOS, Architecture.x64) => 'x86_64-apple-ios',
    (OS.windows, Architecture.arm64) => 'aarch64-pc-windows-msvc',
    (OS.windows, Architecture.x64) => 'x86_64-pc-windows-msvc',
    (OS.linux, Architecture.arm64) => 'aarch64-unknown-linux-gnu',
    (OS.linux, Architecture.x64) => 'x86_64-unknown-linux-gnu',
    (OS.macOS, Architecture.arm64) => 'aarch64-apple-darwin',
    (OS.macOS, Architecture.x64) => 'x86_64-apple-darwin',
    _ => throw UnsupportedError(
      'Unsupported target: ${config.targetOS} on ${config.targetArchitecture}',
    ),
  };
}

Map<String, String> _cargoEnvironment(
  CodeConfig config,
  String targetTriple,
) {
  if (config.targetOS != OS.android) {
    return const {};
  }

  final cCompiler = config.cCompiler;
  if (cCompiler == null) {
    throw UnsupportedError(
      'CCompilerConfig was not provided but is required for $targetTriple',
    );
  }

  final compilerPath = cCompiler.compiler.toFilePath();
  final compilerBinariesDir = Directory(compilerPath).parent.path;
  final targetTripleEnvVar = targetTriple.replaceAll('-', '_');
  final ndkTargetTriple = switch (targetTriple) {
    'armv7-linux-androideabi' => 'armv7a-linux-androideabi',
    _ => targetTriple,
  };
  final ndkSysrootTargetTriple = switch (targetTriple) {
    'armv7-linux-androideabi' => 'arm-linux-androideabi',
    _ => targetTriple,
  };

  String binaryPath(String binaryName, {String windowsSuffix = 'cmd'}) {
    final path = Platform.isWindows
        ? '$compilerBinariesDir/$binaryName.$windowsSuffix'
        : '$compilerBinariesDir/$binaryName';
    if (!File(path).existsSync()) {
      throw StateError('Binary $path not found; is your installed NDK too old?');
    }
    return path;
  }

  const apiTarget = '35';
  final clangPath = binaryPath('$ndkTargetTriple$apiTarget-clang');
  final clangPpPath = binaryPath('$ndkTargetTriple$apiTarget-clang++');
  final ranlibPath = binaryPath('llvm-ranlib', windowsSuffix: 'exe');

  final ndkToolchainRoot = Directory(clangPath).parent.parent.path;
  final sysroot = '$ndkToolchainRoot/sysroot';
  final extraInclude = '$sysroot/usr/include/$ndkSysrootTargetTriple';
  final bindgenClangArgs = '--sysroot=$sysroot -I$extraInclude'.replaceAll(
    r'\',
    '/',
  );

  return {
    'AR_$targetTripleEnvVar': cCompiler.archiver.toFilePath(),
    'CC_$targetTripleEnvVar': clangPath,
    'CXX_$targetTripleEnvVar': clangPpPath,
    'RANLIB_$targetTripleEnvVar': ranlibPath,
    'CARGO_TARGET_${targetTripleEnvVar.toUpperCase()}_LINKER': clangPath,
    'BINDGEN_EXTRA_CLANG_ARGS_$targetTripleEnvVar': bindgenClangArgs,
  };
}
