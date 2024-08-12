use dedma::Config;
use std::{env, process};
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args);
    dedma::run(config).unwrap_or_else(|error|{
        println!("Application error : {error}");
        process::exit(1)
    })
}
