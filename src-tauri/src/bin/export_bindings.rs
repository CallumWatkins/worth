fn main() -> anyhow::Result<()> {
    worth_lib::api::export_bindings_to_app_generated()?;
    println!("Bindings export complete.");
    Ok(())
}
