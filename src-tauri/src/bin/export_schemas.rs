use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("app")
        .join("generated")
        .join("schemas");

    worth_lib::contracts::schema_export::export_all(&output_dir)?;

    println!("Schema export complete ({}).", output_dir.display());
    Ok(())
}
