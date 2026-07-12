import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const profile = process.env.CARGO_PROFILE ?? "debug";

const args = ["build", "-p", "interop", "--target", "wasm32-unknown-unknown"];
if (profile === "release") {
  args.push("--release");
}

const result = spawnSync("cargo", args, {
  cwd: workspaceRoot,
  stdio: "inherit",
  shell: true,
});

process.exit(result.status ?? 1);
