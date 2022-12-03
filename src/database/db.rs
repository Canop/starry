use {
    crate::*,
    anyhow::{bail, Result},
    cli_log::*,
    chrono::{DateTime, Utc},
    crossbeam::{channel, thread},
    std::{
        fs,
        io::{self, Write},
        path::PathBuf,
    },
    termimad::{
        crossterm::{
            cursor,
            style::{style, Color, Print, PrintStyledContent, Stylize},
            terminal::{Clear, ClearType},
            queue,
        },
        ProgressBar,
    },
};

/// the database
#[derive(Debug)]
pub struct Db {
    /// where the DB is on disk
    pub dir: PathBuf,
    /// whether to tell everything when we work
    pub verbose: bool,
    /// whether to save on disk
    pub read_only: bool,
}

impl Db {
    pub fn new() -> Result<Self> {
        let dir = app_dirs()?.data_dir().to_path_buf();
        let verbose = false;
        let read_only = false;
        Ok(Self { dir, verbose, read_only })
    }
    pub fn user_stars_dir(&self, user_id: &UserId) -> PathBuf {
        self.dir.join("stars").join(user_id.to_string())
    }
    pub fn last_user_obs(&self, user_id: &UserId) -> Result<Option<UserObs>> {
        let user_dir = self.user_stars_dir(user_id);
        if !user_dir.exists() {
            return Ok(None);
        }
        // TODO: parsing all filenames as dates serves both as filter and
        // to get the most recent one, but it should be more efficient
        // to just look in order at the most recent one as given by
        // the metadata until one is valid
        fs::read_dir(user_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter_map(|path| {
                UserObs::filename_date(&path)
                    .map(|date| (path, date))
            })
            .max_by_key(|t| t.1)
            .map(|(path, date)| {
                UserObs::read_file(&path, user_id.clone(), date)
            })
            .transpose()
    }
    pub fn count_user_obs(&self, user_id: &UserId) -> Result<usize> {
        let user_dir = self.user_stars_dir(user_id);
        if !user_dir.exists() {
            return Ok(0);
        }
        // we currently don't check there's no extraneous file in the directory
        Ok(fs::read_dir(user_dir)?.count())
    }
    /// read in database the time serie made of
    /// the total numbers of stars of a user per date.
    /// Records are not sorted by date.
    pub fn extract_user(&self, user_id: &UserId) -> Result<Vec<DatedObs>> {
        let user_dir = self.user_stars_dir(user_id);
        if !user_dir.exists() {
            bail!("no data for user {}", user_id);
        }
        Ok(fs::read_dir(user_dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                UserObs::filename_date(&e.path())
                    .and_then(|date| UserObs::read_file(&e.path(), user_id.clone(), date).ok())
                    .map(|uo| uo.sum())
            })
            .collect())
    }
    /// fetches and return the (unordered) lines, one per time,
    /// for the given user query
    pub fn extract_user_query(
        &self,
        user_id: &UserId,
        repo_names: Vec<&str>,
    ) -> Result<Vec<UserResponseLine>> {
        let user_dir = self.user_stars_dir(user_id);
        debug!("user_dir: {:#?}", &user_dir);
        if !user_dir.exists() {
            bail!("no data for user {}", user_id);
        }
        let mut lines = Vec::new();
        for path in fs::read_dir(user_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
        {
            debug!("path: {:#?}", &path);
            if let Some(time) = UserObs::filename_date(&path) {
                debug!("reading file {:?}", path);
                //let uo = UserObs::read_file(&path, user_query.user_id.clone(), time)?;
                let (counts, sum) = UserObs::read_lines_and_sum(&path, &repo_names)?;
                lines.push(UserResponseLine { time, sum, counts });
            }
        }
        Ok(lines)
    }
    fn update_user(
        &self,
        user_id: &UserId,
        github_client: &GithubClient,
        now: DateTime<Utc>,
    ) -> Result<Vec<RepoChange>> {
        debug!("checking user {}", user_id);
        let user_dir = self.user_stars_dir(user_id);
        let user_obs = github_client.get_user_star_counts(user_id.clone(), now)?;
        let mut changes = Vec::new();
        if let Some(old_user_obs) = self.last_user_obs(user_id)? {
            changes.append(& mut user_obs.diff_from(&old_user_obs));
            if !changes.is_empty() {
                debug!("changes: {:#?}", &changes);
                if !self.read_only {
                    user_obs.write_in_dir(&user_dir, self.verbose)?;
                }
            }
        } else {
            debug!("{} enters the db", &user_id);
            if !self.read_only {
                user_obs.write_in_dir(&user_dir, self.verbose)?;
            }
        }
        Ok(changes)
    }
    pub fn update(
        &self,
        conf: &Conf,
        thread_count: usize,
    ) -> Result<Vec<RepoChange>> {
        let n = conf.watched_users.len();
        if n == 0 {
            eprintln!("No user followed. Use `starry follow some_name` to add one.");
            return Ok(vec![]);
        }
        print_progress(0, n)?;

        // we use the same date, so that it will look better in extracts
        let now = Utc::now();

        // a channel for the user_ids to process
        let (s_users, r_users) = channel::bounded(n);
        for user in &conf.watched_users {
            s_users.send(UserId::new(user)).unwrap();
        }

        // a channel to receive vecs of changes
        let (s_changes, r_changes) = channel::bounded(n);

        // a channel to receive progress
        let (s_progress, r_progress) = channel::bounded(n);

        // a thread to process the resuts and display progress
        std::thread::spawn(move || {
            let mut done = 0;
            while r_progress.recv().is_ok() {
                done += 1;
                print_progress(done, n).unwrap();
                if done == n { break; }
            }
        });

        // threads doing the heavy work
        thread::scope(|scope| {
            for _ in 0..thread_count {
                let r_users = r_users.clone();
                let s_changes = s_changes.clone();
                let s_progress = s_progress.clone();
                let github_client = match GithubClient::new(conf) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error while creating GitHub client: {:?}", e);
                        return;
                    }
                };
                scope.spawn(move |_| {
                    while let Ok(user_id) = r_users.try_recv() {
                        let changes = self
                            .update_user(&user_id, &github_client, now)
                            .unwrap_or_else(|e| {
                                eprintln!("error while updating user: {:?}", e);
                                Vec::new()
                            });
                        s_changes.send(changes).unwrap();
                        s_progress.send(()).unwrap();
                    }
                });
            }
        }).unwrap();

        let mut changes = Vec::new();
        for mut user_changes in r_changes.try_iter() {
            changes.append(& mut user_changes);
        }
        eprintln!("                                              ");
        println!("{} users queried from GitHub                   ", n);
        Ok(changes)
    }
}

fn print_progress(done: usize, total: usize) -> Result<()> {
    let width = 20;
    let p = ProgressBar::new(done as f32 / (total as f32), width);
    let s = format!("{:width$}", p, width=width);
    let mut stderr = io::stderr();
    queue!(stderr, cursor::SavePosition)?;
    queue!(stderr, Clear(ClearType::CurrentLine))?;
    queue!(stderr, Print(format!("{:>4} / {} users ", done, total)))?;
    queue!(stderr, PrintStyledContent(style(s).with(Color::Yellow).on(Color::DarkMagenta)))?;
    queue!(stderr, cursor::RestorePosition)?;
    stderr.flush()?;
    Ok(())
}
