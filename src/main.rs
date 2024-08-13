use dedma::Config;
use std::{env, process};
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args);

    let _ = dedma::run(config).await.unwrap_or_else(|error| {
        println!("Application error : {error}");
        process::exit(1)
    });
}
