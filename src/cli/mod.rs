mod args;

pub use args::*;

use {crate::*, anyhow::*, argh, chrono::Utc, std::io};

pub fn run() -> Result<()> {
    let args: Args = argh::from_env();
    debug!("args: {:#?}", &args);
    if args.version {
        println!("starry {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let mut conf = Conf::read()?;
    match args.command {
        Some(ArgsCommand::Set(SetCommand { name, value })) => {
            conf.set(name, value);
            conf.save()?;
        }
        Some(ArgsCommand::Get(GetCommand { name })) => match conf.get(&name) {
            Some(value) => {
                println!("{} = {:?}", name, value);
            }
            None => {
                println!("No value for {:?}", name);
            }
        },
        Some(ArgsCommand::Follow(FollowCommand { name })) => {
            conf.follow(name);
            conf.save()?;
        }
        Some(ArgsCommand::Unfollow(UnfollowCommand { name })) => {
            conf.unfollow(&name);
            conf.save()?;
        }
        Some(ArgsCommand::Extract(ExtractCommand { names })) => {
            let db = Db::new()?;
            let extract = Extract::read(&db, names)?;
            extract.write_csv(io::stdout())?;
        }
        Some(ArgsCommand::Gaze { .. }) | None => {
            if conf.watched_users.is_empty() {
                eprintln!("No user followed. Use `starry follow some_name` to add one.");
                return Ok(());
            }
            let db = Db::new()?;
            let github_client = GithubClient::new(&conf)?;
            // we use the same date, so that it will look better in extracts
            let now = Utc::now();
            for user in &conf.watched_users {
                let user_id = UserId::new(user);
                let user_dir = db.user_stars_dir(&user_id);
                let user_obs = github_client.get_user_star_counts(user_id, now)?;
                user_obs.write_in_dir(&user_dir)?;
            }
        }
    }
    Ok(())
}
