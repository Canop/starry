#[macro_use]
extern crate log;

mod app;
mod cli;
mod conf;
mod database;
mod github;

pub use {app::*, conf::*, database::*, github::*};

use {
    anyhow::*,
    log::LevelFilter,
    simplelog,
    std::{env, fs::File, str::FromStr},
};

/// configure the application log according to env variable.
fn configure_log() {
    let level = env::var("STARRY_LOG").unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("starry.log").expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting starry v{} with log level {}",
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}

fn main() -> Result<()> {
    configure_log();
    cli::run()?;
    info!("bye");
    Ok(())
}
