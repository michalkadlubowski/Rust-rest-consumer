use crate::github::{Contributor, PagedResult, Repository};
use futures::{stream, Stream, StreamExt};
use reqwest::{header::USER_AGENT, Client, Error};

const GITHUB_REST_SEARCH_API_URL: &str = "https://api.github.com";
const CONTRIBUTORS_COUNT_FOR_ANALYSIS: u8 = 25;

pub struct GithubClient {
    auth_token: String,
    client: Client,
}

pub struct ContributionResult {
    pub repo: String,
    pub username: String,
    pub percentage: f32,
}

fn calc_most_contributing_percentage(mut data: Vec<Contributor>) -> Result<(String, f32), Error> {
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

    async fn get_top_contributor_percentage(&self, repo: Repository) -> ContributionResult {
        let request_url = format!(
            "{}/{}/{}/{}/contributors",
            GITHUB_REST_SEARCH_API_URL, "repos", repo.owner.login, repo.name
        );
        let response = self
            .client
            .get(&request_url)
            .query(&[
                ("sort", "contributions"),
                ("o", "desc"),
                ("per_page", &CONTRIBUTORS_COUNT_FOR_ANALYSIS.to_string()),
            ])
            .header(USER_AGENT, "request")
            .bearer_auth(self.auth_token.to_string())
            .send()
            .await
            .unwrap();

        //println!("Calling for 25 cotntributors Res: {}", response.status());

        let data: Vec<Contributor> = response.json().await.unwrap();
        let most_contribution = calc_most_contributing_percentage(data).unwrap();
        ContributionResult {
            repo: repo.name.to_string(),
            username: most_contribution.0,
            percentage: most_contribution.1,
        }
    }

    async fn get_repos_page<'a>(&self, lang: &str, i: i32) -> Result<PagedResult<Repository>, Error> {
        let request_url = format!(
            "{}/{}/{}",
            GITHUB_REST_SEARCH_API_URL, "search", "repositories"
        );
        let lang_param = &format!("language:{}", lang)[..];
        let response = self
            .client
            .get(&request_url)
            .query(&[
                ("q", lang_param),
                ("sort", "stars"),
                ("o", "desc"),
                ("page", &i.to_string()),
            ])
            .header(USER_AGENT, "request")
            .bearer_auth(self.auth_token.to_string())
            .send()
            .await?;

        //println!("Calling for page: {}", response.status());

        let data: PagedResult<Repository> = response.json().await?;
        Ok(data)
    }

    pub fn get_pages<'l>(&'l self, lang: &'l str) -> impl Stream<Item = Repository> + 'l {
        stream::iter(0..)
            .then(move |i| self.get_repos_page(&lang, i))
            .map(|x| x.unwrap().items)
            .flat_map(|page| stream::iter(page))
    }

    pub fn get_contributors<'l>(&'l self, lang: &'l str, i: usize) -> impl Stream<Item = ContributionResult> + 'l {
        self.get_pages(&lang)
            .then(move |repo: Repository| self.get_top_contributor_percentage(repo))
            .take(i)
    }
}
