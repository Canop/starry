use {
    crate::*,
    anyhow::*,
    chrono::{DateTime, Utc},
    rayon::prelude::*,
    std::{fs, path::PathBuf},
};

/// the database
#[derive(Debug)]
pub struct Db {
    /// where the DB is on disk
    pub dir: PathBuf,
}

impl Db {
    pub fn new() -> Result<Self> {
        let dir = app_dirs()?.data_dir().to_path_buf();
        Ok(Self { dir })
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
        let user_dir = self.user_stars_dir(&user_id);
        let user_obs = github_client.get_user_star_counts(user_id.clone(), now)?;
        let mut changes = Vec::new();
        if let Some(old_user_obs) = self.last_user_obs(&user_id)? {
            changes.append(& mut user_obs.diff_from(&old_user_obs));
            if !changes.is_empty() {
                debug!("changes: {:#?}", &changes);
                user_obs.write_in_dir(&user_dir)?;
            }
        } else {
            debug!("{} enters the db", &user_id);
            user_obs.write_in_dir(&user_dir)?;
        }
        Ok(changes)
    }
    pub fn update(&mut self, conf: &Conf) -> Result<Vec<RepoChange>> {
        if conf.watched_users.is_empty() {
            eprintln!("No user followed. Use `starry follow some_name` to add one.");
            return Ok(vec![]);
        }
        println!("checking {} users...", conf.watched_users.len());
        let github_client = GithubClient::new(&conf)?;
        // we use the same date, so that it will look better in extracts
        let now = Utc::now();
        let changes = conf.watched_users.iter()
            .map(UserId::new)
            .par_bridge()
            .map(|user_id| self.update_user(&user_id, &github_client, now))
            .filter_map(|r| match r {
                Ok(changes) => Some(changes),
                Err(e) => {
                    eprintln!("error while updating user: {:?}", e);
                    None
                }
            })
            .reduce(
                Vec::new,
                |mut a, mut b| {
                    a.append(&mut b);
                    a
                }
            );
        Ok(changes)
    }
}
