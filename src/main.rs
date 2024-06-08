mod crud;

use axum::{ routing::{ delete, get, post, put }, Router };

#[derive(Debug, serde::Deserialize)]
pub struct Dummy {
    pub name: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/dummy/", post(crud::create::<Dummy>))
        .route("/dummy/:id", get(crud::retrieve::<Dummy>))
        .route("/dummy/:id", put(crud::update::<Dummy>))
        .route("/dummy/:id", delete(crud::delete::<Dummy>));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "It works!"
}
