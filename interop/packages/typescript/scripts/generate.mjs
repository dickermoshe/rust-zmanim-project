import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const interopDir = path.join(packageRoot, "../..");
const libDir = path.join(packageRoot, "lib");

const result = spawnSync(
  "diplomat-tool",
  ["js", libDir, "--entry", "src/lib.rs"],
  {
    cwd: interopDir,
    stdio: "inherit",
    shell: true,
  },
);

process.exit(result.status ?? 1);
