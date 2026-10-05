use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::source::check;

#[test]
fn build_scripts_and_modules_receive_every_source_check() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;

    for file in [
        "build.rs",
        "build_icon.rs",
        "sql-prepare/build.rs",
        "style-check/build.rs",
        "helper/asset.rs",
    ] {
        let path = directory
            .path()
            .join(file)
            .components()
            .collect::<PathBuf>();

        fs::create_dir_all(path.parent().ok_or("source file has no parent")?)?;

        fs::write(&path, "pub(crate) struct Forbidden;\n")?;

        let errors = check(directory.path())?;

        assert!(errors.iter().any(|error| {
            error.contains(path.to_string_lossy().as_ref()) && error.contains("scoped visibility")
        }));

        fs::write(&path, "// source fixture\n".repeat(401))?;

        let errors = check(directory.path())?;

        assert!(errors.iter().any(|error| {
            error.contains(path.to_string_lossy().as_ref()) && error.contains("401 physical lines")
        }));

        fs::write(&path, "// source fixture\n".repeat(400))?;

        assert!(check(directory.path())?.is_empty());
    }

    Ok(())
}

#[test]
fn generated_trees_are_excluded_without_excluding_handwritten_siblings()
-> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;

    for file in [
        "target/output.rs",
        "gen/output.rs",
        "generated/output.rs",
        "helper/target/output.rs",
    ] {
        let path = directory.path().join(file);

        fs::create_dir_all(path.parent().ok_or("source file has no parent")?)?;

        fs::write(path, "this is generated Rust that cannot be parsed")?;
    }

    assert!(check(directory.path())?.is_empty());

    fs::write(directory.path().join("build_icon.rs"), "fn valid() {}")?;

    assert!(check(directory.path())?.is_empty());

    fs::create_dir_all(directory.path().join("helper/src"))?;

    fs::write(directory.path().join("helper/src/mod.rs"), "fn valid() {}")?;

    assert!(
        check(directory.path())?
            .iter()
            .any(|error| error.contains("use sibling module file"))
    );

    Ok(())
}
