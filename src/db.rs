use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;

pub async fn create_pool() -> anyhow::Result<SqlitePool> {
    dotenv().ok();
    let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&db_url).await?;
    Ok(pool)
}

pub async fn add_hash(pool: &SqlitePool, url: &String, hash: &String) -> anyhow::Result<i64> {
    let id = sqlx::query!(
        r#"
            INSERT INTO hashes (url, hash)
            VALUES (?1, ?2);
        "#,
        url,
        hash
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

pub async fn url_from_hash(pool: &SqlitePool, hash: &String) -> anyhow::Result<String> {
    let row: (String,) = sqlx::query_as(
        r#"
            SELECT url FROM hashes
            WHERE hash = $1
        "#,
    )
    .bind(hash)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
