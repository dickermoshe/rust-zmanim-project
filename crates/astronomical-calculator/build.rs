//! Build script for the suncalc crate.
//!
//! This script conditionally compiles the SPA (Solar Position Algorithm) C library
//! when the `__spa-sys` feature is enabled, which is used for testing purposes only.
fn main() {
    // Only build the SPA C library when the __spa-sys feature is enabled
    #[cfg(feature = "__spa-sys")]
    {
        use std::env;
        use std::path::PathBuf;

        #[allow(clippy::panic, clippy::manual_assert)]
        if env::var("ALLOW_SPA_SYS_BUILD").map(|v| v != "1").unwrap_or(false) {
            panic!("The __spa-sys feature is only for testing purposes");
        }

        // Fallback: compile from source
        println!("cargo:warning=Pre-built library not found in bin/, compiling from source");
        cc::Build::new().file("c_src/spa.c").include("c_src").compile("spa");
        println!("cargo:rerun-if-changed=c_src/spa.c");

        // Generate bindings for the SPA library
        #[allow(clippy::expect_used)]
        let bindings = bindgen::Builder::default()
            .header("c_src/spa.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file
        #[allow(clippy::expect_used)]
        let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable is not set"));
        #[allow(clippy::expect_used)]
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");

        println!("cargo:rerun-if-changed=c_src/spa.h");
    }
}
