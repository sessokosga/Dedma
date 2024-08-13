use sqlx::sqlite::SqlitePool;
use sqlx::{self, Result, Row};
use std::error::Error;

use crate::ParsedLine;

#[derive(Debug, sqlx::FromRow)]
pub struct Commit {
    pub content: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Kind{
    pub kind:String
}

#[derive(Debug, sqlx::FromRow)]
pub struct Title{
    pub kind:String,
    pub title:String
}

pub async fn connect() -> Result<SqlitePool, Box<dyn Error>> {
    let pool = SqlitePool::connect("sqlite:mydb.db").await?;
    Ok(pool)
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

pub async fn get_titles(tag: &str, kind:&Kind, pool: &SqlitePool) -> anyhow::Result<Vec<Title>> {
    let titles: Vec<Title> = sqlx::query_as("SELECT DISTINCT kind, title FROM `Commit` WHERE tag = $1 AND kind = $2")
    .bind(tag)
    .bind(&kind.kind)
        .fetch_all(pool)
        .await?;
    Ok(titles)
}

pub async fn get_commits(tag: &str, title:&Title, pool: &SqlitePool) -> anyhow::Result<Vec<Commit>> {
    let titles: Vec<Commit> = sqlx::query_as("SELECT DISTINCT content FROM `Commit` WHERE tag = $1 AND kind = $2 AND title = $3")
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
) -> Result<i32, Box<dyn Error>> {
    let mut line_recorded = 0;
    for line in parsed_lines {
        let id = add_commit(tag, pool, line).await;
        match id {
            Ok(_) => line_recorded += 1,
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
