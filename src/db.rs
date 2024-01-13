use rizz::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

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

            let db = Database::new(connection);
            let _ = migrate(&db).await.expect("Failed to migrate");
            DATABASE.set(db).unwrap();
            DATABASE.get().unwrap()
        }
    }
}

async fn migrate(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let users = Users::new();

    db.create_table(users)
        .create_unique_index(users, vec![users.name])
        .migrate()
        .await?;

    Ok(())
}

#[derive(Table, Clone, Copy)]
#[rizz(table = "users")]
pub struct Users {
    #[rizz(primary_key, not_null)]
    id: Text,
    #[rizz(not_null)]
    name: Text,
    #[rizz(not_null)]
    secret: Text,
    #[rizz(not_null)]
    created_at: Integer,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: String,
    name: String,
    secret: String,
    created_at: u64,
}
