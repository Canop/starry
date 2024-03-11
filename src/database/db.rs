use {
    crate::*,
    anyhow::{
        bail,
        Result,
    },
    chrono::Utc,
    cli_log::*,
    futures::stream::{
        self,
        StreamExt,
    },
    std::{
        fs,
        path::PathBuf,
        sync::Arc,
    },
    tokio::sync::Mutex,
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
        Ok(Self {
            dir,
            verbose,
            read_only,
        })
    }
    pub fn user_stars_dir(
        &self,
        user_id: &UserId,
    ) -> PathBuf {
        self.dir.join("stars").join(user_id.to_string())
    }
    pub fn last_user_obs(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserObs>> {
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
            .filter_map(|path| UserObs::filename_date(&path).map(|date| (path, date)))
            .max_by_key(|t| t.1)
            .map(|(path, date)| UserObs::read_file(&path, user_id.clone(), date))
            .transpose()
    }
    pub fn count_user_obs(
        &self,
        user_id: &UserId,
    ) -> Result<usize> {
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
    pub fn extract_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<DatedObs>> {
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

    pub async fn update(
        &self,
        conf: &Conf,
    ) -> Result<Vec<RepoChange>> {
        let n = conf.watched_users.len();
        if n == 0 {
            eprintln!("No user followed. Use `starry follow some_name` to add one.");
            return Ok(vec![]);
        }
        let task = Arc::new(Mutex::new(
            Task::new(format!("Query {n} users")).with_total(n),
        ));

        // we use the same date, so that it will look better in extracts
        let now = Utc::now();

        let results = stream::iter(conf.watched_users.clone())
            .map(|user| {
                let github_client = GithubClient::new(conf).unwrap();
                let task = task.clone();
                let user_id = UserId::new(user);
                tokio::spawn(async move {
                    let user_obs = github_client
                        .get_user_star_counts(user_id.clone(), now)
                        .await;
                    task.lock().await.increment();
                    user_obs.map(|user_obs| (user_id, user_obs))
                })
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await;
        let mut changes = Vec::new();
        for result in results.into_iter() {
            match result {
                Ok(Ok((user_id, user_obs))) => {
                    let user_dir = self.user_stars_dir(&user_id);
                    if let Some(old_user_obs) = self.last_user_obs(&user_id)? {
                        let mut diff = user_obs.diff_from(&old_user_obs);
                        if !diff.is_empty() {
                            changes.append(&mut diff);
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
                }
                e => {
                    eprintln!("Error reading user changes: {:?}", e);
                }
            }
        }
        task.lock()
            .await
            .finish(format!("Found {} changes", changes.len()));
        Ok(changes)
    }
}
