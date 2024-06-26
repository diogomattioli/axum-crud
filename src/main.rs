mod crud;
mod list;
mod traits;
mod types;

use std::env;

use axum::{ routing::{ delete, get, post, put }, Router };
use sqlx::any::AnyPoolOptions;
use types::dummy::Dummy;

#[tokio::main]
async fn main() {
    let Ok(database_url) = env::var("DATABASE_URL") else {
        panic!("DATABASE_URL not set");
    };

    sqlx::any::install_default_drivers();
    let Ok(pool) = AnyPoolOptions::new().max_connections(5).connect(&database_url).await else {
        panic!("Cannot connect to the database");
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/dummy/", get(list::list::<Dummy>))
        .route("/dummy/", post(crud::create::<Dummy>))
        .route("/dummy/:id", get(crud::retrieve::<Dummy>))
        .route("/dummy/:id", put(crud::update::<Dummy>))
        .route("/dummy/:id", delete(crud::delete::<Dummy>))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "It works!"
}
