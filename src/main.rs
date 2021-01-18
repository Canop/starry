#[macro_use] extern crate log;

fn main() -> anyhow::Result<()> {
    cli_log::init("starry");
    starry::run()?;
    info!("bye");
    Ok(())
}
