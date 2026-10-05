mod order;
mod rules;
mod source;

use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing workspace root")?;

    let errors = source::check(root)?;

    for message in &errors {
        eprintln!("{message}");
    }

    if !errors.is_empty() {
        return Err("Rust project style checks failed".into());
    }

    println!("Rust syntax and source style checks passed.");

    Ok(())
}
