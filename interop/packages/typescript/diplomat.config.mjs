import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.dirname(fileURLToPath(import.meta.url));
const profile = process.env.CARGO_PROFILE ?? "debug";

export default {
  wasm_path: path.join(
    packageRoot,
    "../../../target/wasm32-unknown-unknown",
    profile,
    "interop.wasm",
  ),
};
