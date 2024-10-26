use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{crud, list, types::dummy::Dummy};

pub type SqlxPool = sqlx::pool::Pool<sqlx::Any>;
pub type Pool = SqlxPool;

pub fn router() -> axum::Router<Pool> {
    Router::new()
        .route("/", get(root))
        .route("/dummy/", get(list::list::<Dummy>))
        .route("/dummy/", post(crud::create::<Dummy>))
        .route("/dummy/:id", get(crud::retrieve::<Dummy>))
        .route("/dummy/:id", put(crud::update::<Dummy>))
        .route("/dummy/:id", delete(crud::delete::<Dummy>))
}

async fn root() -> &'static str {
    "It works!"
}
