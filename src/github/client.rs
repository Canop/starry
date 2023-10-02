use {
    crate::*,
    anyhow::*,
    byo_graphql::{Count, GraphqlClient, List},
    chrono::{DateTime, Utc},
    serde::Deserialize,
};

pub static GITHUB_API_TOKEN_KEY: &str = "github-api-token";

pub struct GithubClient {
    gql_client: GraphqlClient,
}

impl GithubClient {
    pub fn new(conf: &Conf) -> Result<Self> {
        let github_api_token = conf.get(GITHUB_API_TOKEN_KEY).ok_or_else(|| {
            anyhow!(
                "You must first set a github API token with `starry set {} your-key`",
                GITHUB_API_TOKEN_KEY
            )
        })?;
        let mut gql_client = GraphqlClient::new("https://api.github.com/graphql")?;
        gql_client.set_bearer_auth(github_api_token);
        Ok(Self { gql_client })
    }
    /// get a GitHub user's information by its login
    pub fn get_user(&self, user_id: UserId) -> Result<User> {
        // we extract into a dedicated structure matching the graphql response
        #[derive(Deserialize)]
        pub struct GQUser {
            pub name: String,
            pub repositories: Count,
        }
        let query = format!(
            "{{ {} {{ name {}  }} }}",
            user_id.graphql_selector(),
            Count::query("repositories", "isFork: false"),
        );
        let gquser: GQUser = self.gql_client.get_first_item(query)?;
        Ok(User {
            user_id,
            name: gquser.name,
            non_fork_repositories_count: gquser.repositories.into(),
        })
    }
    /// query the GitHub API to get a UserObs which has the number of stars
    /// of all this user's repositories
    pub fn get_user_star_counts(&self, user_id: UserId, now: DateTime<Utc>) -> Result<UserObs> {
        #[derive(Deserialize)]
        pub struct User {
            pub repositories: Repositories,
        }
        #[derive(Debug, Deserialize)]
        pub struct Repository {
            pub name: String,
            pub stargazers: Count,
        }
        let mut counts = Vec::new();
        type Repositories = List<Repository>;
        // we'll do several requests if needed, using graphql pagination,
        // as the number of repositories of a user may exceed the tiny
        // capacity of a github graphql response
        let page_size = 100;
        let mut cursor: Option<String> = None;
        loop {
            let query = format!(
                "{{ {} {{ repositories{}{} }} }}",
                user_id.graphql_selector(),
                Repositories::query_page_selector(
                    &cursor,
                    page_size,
                    "isFork: false, ownerAffiliations: OWNER",
                ),
                Repositories::query_page_body("{ name, stargazers { totalCount } }"),
            );
            // println!("query: {}", &query);
            // println!("raw answer: {}", self.gql_client.text(&query)?);
            let mut user: User = self.gql_client.get_first_item(&query)?;
            for repo in user.repositories.nodes.drain(..) {
                counts.push(RepoObs {
                    repo_name: repo.name,
                    stars: repo.stargazers.into(),
                });
            }
            cursor = user.repositories.next_page_cursor();
            if cursor.is_none() {
                break;
            }
        }
        Ok(UserObs {
            user_id,
            time: now,
            counts,
        })
    }
}
