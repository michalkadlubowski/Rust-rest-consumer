use std::fmt;

use crate::github::{Contributor, PagedResult, Repository};
use futures::{stream, Stream, StreamExt, TryStreamExt};
use reqwest::{header::USER_AGENT, Client, RequestBuilder, StatusCode};

const GITHUB_REST_SEARCH_API_URL: &str = "https://api.github.com";
const CONTRIBUTORS_COUNT_FOR_ANALYSIS: u8 = 25;

#[derive(Debug)]
pub struct ApiError(pub String);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error - {}", self.0)
    }
}

pub struct GithubClient {
    auth_token: Option<String>,
    client: Client,
}

pub struct ContributionResult {
    pub repo: String,
    pub username: String,
    pub percentage: f32,
}

impl GithubClient {
    pub fn new(auth_token: Option<String>) -> Self {
        Self {
            auth_token,
            client: Client::new(),
        }
    }

    fn create_get_request_builder(&self, url: &str) -> RequestBuilder {
        let builder = match &self.auth_token {
            Some(token) => self.client.get(url).bearer_auth(token),
            None => self.client.get(url),
        };
        builder.header(USER_AGENT, "request")
    }

    fn calc_most_contributing_percentage(mut data: Vec<Contributor>) -> Option<(String, f32)> {
        if data.len() == 0 {
            return None;
        }

        data.sort_by(|a, b| b.contributions.cmp(&a.contributions));
        let all_contrubutions: u32 = data.iter().map(|x| x.contributions).sum();
        Some((
            data[0].login.to_string(),
            (data[0].contributions as f32) / (all_contrubutions as f32),
        ))
    }

    async fn get_top_contributor_percentage(&self, repo: Repository) -> Result<ContributionResult, ApiError> {
        let request_url = format!(
            "{}/{}/{}/{}/contributors",
            GITHUB_REST_SEARCH_API_URL, "repos", repo.owner.login, repo.name
        );
        let response = self
            .create_get_request_builder(&request_url)
            .query(&[
                ("sort", "contributions"),
                ("o", "desc"),
                ("per_page", &CONTRIBUTORS_COUNT_FOR_ANALYSIS.to_string()),
            ])
            .send()
            .await
            .map_err(|_x| ApiError("Error calling API".to_owned()))?;

        // println!("Calling for 25 cotntributors Res: {}", response.status());

        if response.status() != StatusCode::OK {
            return Err(ApiError(format!(
                "Error calling API - response code: {}",
                response.status()
            )));
        }
        let data: Vec<Contributor> = response
            .json()
            .await
            .map_err(|_x| ApiError("Error deserializing contributors data".to_owned()))?;
        let most_contribution =
            Self::calc_most_contributing_percentage(data).unwrap_or(("N/A".to_owned(), 0 as f32));
        Ok(ContributionResult {
            repo: repo.name.to_owned(),
            username: most_contribution.0,
            percentage: most_contribution.1,
        })
    }

    async fn get_repos_page<'a>(&self, lang: &str, i: i32) -> Result<PagedResult<Repository>, ApiError> {
        let request_url = format!(
            "{}/{}/{}",
            GITHUB_REST_SEARCH_API_URL, "search", "repositories"
        );
        let lang_param = &format!("language:{}", lang)[..];
        let response = self
            .create_get_request_builder(&request_url)
            .query(&[
                ("q", lang_param),
                ("sort", "stars"),
                ("o", "desc"),
                ("page", &i.to_string()),
            ])
            .send()
            .await
            .map_err(|_x| ApiError("Error calling API".to_owned()))?;

        //println!("Calling for page: {}", response.status());
        
        if response.status() != StatusCode::OK {
            return Err(ApiError(format!(
                "Error calling API - response code: {}",
                response.status()
            )));
        }

        let data: PagedResult<Repository> = response
            .json()
            .await
            .map_err(|_x| ApiError("Error deserializing repos data".to_owned()))?;
        Ok(data)
    }

    pub fn get_pages<'l>(&'l self, lang: &'l str) -> impl Stream<Item = Result<Repository, ApiError>> + 'l {
        stream::iter(0..)
            .then(move |i| self.get_repos_page(&lang, i))
            .map_ok(|x| stream::iter(x.items.into_iter().map(Ok)))
            .try_flatten()
    }

    pub fn get_contributors<'l>(&'l self, lang: &'l str, project_count: usize) -> impl Stream<Item = Result<ContributionResult, ApiError>> + 'l {
        self.get_pages(&lang)
            .and_then(move |repo: Repository| self.get_top_contributor_percentage(repo))
            .take(project_count)
    }
}
