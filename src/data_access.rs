use indicatif::ProgressBar;
use sqlx::{self,sqlite::SqlitePool};
use std::fs::{DirBuilder, File};
use crate::ParsedLine;

#[derive(Debug, sqlx::FromRow)]
pub struct Commit {
    pub content: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Kind {
    pub kind: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Title {
    pub kind: String,
    pub title: String,
}

async fn create_database(pool:&SqlitePool)->anyhow::Result<()> {
    sqlx::query("\
        BEGIN TRANSACTION;
        CREATE TABLE IF NOT EXISTS \"Commit\" (
            \"id\"	INTEGER,
            \"content\"	TEXT NOT NULL,
            \"kind\"	TEXT NOT NULL,
            \"title\"	TEXT NOT NULL,
            \"tag\"	TEXT NOT NULL,
            \"hash\"	INTEGER NOT NULL UNIQUE,
            PRIMARY KEY(\"id\" AUTOINCREMENT)
        COMMIT;").execute(pool).await?;
    Ok(())
}

pub async fn connect() -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite:./.dedma/dedma_db.db").await;
    match pool {
        Ok(pool) => {return  Ok(pool);}
        Err(_) => {
            DirBuilder::new().create("./.dedma")?;
            File::create("./.dedma/dedma_db.db")?;
            let pool = SqlitePool::connect("sqlite:./.dedma/dedma_db.db").await;
            match pool {
                Ok(pool) =>{
                    create_database(&pool).await?;
                    return Ok(pool);
                } 
                Err(error) => {
                    panic!("Error creating database: {error}")
                },
            }
        }
    }
}

pub async fn connect_test()-> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite:./tests/test_db.db").await;
    match pool {
        Ok(pool) => {return  Ok(pool);}
        Err(_) => {
            let _ = DirBuilder::new().create("./tests");
            let _ = File::create("./tests/test_db.db");
            let pool = SqlitePool::connect("sqlite:./tests/test_db.db").await;
            match pool {
                Ok(pool) =>{
                    create_database(&pool).await?;
                    return Ok(pool);
                } 
                Err(error) => {
                    panic!("Error creating database: {error}")
                },
            }
        }
    }
}

pub async fn add_commit(
    tag: &str,
    pool: &SqlitePool,
    parsed_line: ParsedLine,
) -> anyhow::Result<u64> {
    let mut conn = pool.acquire().await?;
    let id = sqlx::query(
        "INSERT INTO `Commit` (content,kind,title,tag,hash)
    VALUES($1,$2,$3,$4,$5)",
    )
    .bind(parsed_line.content)
    .bind(parsed_line.kind)
    .bind(parsed_line.title)
    .bind(tag)
    .bind(parsed_line.hash)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(id)
}

pub async fn get_kinds(tag: &str, pool: &SqlitePool) -> anyhow::Result<Vec<Kind>> {
    let kinds: Vec<Kind> = sqlx::query_as("SELECT DISTINCT kind FROM `Commit` WHERE tag = $1")
        .bind(tag)
        .fetch_all(pool)
        .await?;
    Ok(kinds)
}

pub async fn get_titles(tag: &str, kind: &str, pool: &SqlitePool) -> anyhow::Result<Vec<Title>> {
    let titles: Vec<Title> =
        sqlx::query_as("SELECT DISTINCT kind, title FROM `Commit` WHERE tag = $1 AND kind = $2")
            .bind(tag)
            .bind(kind)
            .fetch_all(pool)
            .await?;
    Ok(titles)
}

pub async fn get_commits(
    tag: &str,
    title: &Title,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<Commit>> {
    let titles: Vec<Commit> = sqlx::query_as(
        "SELECT DISTINCT content FROM `Commit` WHERE tag = $1 AND kind = $2 AND title = $3",
    )
    .bind(tag)
    .bind(&title.kind)
    .bind(&title.title)
    .fetch_all(pool)
    .await?;
    Ok(titles)
}

pub async fn record_commits(
    tag: &str,
    pool: &SqlitePool,
    parsed_lines: Vec<ParsedLine>,
    progress:Option<&ProgressBar>
) -> anyhow::Result<i32> {
    let mut line_recorded = 0;
    for line in parsed_lines {
        let id = add_commit(tag, pool, line).await;
        match id {
            Ok(_) => {
                line_recorded += 1;
                if let Some(p) = progress{
                    p.inc(1);
                }
            }
            Err(error) => {
                if !error
                    .to_string()
                    .contains("(code: 2067) UNIQUE constraint failed")
                {
                    panic!("{}", error);
                }
            }
        }
    }

    Ok(line_recorded)
}
