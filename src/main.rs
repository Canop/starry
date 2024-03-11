use cli_log::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_cli_log!();
    starry::run().await?;
    info!("bye");
    Ok(())
}
