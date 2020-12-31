use {
    crate::*,
    std::fmt,
};

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

impl UserId {
    /// check a user exists on github. Print some basic information
    /// if it's the case, print an error in other cases
    pub fn check_on_github(self, conf: &Conf) -> Result<bool> {
        let github_client = GithubClient::new(conf)?;
        match github_client.get_user(self) {
            Ok(user) => {
                println!(
                    "User {} has {} non forked repositories on GitHub",
                    user.name,
                    user.non_fork_repositories_count,
                );
                Ok(true)
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(false)
            }
        }
    }
}
