use std::error::Error;
use std::env;
use sqlx;

enum CommitSource {File(String),Git}

pub struct Config{
    output:String,
    source:CommitSource
}

impl Config{

    pub fn build(args:&[String])->Config{
        let mut output = String::from("whats_new.md");
        let mut source = CommitSource::Git;
        if args.len() >= 3{
            source = CommitSource::File(args[1].clone());
            output = args[2].clone();
        }else if args.len() >= 2{
            output = args[1].clone();
        }
        println!("{output}");
        Config { output,source }
        
    }
}

pub fn run(config:Config)->Result<(),Box<dyn Error>>{
    if let CommitSource::File(file) = config.source{
        println!("Gathering commits from '{file}'");
    }else{
        println!("Gathering commits from Git");
    }

    println!("Writing the release note in '{}'",config.output);

    Ok(())
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
} */