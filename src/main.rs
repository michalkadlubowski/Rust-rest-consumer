mod github_client;
mod github;

use clap::{App, Arg};

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
                    .help("Sets programming language")
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

    // TODO token
    let token = "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";
    
    println!("Lang: {} count {}", lang, count);

    let client = GithubClient::new(token.to_string());
    let res  = client.get_contributors(&lang, count as usize).await;
    for i in &res {
        println!("project: {0: <20} user: {1: <20} percentage: {2: <10}", i.repo, i.username, i.percentage);
    }
}
