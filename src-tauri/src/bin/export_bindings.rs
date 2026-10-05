use std::path::PathBuf;

use poprako_native_lib::bridge;
use specta_typescript::Typescript;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from("../src/bridge/generated/bindings.ts"),
        PathBuf::from,
    );

    bridge::builder().export(Typescript::default(), destination)?;

    Ok(())
}
