use std::env;
use dedma::Config;
fn main() {
    let args :Vec<String> = env::args().collect();
    let config = Config::build(&args);
    dedma::run(config);
}
