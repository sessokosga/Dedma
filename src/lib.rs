#[allow(unstable)]
use git2::Repository;
use sqlx;
use std::env;
use std::error::Error;
use std::fs;

enum CommitSource {
    File(String),
    Git,
}

#[derive(Debug, PartialEq)]
struct ParsedLine {
    typee: String,
    title: String,
    content: String,
}

pub struct Config {
    output: String,
    source: CommitSource,
}

impl Config {
    pub fn build(args: &[String]) -> Config {
        let mut output = String::from("whats_new.md");
        let mut source = CommitSource::Git;
        if args.len() >= 3 {
            source = CommitSource::File(args[1].clone());
            output = args[2].clone();
        } else if args.len() >= 2 {
            output = args[1].clone();
        }
        println!("{output}");
        Config { output, source }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents;
    if let CommitSource::File(file) = config.source {
        println!("Gathering commits from '{file}'");
        contents = fs::read_to_string(file)?;
    } else {
        println!("Gathering commits from Git");
        let repo = Repository::open("./")?;
        contents = String::new();
    }
    println!("Parsing the commits");
    let parsed_text =  split_all(&contents);
    println!("{} commits found",parsed_text.len());

    println!("Writing the release note in '{}'", config.output);

    Ok(())
}

fn split_all(contents: &str) -> Vec<ParsedLine> {
    let mut res: Vec<ParsedLine> = Vec::new();
    for line in contents.lines() {
        res.push(split_one(line.trim()))
    }
    res
}

// fun parse(contents:&[String])->

fn split_one(line: &str) -> ParsedLine {
    let typee;
    let title;
    let content;
    let parts: Vec<&str> = line.split(':').collect();

    if parts[0].contains('(') {
        let sc: Vec<&str> = parts[0].split('(').collect();
        typee = sc[0].trim().to_string();
        title = sc[1].replace(')', " ").trim().to_string();
    } else {
        typee = parts[0].to_string();
        title = String::from("other");
    }
    content = parts[1].trim().to_string();

    ParsedLine {
        typee,
        title,
        content,
    }
}

/*
#[derive(Debug, sqlx::FromRow)]
struct Commit{
    id:i32,
    content:String
}

#[derive(Debug, sqlx::FromRow)]
struct Type{
    id:i32,
    name:String
}

#[derive(Debug, sqlx::FromRow)]
struct Title{
    id:i32,
    name:String
}

fn connect()->Result<(),Box<dyn Error>>{
    let pool = sqlx::sqlite::SqlitePool("sqlite:mydb.db").await?;
    Ok(())
}  */

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_one_work() {
        let contents = "update: Update export_game.yml";
        let pars = ParsedLine {
            typee: String::from("update"),
            title: String::from("other"),
            content: String::from("Update export_game.yml"),
        };
        assert_eq!(pars, split_one(contents));
    }

    #[test]
    fn split_all_work() {
        let contents = "\
        fix:Disabled the middleware
        update (ci):Update export_game.yml
        update:Updated github build action
        Fix (Opponents): blackboard for AI drops its content
        Fix (Audio): fmod plugin not working";
        let res = vec![
            ParsedLine {
                typee: String::from("fix"),
                title: String::from("other"),
                content: String::from("Disabled the middleware"),
            },
            ParsedLine {
                typee: String::from("update"),
                title: String::from("ci"),
                content: String::from("Update export_game.yml"),
            },
            ParsedLine {
                typee: String::from("update"),
                title: String::from("other"),
                content: String::from("Updated github build action"),
            },
            ParsedLine {
                typee: String::from("Fix"),
                title: String::from("Opponents"),
                content: String::from("blackboard for AI drops its content"),
            },
            ParsedLine {
                typee: String::from("Fix"),
                title: String::from("Audio"),
                content: String::from("fmod plugin not working"),
            },
        ];

        assert_eq!(res,split_all(contents))
    }
}
