use git2::Repository;
use sqlx;
use sqlx::sqlite::SqlitePool;
use std::env;
use std::error::Error;
use std::fs;
use tokio;

enum CommitSource {
    File(String),
    Git,
}

#[derive(Debug, PartialEq)]
struct ParsedLine {
    kind: String,
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
    let parsed_lines =  split_all(&contents);
    println!("{} commits found",parsed_lines.len());

    // Recording them to the database 
    let pool = connect().await?;
    record_commits(&pool,parsed_lines).await?;
    // Writing the release note
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


fn split_one(line: &str) -> ParsedLine {
    let kind;
    let title;
    let content;
    let parts: Vec<&str> = line.split(':').collect();

    if parts[0].contains('(') {
        let sc: Vec<&str> = parts[0].split('(').collect();
        kind = sc[0].trim().to_string();
        title = sc[1].replace(')', " ").trim().to_string();
    } else {
        kind = parts[0].to_string();
        title = String::from("other");
    }
    content = parts[1].trim().to_string();

    ParsedLine {
        kind,
        title,
        content,
    }
}

#[derive(Debug, sqlx::FromRow)]
struct Commit{
    id:i32,
    kind:String,
    title:String,
    content:String,
    tag:String
}

pub async fn connect()->Result<SqlitePool,Box<dyn Error>>{
    let pool = SqlitePool::connect("sqlite:mydb.db").await?;
    Ok(pool)
}

async fn add_commit(pool:&SqlitePool,parsed_line:ParsedLine)->anyhow::Result<u64>{
    let mut conn = pool.acquire().await?;
    let id = sqlx::query("INSERT INTO `Commit` (content,kind,title,tag)
    VALUES($1,$2,$3,$4)")
    .bind(parsed_line.content)
    .bind(parsed_line.kind)
    .bind(parsed_line.title)
    .bind("tag")
    .execute(&mut *conn).await?.rows_affected();
    Ok(id)
}

async fn record_commits(pool:&SqlitePool,parsed_lines:Vec<ParsedLine>)->Result<(),Box<dyn Error>>{
   for line in parsed_lines{
    let id = add_commit(pool, line).await?;
    println!("added a commit with id {id}");
   }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_one_work() {
        let contents = "update: Update export_game.yml";
        let pars = ParsedLine {
            kind: String::from("update"),
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
                kind: String::from("fix"),
                title: String::from("other"),
                content: String::from("Disabled the middleware"),
            },
            ParsedLine {
                kind: String::from("update"),
                title: String::from("ci"),
                content: String::from("Update export_game.yml"),
            },
            ParsedLine {
                kind: String::from("update"),
                title: String::from("other"),
                content: String::from("Updated github build action"),
            },
            ParsedLine {
                kind: String::from("Fix"),
                title: String::from("Opponents"),
                content: String::from("blackboard for AI drops its content"),
            },
            ParsedLine {
                kind: String::from("Fix"),
                title: String::from("Audio"),
                content: String::from("fmod plugin not working"),
            },
        ];

        assert_eq!(res,split_all(contents))
    }
}
