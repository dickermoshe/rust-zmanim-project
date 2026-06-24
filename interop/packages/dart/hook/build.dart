import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

const crateName = 'interop';

void main(List<String> args) async {
  await build(args, (input, output) async {
    final workspaceRoot = input.packageRoot.resolve('../../..');
    final cargo = await Process.run(
      'cargo',
      ['build', '-p', crateName],
      workingDirectory: workspaceRoot.toFilePath(),
    );

    if (cargo.exitCode != 0) {
      throw Exception(
        'Failed to build $crateName:\n${cargo.stderr}\n${cargo.stdout}',
      );
    }

    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'lib.g.dart',
        linkMode: DynamicLoadingBundled(),
        file: workspaceRoot.resolve(
          'target/debug/${input.config.code.targetOS.dylibFileName(crateName.replaceAll('-', '_'))}',
        ),
      ),
    );

    output.dependencies.add(workspaceRoot.resolve('interop/Cargo.toml'));
  });
}
