use {
    log::LevelFilter,
    simplelog,
    std::{env, fs::File, str::FromStr},
};


/// configure the application log according to env variable.
pub fn configure_log(app_name: &str) {
    let env_var_name = format!("{}_LOG", app_name.to_ascii_uppercase());
    let level = env::var(&env_var_name).unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        let log_file_name = format!("{}.log", app_name);
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create(&log_file_name).expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting {} v{} with log level {}",
            app_name,
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}
