use {
    crate::*,
    std::fmt,
};

#[derive(Debug, Clone)]
pub struct RepoId {
    pub owner: UserId,
    pub name: String,
}

impl RepoId {
    pub fn new<S: Into<String>>(owner: UserId, name: S) -> Self {
        Self {
            owner,
            name: name.into(),
        }
    }
    pub fn graphql_selector(&self) -> String {
        format!(
            r#"repository(owner:"{}", name:"{}")"#,
            &self.owner, &self.name
        )
    }
}

impl fmt::Display for RepoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}
