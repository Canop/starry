mod args;

pub use args::*;

use {
    crate::*,
    anyhow::*,
    cli_log::*,
    std::io,
    termimad::crossterm::tty::IsTty,
};

pub async fn run() -> Result<()> {
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
            if !args.no_save {
                conf.save()?;
            }
        }
        Some(ArgsCommand::Get(GetCommand { name })) => match conf.get(&name) {
            Some(value) => {
                println!("{} = {:?}", name, value);
            }
            None => {
                println!("No value for {:?}", name);
            }
        },
        Some(ArgsCommand::Check(CheckCommand { name })) => {
            UserId::new(name).check_on_github(&conf).await?;
        }
        Some(ArgsCommand::Follow(FollowCommand { name })) => {
            if UserId::new(name.clone()).check_on_github(&conf).await? {
                conf.follow(name);
                if !args.no_save {
                    conf.save()?;
                }
            }
        }
        Some(ArgsCommand::Unfollow(UnfollowCommand { name })) => {
            conf.unfollow(&name);
            if !args.no_save {
                conf.save()?;
            }
        }
        Some(ArgsCommand::Extract(ExtractCommand { names })) => {
            let db = Db::new()?;
            let extract = Extract::read(&db, names)?;
            extract.write_csv(&mut io::stdout())?;
        }
        Some(ArgsCommand::List(ListCommand { login })) => {
            let db = Db::new()?;
            let list = match login {
                Some(login) => {
                    let uo = db.last_user_obs(&UserId::new(&login))?;
                    match uo {
                        Some(uo) => uo.into(),
                        None => bail!("no data for {:?}", login),
                    }
                }
                None => List::users(&db, &conf, false)?,
            };
            list.write_csv(&mut io::stdout())?;
        }
        Some(ArgsCommand::Gaze { .. }) | None => {
            let color = args
                .color
                .value()
                .unwrap_or_else(|| std::io::stdout().is_tty());
            let skin = make_skin(color);
            let mut db = Db::new()?;
            db.verbose = args.verbose;
            db.read_only = args.no_save;
            let mut changes = db.update(&conf).await?;
            changes.sort_by(|a, b| b.interest().partial_cmp(&a.interest()).unwrap());
            let report = ChangeReport::new(&changes, args.max_rows);
            report.print(&skin);
        }
    }
    Ok(())
}
