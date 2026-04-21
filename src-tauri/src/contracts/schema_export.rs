use anyhow::{Context, Result};
use std::path::Path;

pub struct SchemaExport {
    pub type_name: &'static str,
    pub file_name: &'static str,
    pub schema_json: fn() -> Result<String>,
}

inventory::collect!(SchemaExport);

pub fn export_all(output_dir: &Path) -> Result<()> {
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)
            .with_context(|| format!("remove schema output dir {}", output_dir.display()))?;
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("create schema output dir {}", output_dir.display()))?;

    let mut exports: Vec<&SchemaExport> = inventory::iter::<SchemaExport>.into_iter().collect();
    exports.sort_unstable_by_key(|schema_export| schema_export.file_name);

    for schema_export in exports {
        let json = (schema_export.schema_json)()
            .with_context(|| format!("generate schema {}", schema_export.type_name))?;
        let path = output_dir.join(schema_export.file_name);
        std::fs::write(&path, json).with_context(|| format!("write schema {}", path.display()))?;
    }

    Ok(())
}
