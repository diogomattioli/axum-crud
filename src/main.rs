mod crud;
mod list;
mod prelude;
mod routes;
mod types;

use std::env;

use prelude::*;
use sqlx::any::AnyPoolOptions;

#[tokio::main]
async fn main() {
    let Ok(database_url) = env::var("DATABASE_URL") else {
        panic!("DATABASE_URL not set");
    };

    sqlx::any::install_default_drivers();
    let Ok(pool) = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    else {
        panic!("Cannot connect to the database");
    };

    let app = crate::routes::router().with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
