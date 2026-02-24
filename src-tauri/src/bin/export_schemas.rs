use anyhow::{Context, Result};
use schemars::schema_for;
use std::path::{Path, PathBuf};

use worth_lib::contracts::{AccountUpsertInput, InstitutionUpsertInput};

fn main() -> Result<()> {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("app")
        .join("generated")
        .join("schemas");

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("create schema output dir {}", output_dir.display()))?;

    write_schema::<InstitutionUpsertInput>(&output_dir, "InstitutionUpsertInput.schema.json")?;
    write_schema::<AccountUpsertInput>(&output_dir, "AccountUpsertInput.schema.json")?;

    println!("Schema export complete.");
    Ok(())
}

fn write_schema<T>(output_dir: &Path, file_name: &str) -> Result<()>
where
    T: schemars::JsonSchema,
{
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).context("serialize schema as JSON")?;
    let path = output_dir.join(file_name);
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("write schema {}", path.display()))?;
    Ok(())
}
