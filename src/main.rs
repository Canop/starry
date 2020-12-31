#[macro_use]
extern crate log;

mod app;
mod cli;
mod conf;
mod database;
mod github;

pub use {
    app::*,
    conf::*,
    database::*,
    github::*,
};

use {
    anyhow::*,
};

fn main() -> Result<()> {
    cli_log::init("starry");
    cli::run()?;
    info!("bye");
    Ok(())
}
