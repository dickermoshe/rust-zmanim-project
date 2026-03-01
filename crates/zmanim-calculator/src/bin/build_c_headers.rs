#[cfg(feature = "c")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use interoptopus::Interop;
    use interoptopus_backend_c::{Config, Generator};
    use std::fs;
    use std::path::Path;

    let out_dir = Path::new("bindings").join("c");
    fs::create_dir_all(&out_dir)?;
    let out_file = out_dir.join("zmanim_calendar.h");

    Generator::new(
        Config {
            ifndef: "zmanim_calendar_h".to_string(),
            ..Config::default()
        },
        zmanim_calculator::c_api::my_inventory(),
    )
    .write_file(out_file)?;

    Ok(())
}

#[cfg(not(feature = "c"))]
fn main() {
    eprintln!("Enable the C feature: cargo run --features c --bin build_c_headers");
    std::process::exit(1);
}
