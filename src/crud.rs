use axum::{ extract::{ Path, State }, http::StatusCode, response::IntoResponse, Json };
use sqlx::{ any::AnyArguments, Any, Executor, Pool };

use std::fmt::Debug;

pub trait Creator {
    fn create_is_valid(&self) -> Result<(), String> {
        Ok(())
    }

    fn create_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Retriever {
    fn retrieve_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}

pub trait Updater {
    fn update_is_valid(&self) -> Result<(), String> {
        Ok(())
    }

    fn update_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Deleter {
    fn delete_is_valid(&self) -> Result<(), String> {
        Ok(())
    }

    fn delete_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}

pub async fn create<T: Creator + Debug>(Json(payload): Json<T>) -> StatusCode {
    println!("{:?}", payload);
    StatusCode::CREATED
}

pub async fn retrieve<T: Retriever + Debug>(Path(id): Path<i64>) -> impl IntoResponse {
    println!("{:?}", id);
    (StatusCode::OK, "")
}

pub async fn update<T: Updater + Debug>(Path(id): Path<i64>, Json(payload): Json<T>) -> StatusCode {
    println!("{:?}", id);
    println!("{:?}", payload);
    StatusCode::OK
}

pub async fn delete<T: Deleter + Debug>(Path(id): Path<i64>) -> StatusCode {
    println!("{:?}", id);
    StatusCode::NO_CONTENT
}
