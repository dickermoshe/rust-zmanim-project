#![allow(missing_docs, clippy::expect_used, clippy::panic)]

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../tools/dsl.py");
    println!("cargo:rerun-if-changed=../tools/generate-interop.py");

    let tools_dir = std::path::Path::new("..").join("tools");
    let status = Command::new("uv")
        .args(["run", "python", "generate-interop.py"])
        .current_dir(tools_dir)
        .status()
        .expect("failed to spawn uv for interop codegen");

    if !status.success() {
        panic!("generate-interop.py failed");
    }
}
