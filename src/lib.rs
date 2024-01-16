use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use rizz::desc;
use rizz::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[allow(unused)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(String),
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
}

pub type Result<T> = std::result::Result<T, Error>;

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

impl From<rizz::Error> for Error {
    fn from(value: rizz::Error) -> Self {
        match value {
            rizz::Error::RowNotFound => Error::NotFound,
            rizz::Error::Database(err) => Error::Database(err),
            _ => Error::InternalServer,
        }
    }
}

static DATABASE: OnceLock<Database> = OnceLock::new();

pub async fn db() -> &'static Database {
    match DATABASE.get() {
        Some(db) => db,
        None => {
            let connection = Connection::new("db.sqlite3")
                .create_if_missing(true)
                .journal_mode(JournalMode::Wal)
                .synchronous(Synchronous::Normal)
                .open()
                .await
                .expect("Could not open db connection");

            let db = rizz::Database::new(connection);
            let _ = migrate(&db).await.expect("Failed to migrate");
            DATABASE.set(Database::new(db, Posts::new())).unwrap();
            DATABASE.get().unwrap()
        }
    }
}

async fn migrate(db: &rizz::Database) -> Result<()> {
    let posts = Posts::new();

    db.create_table(posts)
        .create_unique_index(posts, vec![posts.link])
        .migrate()
        .await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Database {
    pub db: rizz::Database,
    pub posts: Posts,
}

impl Database {
    pub fn new(db: rizz::Database, posts: Posts) -> Self {
        Self { db, posts }
    }
}

#[derive(Table, Clone, Copy, Debug)]
#[rizz(table = "posts")]
pub struct Posts {
    #[rizz(primary_key, not_null)]
    pub id: Text,
    #[rizz(not_null)]
    pub title: Text,
    #[rizz(not_null)]
    pub link: Text,
    #[rizz(not_null)]
    pub created_at: Integer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub link: String,
    pub created_at: u64,
}

pub async fn import() -> Result<()> {
    let _x = download("https://blog.rust-lang.org/feed.xml").await?;
    // TODO: https://blog.rust-lang.org/feed.xml
    // TODO: https://lib.rs/atom.xml
    // TODO: https://this-week-in-rust.org/atom.xml
    // TODO: https://reddit.com/r/rust.json
    // YT videos?
    // hacker news links?
    Ok(())
}

async fn download(url: &'static str) -> Result<()> {
    let Database { db, posts } = db().await;
    let xml = fetch(url)?;
    let feed = atom_feed(&xml)?;
    for entry in feed.entry {
        let post = Post {
            id: ulid(),
            title: entry.title,
            link: entry.link.href,
            created_at: 0,
        };
        let _rows_affected = db.insert(posts).values(post)?.rows_affected().await?;
    }

    Ok(())
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
    pub content: String,
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
