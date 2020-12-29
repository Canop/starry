use {
    crate::*,
    chrono::{DateTime, Utc},
};

/// a query for the time series for some
/// repos (maybe) and the sum (maybe) of a user
#[derive(Debug)]
pub(crate) struct UserQuery {
    pub user_id: UserId,
    pub sum: Option<Col>,
    pub repos: Vec<Col>,
}

#[derive(Debug)]
pub struct UserResponseLine {
    pub time: DateTime<Utc>,
    pub sum: usize,
    pub counts: Vec<Option<usize>>,
}
