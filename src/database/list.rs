use {
    crate::*,
    anyhow::*,
    serde::Serialize,
    std::{
        convert::From,
        io::Write,
    },
};

#[derive(Debug, Serialize)]
pub struct ListLine {
    pub name: String,
    pub stars: usize,
}

#[derive(Debug, Serialize)]
pub struct List {
    pub lines: Vec<ListLine>,
}

impl List {
    /// find the users with at least 2 user observations, return them
    /// with the last total count of stars
    pub fn users(db: &Db, conf: &Conf, drawable: bool) -> Result<Self> {
        let mut lines = Vec::new();
        for name in &conf.watched_users {
            let user_id = UserId::new(name);
            if drawable && db.count_user_obs(&user_id)? < 2 {
                continue;
            }
            if let Some(uo) = db.last_user_obs(&user_id)? {
                let stars = uo.sum().stars;
                lines.push(ListLine {
                    name: name.to_string(),
                    stars,
                });
            }
        }
        Ok(Self { lines })
    }
    pub fn write_csv<W: Write>(&self, w: &mut W) -> Result<()> {
        writeln!(w, "name,stars")?;
        for line in &self.lines {
            writeln!(w, "{},{}", line.name, line.stars)?;
        }
        w.flush()?;
        Ok(())
    }
}

impl From<UserObs> for List {
    fn from(mut uo: UserObs) -> Self {
        let lines = uo.counts
            .drain(..)
            .filter(|c| c.stars > 0)
            .map(|c| ListLine { name: c.repo_name, stars: c.stars })
            .collect();
        Self { lines }
    }
}
