mod data_access;

use git2::Repository;
use sqlx::{self, Result};
use std::error::Error;
use std::fs;

enum CommitSource {
    File(String),
    Git,
}

#[derive(Debug, PartialEq)]
struct ParsedLine {
    kind: String,
    title: String,
    content: String,
    hash: String,
}

pub struct Config {
    output: String,
    source: CommitSource,
    tag: String,
}

impl Config {
    pub fn build(args: &[String]) -> Config {
        let mut output = String::from("whats_new.md");
        let mut source = CommitSource::Git;
        let tag = String::from("tag");
        if args.len() >= 3 {
            source = CommitSource::File(args[1].clone());
            output = args[2].clone();
        } else if args.len() >= 2 {
            output = args[1].clone();
        }
        println!("{output}");
        Config {
            output,
            source,
            tag,
        }
    }
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents;
    if let CommitSource::File(file) = config.source {
        println!("Gathering commits from '{file}'");
        contents = fs::read_to_string(file)?;
    } else {
        println!("Gathering commits from Git");
        let repo = Repository::open("./")?;
        contents = String::new();
    }
    // Parsing the commits
    let parsed_lines = split_all(&contents);
    println!("{} commits found", parsed_lines.len());

    // Recording them to the database
    let pool = data_access::connect().await?;
    let line_recorded = data_access::record_commits(&config.tag, &pool, parsed_lines).await?;
    println!("{} new lines recorded", line_recorded);
    // Writing the release note
    println!("Writing the release note in '{}'", config.output);
    let kinds = data_access::get_kinds(&config.tag, &pool).await?;
    let titles = data_access::get_titles(&config.tag, &kinds[1], &pool).await?;
    let commits = data_access::get_commits(&config.tag, &titles[0], &pool).await?;
    println!("Got {} commits from  title {} of kind {}",commits.len(),titles[0].title,kinds[1].kind);
    
    for kin in commits{
        println!("{}",kin.content);
    }

    Ok(())
}

fn split_all(contents: &str) -> Vec<ParsedLine> {
    let mut res: Vec<ParsedLine> = Vec::new();
    for line in contents.lines() {
        res.push(split_one(line.trim()))
    }
    res
}

fn split_one(line: &str) -> ParsedLine {
    let kind;
    let title;
    let content;
    let hash;
    let parts: Vec<&str> = line.split(':').collect();

    if parts[0].contains('(') {
        let sc: Vec<&str> = parts[0].split('(').collect();
        kind = sc[0].to_lowercase().trim().to_string();
        title = sc[1].replace(')', " ").trim().to_lowercase().to_string();
    } else {
        kind = parts[0].trim().to_lowercase().to_string();
        title = String::from("other");
    }
    content = parts[1].trim().to_string();
    hash = parts[2].trim().to_string();
    // hash = String::new();

    ParsedLine {
        kind,
        title,
        content,
        hash,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_one_work() {
        let contents =
            "feat (Reward): Added one more reward :13883a342dfe858a234d5366a855b49ddc0c534b";
        let pars = ParsedLine {
            kind: String::from("feat"),
            title: String::from("reward"),
            content: String::from("Added one more reward"),
            hash: String::from("13883a342dfe858a234d5366a855b49ddc0c534b"),
        };
        assert_eq!(pars, split_one(contents));
    }

    #[test]
    fn split_all_work() {
        let contents = "\
        feat (Reward): Added one more reward :13883a342dfe858a234d5366a855b49ddc0c534b
        feat (Reward): Added two more rewards :dd187eebf6321df5b541185dd0fd110b1b384712
        update: Added more balance to the game :9f0b66d57b97a33333681128f70396db7c2b3f53
        feat (Tank): added one tank type :06b9582c4a3a27a27e3a90c4444d8cc40ddf17e8
        fix (ci): fixed release notes path :478faab0a38cc5eb15b36915981ed538005dc9fb";
        let res = vec![
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("reward"),
                content: String::from("Added one more reward"),
                hash: String::from("13883a342dfe858a234d5366a855b49ddc0c534b"),
            },
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("reward"),
                content: String::from("Added two more rewards"),
                hash: String::from("dd187eebf6321df5b541185dd0fd110b1b384712"),
            },
            ParsedLine {
                kind: String::from("update"),
                title: String::from("other"),
                content: String::from("Added more balance to the game"),
                hash: String::from("9f0b66d57b97a33333681128f70396db7c2b3f53"),
            },
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("tank"),
                content: String::from("added one tank type"),
                hash: String::from("06b9582c4a3a27a27e3a90c4444d8cc40ddf17e8"),
            },
            ParsedLine {
                kind: String::from("fix"),
                title: String::from("ci"),
                content: String::from("fixed release notes path"),
                hash: String::from("478faab0a38cc5eb15b36915981ed538005dc9fb"),
            },
        ];

        assert_eq!(res, split_all(contents))
    }
}
