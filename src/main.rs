mod crud;
mod traits;
mod types;

use axum::{ routing::{ delete, get, post, put }, Router };
use sqlx::any::AnyPoolOptions;
use types::Dummy;

#[tokio::main]
async fn main() {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new().max_connections(5).connect("sqlite:test.db").await.unwrap();

    let app = Router::new()
        .route("/", get(root))
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
