use {
    crate::*,
    std::fmt,
};

#[derive(Debug)]
pub struct RepoChange {
    pub repo_id: RepoId,
    pub old_stars: Option<usize>,
    pub new_stars: usize,
}

impl fmt::Display for RepoChange {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        //write!(f, "[{}] ", self.value())?;
        write!(f, "{}", self.repo_id)?;
        if let Some(old_stars) = self.old_stars {
            if old_stars < self.new_stars {
                write!(f, " rised from {} to {}", old_stars, self.new_stars)?;
            } else {
                write!(f, " dropped from {} to {}", old_stars, self.new_stars)?;
            }
        } else if self.new_stars > 1 {
            write!(f, " is new and has already {} stars", self.new_stars)?;
        } else {
            write!(f, " is new")?;
        }
        write!(f, " - https://github.com/{}", self.repo_id)
    }
}

impl RepoChange {
    pub fn url(&self) -> String {
        format!("https://github.com/{}", self.repo_id)
    }
    pub fn value(&self) -> f64 {
        if let Some(old_stars) = self.old_stars {
            let o = old_stars as f64;
            let n = self.new_stars as f64;
            100f64 * (n - o) / (100f64 + o + n)
        } else {
            0.2f64 + (self.new_stars as f64) / 20f64
        }
    }
    /// how much this change is interesting
    pub fn interest(&self) -> f64 {
        self.value().abs()
    }
    pub fn trend_markdown(&self) -> &'static str {
        let value = self.value();
        if value > 3.0 {
            "`U` `U` `U`"
        } else if value > 1.0 {
            "`U` `U`"
        } else if value > 0.2 {
            "`U`"
        } else if value < -2.0 {
            "`D` `D` `D`"
        } else if value < -1.0 {
            "`D` `D`"
        } else if value < 0.0 {
            "`D`"
        } else {
            ""
        }
    }
}
