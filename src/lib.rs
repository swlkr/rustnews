use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::DateTime;
pub use ryzz::*;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] ryzz::Error),
    #[error("internal server error")]
    InternalServer,
    #[error("row not found")]
    RowNotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ureq error: {0}")]
    Ureq(#[from] ureq::Error),
    #[error("xml deserialize error: {0}")]
    DeserializeXml(#[from] serde_xml_rs::Error),
    #[error("chrono parse error: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub type Result<T> = core::result::Result<T, Error>;

fn not_found(error: Error) -> Response {
    (StatusCode::NOT_FOUND, error.to_string()).into_response()
}

fn internal_server_error(error: Error) -> Response {
    #[cfg(debug_assertions)]
    return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
    #[cfg(not(debug_assertions))]
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_owned(),
    )
        .into_response();
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotFound | Error::RowNotFound => not_found(self),
            _ => internal_server_error(self),
        }
    }
}

#[derive(Debug)]
pub struct Database {
    pub db: ryzz::Database,
    pub posts: PostTable,
}

static DATABASE: OnceLock<Database> = OnceLock::new();

pub async fn db() -> Result<&'static Database> {
    match DATABASE.get() {
        Some(db) => Ok(db),
        None => {
            let db = ryzz::Database::new("db.sqlite3").await?;
            let posts = Post::table(&db).await?;
            let posts_link_ix = index("posts_link_ix").unique().on(posts, posts.link);
            db.create(&posts_link_ix).await?;
            DATABASE.set(Database { db, posts }).unwrap();
            Ok(DATABASE.get().ok_or(ryzz::Error::ConnectionClosed)?)
        }
    }
}

#[table("posts")]
pub struct Post {
    #[ryzz(pk)]
    pub id: String,
    pub source: String,
    pub title: String,
    pub link: String,
    pub created_at: i64,
    pub source_link: String,
}

impl Post {
    pub fn source_display(&self) -> &'static str {
        match self.source.as_str() {
            "https://blog.rust-lang.org/feed.xml" => "official rust blog",
            "https://this-week-in-rust.org/atom.xml" => "this week in rust",
            "https://reddit.com/r/rust/.rss" => "/r/rust",
            "https://hnrss.org/newest.atom?q=rust" => "hn",
            _ => "Unknown",
        }
    }
}

pub async fn import() -> Result<()> {
    println!("Importing atom feeds");
    download("https://blog.rust-lang.org/feed.xml").await?;
    download("https://this-week-in-rust.org/atom.xml").await?;
    download("https://reddit.com/r/rust/.rss").await?;
    download("https://hnrss.org/newest.atom?q=rust").await?;
    println!("Finished importing atom feeds");
    // YT videos?
    // twitter ?
    // mastodon ?
    // blogs ?
    // popular crates ?
    Ok(())
}

async fn download(url: &'static str) -> Result<()> {
    let Database { db, posts } = db().await?;
    let xml = fetch(url)?;
    let feed = atom_feed(&xml)?;
    for entry in feed.entry {
        let source_link = match entry.id {
            Some(s) => {
                if s.starts_with("https://") {
                    s
                } else {
                    String::with_capacity(0)
                }
            }
            None => String::with_capacity(0),
        };
        let post = Post {
            id: ulid(),
            source: url.to_owned(),
            title: entry.title,
            source_link,
            link: entry.link.href,
            created_at: to_seconds(&entry.updated.unwrap_or_default()).unwrap_or_default(),
        };
        let _rows_affected = match db.insert(*posts).values(post)?.rows_affected().await {
            Ok(_) => {}
            Err(err) => match err {
                ryzz::Error::TokioRusqlite(tre) => match tre {
                    tokio_rusqlite::Error::Rusqlite(re) => match re {
                        rusqlite::Error::SqliteFailure(error, message) => match error.code {
                            rusqlite::ffi::ErrorCode::ConstraintViolation => match message {
                                Some(s) => {
                                    // TODO expose common ffi error codes in ryzz
                                    // TODO this is a disaster area
                                    // TODO classic rust reverse stack trace
                                    // ignore unique constraint on posts.link
                                    if s == "UNIQUE constraint failed: posts.link" {
                                        ()
                                    }
                                }
                                None => todo!(),
                            },
                            _ => todo!(),
                        },
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
                _ => {
                    return Err(err.into());
                }
            },
        };
    }

    Ok(())
}

fn to_seconds(input: &str) -> Result<i64> {
    Ok(DateTime::parse_from_rfc3339(input)?.timestamp() as i64)
}

fn ulid() -> String {
    ulid::Ulid::new().to_string()
}

fn fetch(url: &'static str) -> Result<String> {
    Ok(ureq::get(url).call()?.into_string()?)
}

#[derive(Debug, Deserialize)]
pub struct AtomFeed {
    pub title: String,
    pub entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub title: String,
    pub link: Link,
    pub content: Option<String>,
    pub id: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: String,
    pub rel: Option<String>,
}

pub fn atom_feed(content: &String) -> Result<AtomFeed> {
    let feed: AtomFeed = serde_xml_rs::from_reader(content.as_bytes())?;

    Ok(feed)
}
