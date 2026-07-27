#[macro_use]
mod macros;

mod common;
pub mod page;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn create_pool() -> Pool<Postgres> {
    let database_url = std::env::var("DATABASE_URL").unwrap();

    // Lambdaは1コンテナが1リクエストしか同時処理しないため、
    // 実行環境ごとにプールの上限を変えられるようにしておく。
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap()
}
