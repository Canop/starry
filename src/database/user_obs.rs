use {
    crate::*,
    anyhow::*,
    chrono::{DateTime, SecondsFormat, Utc},
    csv,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        ffi::OsStr,
        fs,
        path::Path,
    },
};

/// a user observation is what's stored as a file in database.
/// There's one per (user, time) and it contains the
/// counts of stars of all repos of that user on that time.
#[derive(Debug)]
pub struct UserObs {
    pub user_id: UserId,
    pub time: DateTime<Utc>,
    pub counts: Vec<RepoObs>,
}

/// a line in a csv file whose path already contains the user login and date
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepoObs {
    pub repo_name: String,
    pub stars: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatedObs {
    pub time: DateTime<Utc>,
    pub stars: usize,
}

impl UserObs {
    pub fn write_in_dir(&self, user_dir: &Path, verbose: bool) -> Result<()> {
        fs::create_dir_all(user_dir)?;
        // the chosen format, precise to the seconds only, avoids having
        // a dot in the name (which would break the naming with the ext)
        let file_path = user_dir
            .join(self.time.to_rfc3339_opts(SecondsFormat::Secs, true))
            .with_extension("csv");
        let mut w = csv::Writer::from_path(&file_path)?;
        for repo_obs in &self.counts {
            w.serialize(repo_obs)?;
        }
        w.flush()?;
        if verbose {
            println!("wrote file {:?}", file_path);
        }
        Ok(())
    }
    /// if the file has a name matching the userobs filename
    /// format (date.csv), then returns this date
    pub fn filename_date(path: &Path) -> Option<DateTime<Utc>> {
        path.extension()
            .and_then(OsStr::to_str)
            .filter(|&ext| ext == "csv")
            .and_then(|_| {
                path.file_stem()
                    .and_then(OsStr::to_str)
                    .and_then(|stem| DateTime::parse_from_rfc3339(stem).ok())
            })
            .map(DateTime::from)
    }
    pub fn read_file(file_path: &Path, user_id: UserId, time: DateTime<Utc>) -> Result<Self> {
        let mut r = csv::Reader::from_path(file_path)?;
        let mut counts = Vec::new();
        for obs in r.deserialize() {
            counts.push(obs?);
        }
        Ok(Self {
            user_id,
            time,
            counts,
        })
    }
    pub fn read_line(file_path: &Path, repo_name: &str) -> Result<Option<RepoObs>> {
        let mut r = csv::Reader::from_path(file_path)?;
        for repo_obs in r.deserialize() {
            let repo_obs: RepoObs = repo_obs?;
            if repo_obs.repo_name == repo_name {
                return Ok(Some(repo_obs));
            }
        }
        Ok(None)
    }
    pub fn read_lines_and_sum(
        file_path: &Path,
        repo_names: &[&str],
    ) -> Result<(Vec<Option<usize>>, usize)> {
        let mut r = csv::Reader::from_path(file_path)?;
        let mut map: HashMap<&str, usize> = HashMap::new();
        for (idx, name) in repo_names.iter().enumerate() {
            map.insert(name, idx);
        }
        let mut arr = vec![None; repo_names.len()];
        let mut sum = 0;
        for repo_obs in r.deserialize() {
            let repo_obs: RepoObs = repo_obs?;
            sum += repo_obs.stars;
            if let Some(idx) = map.get(&repo_obs.repo_name.as_str()) {
                arr[*idx] = Some(repo_obs.stars);
            }
        }
        Ok((arr, sum))
    }
    pub fn sum(&self) -> DatedObs {
        DatedObs {
            time: self.time,
            stars: self.counts.iter().map(|rc| rc.stars).sum(),
        }
    }
    pub fn repo_count(&self, repo_name: &str) -> Option<usize> {
        for repo_obs in &self.counts {
            if repo_obs.repo_name == repo_name {
                return Some(repo_obs.stars);
            }
        }
        None
    }
    pub fn diff_from(&self, old_uo: &Self) -> Vec<RepoChange> {
        let mut changes = Vec::new();
        for repo_obs in &self.counts {
            let old_stars = old_uo.repo_count(&repo_obs.repo_name);
            if old_stars == Some(repo_obs.stars) {
                continue;
            }
            changes.push(RepoChange {
                repo_id: RepoId::new(self.user_id.clone(), &repo_obs.repo_name),
                old_stars,
                new_stars: repo_obs.stars,
            });
        }
        changes
    }
}
