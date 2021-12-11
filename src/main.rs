mod github_client;
mod github;

use std::env;

use clap::{App, Arg};
use futures::StreamExt;

use crate::github_client::GithubClient;

#[tokio::main]
async fn main() {
    let x = App::new("rust-rest-client-example")
                .version("1.0")
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
    // TODO handle bad input better                
    let lang = x.value_of("LANG").unwrap();
    let count = x.value_of("COUNT").unwrap().parse::<i32>().unwrap();
    let token =  env::var("gh_token").unwrap();
    
    let client = GithubClient::new(token.to_string());

    let mut res  = Box::pin(client.get_contributors(&lang, count as usize));
    while let Some(i) = res.next().await {
        println!("project: {0: <20} user: {1: <20} percentage: {2: <10}", i.repo, i.username, i.percentage);
    }
}
