use eyre::{Context as _, Result};
use sqlx::{Executor, Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

pub async fn setup_db() -> Result<Pool<Sqlite>> {
    let db_url = "./sqlite:db";

    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    let query = sqlx::query("CREATE TABLE IF NOT EXISTS ads (title TEXT, id TEXT PRIMARY KEY);");
    pool.execute(query).await.context("creating ads table")?;

    Ok(pool)
}
