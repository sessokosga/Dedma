mod data_access;

use anyhow::Ok;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::vec;
use std::{fs,process::Command};

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
        // println!("{output}");
        Config {
            output,
            source,
            tag,
        }
    }
}

fn get_tag()->anyhow::Result<(String,String)>{
    let tags = Command::new("git")
        .arg("tag")
        .arg("--sort=-v:refname")
        .output()?;
    
    let tags = String::from_utf8_lossy(&tags.stdout);
    if tags.len()<=0{
      return Ok(("no_tag".to_string(),"no_tag".to_string()))
    }

    let mut vers: Vec<String> = vec![];

    let max = if tags.len() >= 2 { 2 } else { 1 };

    for tag in tags.lines() {
        if vers.len() >= max {
            break;
        } else {
            vers.push(tag.to_string());
        }
    }
    if max == 2{
        Ok((vers[1].clone(),vers[0].clone()))
    }else{
        Ok((vers[0].clone(),vers[0].clone()))
    }
}

fn read_from_git()->anyhow::Result<String>{
    let tags = get_tag()?;

    let commits;
    // print!("git log ");
    if tags.0 != tags.1 {
        let args = format!("{}..{}",tags.0,tags.1);
        commits = Command::new("git")
        .arg("log")
        .arg(&args)
        .arg("--format=%s :%H")
        .output()?;
        
        // print!("{} ",&args);

     
} else {
        commits = Command::new("git")
        .arg("log")
        .arg("--format=%s :%H")
        .output()?;
    }
    // println!("--format=\"%s :%H\"");

    Ok(String::from_utf8_lossy(&commits.stdout).to_string())

}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let contents;
    let tag;
    if let CommitSource::File(file) = config.source {
        println!("Gathering commits from '{file}...'");
        contents = fs::read_to_string(file)?;
        tag = config.tag;
    } else {
        println!("Gathering commits from Git repository...");
        contents = read_from_git()?;
        let tagi = get_tag()?;
        tag = tagi.1;
    }

    // Parsing the commits
    let parsed_lines = split_all(&contents);
    println!("{} commits found", parsed_lines.len());

    // Recording them to the database
    let pool = data_access::connect().await?;
    let _ = data_access::record_commits(&tag, &pool, parsed_lines).await?;
    // println!("{} new lines recorded", line_recorded);
    // Writing the release note
    println!("Writing the release note in '{}'...", config.output);
    let notes = generate_release_notes(&tag, &pool).await?;
    // println!("{}",notes);
    let _ = write_release_note(&config.output, notes)?;

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

fn beautify_kind(kind: &str) -> anyhow::Result<&str> {
    let result;
    let kinds: HashMap<&str, &str> = HashMap::from([
        ("feat", "New features"),
        ("fix", "Bug fix"),
        ("chore", "Chore"),
        ("refactor", "Refactoring"),
        ("docs", "Documentation"),
        ("style", "Code Style"),
        ("test", "Test"),
        ("perf", "Performances"),
        ("ci", "Continuous Integration (CI)"),
        ("build", "Build System"),
        ("revert", "Reverts"),
        ("update", "Updates"),
    ]);
    if kinds.contains_key(kind) {
        result = kinds[kind];
    } else {
        panic!("kind {kind} not found");
    }
    Ok(result)
}

fn beautify_title(title: &str) -> String {
    let result;
    if title.len() <= 3 {
        result = title.to_uppercase();
    } else {
        let cap = title.to_uppercase();
        let mut res = String::from(&cap[0..1].to_string());
        res.push_str(&title[1..]);
        result = res;
    }
    result
}

async fn generate_release_notes(tag: &str, pool: &SqlitePool) -> anyhow::Result<String> {
    let order = vec![
        "feat".to_string(),
        "fix".to_string(),
        "update".to_string(),
        "chore".to_string(),
        "refactor".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "test".to_string(),
        "perf".to_string(),
        "ci".to_string(),
        "build".to_string(),
        "revert".to_string(),
    ];
    let mut notes = String::new();
    let ki = data_access::get_kinds(tag, &pool).await?;
    let mut kinds: Vec<String> = vec![];
    for k in ki {
        kinds.push(k.kind);
    }

    for kind in order {
        if !kinds.contains(&kind) {
            continue;
        }
        notes.push_str(&format!("# {}\n", beautify_kind(&kind)?));

        let titles = data_access::get_titles(tag, &kind, &pool).await?;
        for title in &titles {
            if title.title != String::from("other") {
                notes.push_str(&format!("## {}\n", beautify_title(&title.title)));
            }

            let commits = data_access::get_commits(tag, title, &pool).await?;
            for commit in &commits {
                notes.push_str(&format!("- {}\n", commit.content));
            }
        }
    }

    Ok(notes)
}

fn write_release_note(file_path: &str, notes: String) -> anyhow::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_fmt(format_args!("{}", notes))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::result::Result::Ok;
    use tokio;

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

    async fn run_test(contents: &str) -> anyhow::Result<String> {
        let tag = "tag";
        let result;
        let parsed_lines = split_all(&contents);
        let pool = data_access::connect_test().await?;
        let _ = data_access::record_commits(tag, &pool, parsed_lines).await?;
        result = generate_release_notes(tag, &pool).await?;
        Ok(result)
    }

    #[tokio::test]
    // #[ignore = "need to find a way to run async tests"]
    async fn release_notes() {
        let contents = fs::read_to_string("./tests/logs.txt").expect("check input file");

        let notes = fs::read_to_string("./tests/parsed.md").expect("check output file");

        let res = run_test(&contents).await;
        let result;
        match res {
            Ok(res) => result = res,
            Err(error) => {
                println!("error while testing {error}");
                result = String::new()
            }
        }

        assert_eq!(result, notes)
    }
}
