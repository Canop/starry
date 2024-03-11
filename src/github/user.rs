use crate::*;

#[derive(Debug)]
pub struct User {
    pub user_id: UserId,
    pub name: String,
    pub non_fork_repositories_count: usize,
}
