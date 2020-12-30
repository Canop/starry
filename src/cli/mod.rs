mod args;

pub use args::*;

use {
    crate::*,
    anyhow::*,
    argh,
    std::io,
};

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
            extract.write_csv(&mut io::stdout())?;
        }
        Some(ArgsCommand::Gaze { .. }) | None => {
            let mut db = Db::new()?;
            let mut changes = db.update(&conf)?;
            if changes.is_empty() {
                println!("no change");
            } else {
                println!("{} changes", changes.len());
                changes.sort_by(|a, b| b.value().partial_cmp(&a.value()).unwrap());
                for change in changes.iter().take(5) {
                    println!("{}", change);
                }
            }
        }
    }
    Ok(())
}
