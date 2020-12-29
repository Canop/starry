use {
    crate::*,
    anyhow::*,
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
}
