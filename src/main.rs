use cli_log::*;

fn main() -> anyhow::Result<()> {
    init_cli_log!();
    starry::run()?;
    info!("bye");
    Ok(())
}
