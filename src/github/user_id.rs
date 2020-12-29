use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct UserId {
    pub login: String,
}

impl UserId {
    pub fn new<S: Into<String>>(login: S) -> Self {
        Self {
            login: login.into(),
        }
    }
    pub fn graphql_selector(&self) -> String {
        format!(r#"user(login:"{}")"#, &self.login)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.login)
    }
}
