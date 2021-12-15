mod github_client;
mod github;

use std::env;

use clap::{App, Arg};
use futures::StreamExt;

use crate::github_client::GithubClient;

#[tokio::main]
async fn main() {
    let arg_matches = App::new("rust-rest-client-example")
                .version("1.0")
                .after_help("USE \"gh_token\" ENV VARIABLE TO PASS GITHUB ACCESS TOKEN")
                .author("Michał Kadłubowski")
                .arg(Arg::with_name("LANG")
                    .short("l")
                    .long("language")
                    .required(true)
                    .help("Programming language")
                    .takes_value(true))
                .arg(Arg::with_name("COUNT")
                    .short("c")
                    .long("project_count")
                    .help("Projects count")
                    .required(true)
                    .takes_value(true))
                .get_matches();

    // Validated by clap so unwrap is mostly ok
    let lang = arg_matches.value_of("LANG").unwrap();
    let count = arg_matches.value_of("COUNT").unwrap().parse::<usize>().unwrap();
    let token =  env::var("gh_token").ok();
    
    let client = GithubClient::new(token);

    let mut res  = Box::pin(client.get_contributors(&lang, count));
    while let Some(i) = res.next().await {
        match i {
            Ok(result ) => println!("project: {0: <20} user: {1: <20} percentage: {2: <10}", result.repo, result.username, result.percentage),
            Err(api_error ) => {
                println!("Something failed - {}", api_error.0);
                break;
            }
        }
    }
}
