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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repo_id)?;
        if let Some(old_stars) = self.old_stars {
            if old_stars < self.new_stars {
                write!(f, " rised from {} to {}", old_stars, self.new_stars)
            } else {
                write!(f, " dropped from {} to {}", old_stars, self.new_stars)
            }
        } else if self.new_stars > 1 {
            write!(f, " is new and has already {} stars", self.new_stars)
        } else {
            write!(f, " is new")
        }
    }
}
