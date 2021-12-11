use crate::github::{Contributor, PagedResult, Repository};
use futures::{stream, Stream, StreamExt};
use reqwest::{header::USER_AGENT, Client, Error};

const GITHUB_REST_SEARCH_API_URL: &str = "https://api.github.com";

pub struct GithubClient {
    auth_token: String,
    client: Client,
}

pub struct ContributionResult {
    pub repo: String,
    pub username: String,
    pub percentage: f32,
}

async fn get_repos_page<'a>(
    client: &Client,
    auth_token: &str,
    lang: &str,
    i: i32,
) -> Result<PagedResult<Repository>, Error> {
    let request_url = format!(
        "{}/{}/{}",
        GITHUB_REST_SEARCH_API_URL, "search", "repositories"
    );
    let lang_param = &format!("language:{}", lang)[..];
    let response = client
        .get(&request_url)
        .query(&[
            ("q", lang_param),
            ("sort", "stars"),
            ("o", "desc"),
            ("page", &i.to_string()),
        ])
        .header(USER_AGENT, "request")
        .bearer_auth(auth_token.to_string())
        .send()
        .await?;

    //println!("Calling for page: {}", response.status());

    let data: PagedResult<Repository> = response.json().await?;
    Ok(data)
}

async fn get_top_contributor_percentage(
    client: &Client,
    auth_token: &str,
    repo: Repository,
) -> ContributionResult {
    let request_url = format!(
        "{}/{}/{}/{}/contributors",
        GITHUB_REST_SEARCH_API_URL, "repos", repo.owner.login, repo.name
    );
    let response = client
        .get(&request_url)
        .query(&[
            ("sort", "contributions"),
            ("o", "desc"),
            ("per_page", &25.to_string()),
        ])
        .header(USER_AGENT, "request")
        .bearer_auth(auth_token.to_string())
        .send()
        .await
        .unwrap();

    //println!("Calling for 25 cotntributors Res: {}", response.status());

    let data: Vec<Contributor> = response.json().await.unwrap();
    let most_contribution = get_most_contributing_percentage(data).unwrap();
    ContributionResult {
        repo: repo.name.to_string(),
        username: most_contribution.0,
        percentage: most_contribution.1,
    }
}

fn get_most_contributing_percentage(mut data: Vec<Contributor>) -> Result<(String, f32), Error> {
    data.sort_by(|a, b| b.contributions.cmp(&a.contributions));
    let all_contrubutions: u32 = data.iter().map(|x| x.contributions).sum();
    let most_contributing_user_percent = data
        .iter()
        .map(|x| {
            (
                x.login.to_string(),
                (x.contributions as f32) / (all_contrubutions as f32),
            )
        })
        .next();
    Ok(most_contributing_user_percent.unwrap())
}

impl GithubClient {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            client: Client::new(),
        }
    }

    pub fn get_pages<'l>(&'l self, lang: &'l str) -> impl Stream<Item = Repository> + 'l {
        stream::iter(0..)
            .then(move |i| get_repos_page(&self.client, &self.auth_token, &lang, i))
            .map(|x| x.unwrap().items)
            .flat_map(|page| stream::iter(page))
    }

    pub fn get_contributors<'l>(
        &'l self,
        lang: &'l str,
        i: usize,
    ) -> impl Stream<Item = ContributionResult> + 'l {
        self.get_pages(&lang)
            .then(move |repo: Repository| {
                get_top_contributor_percentage(&self.client, &self.auth_token, repo)
            })
            .take(i)
    }
}
