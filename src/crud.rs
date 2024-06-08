use axum::{ extract::Path, http::StatusCode, response::IntoResponse, Json };
use sqlx::{ any::{ AnyQueryResult, AnyRow }, Executor };

use std::fmt::Debug;

pub trait Creator {
    async fn create_is_valid<'e, E>(&self, executor: E) -> Result<(), String>
        where E: Executor<'e, Database = sqlx::Any>
    {
        let _ = executor;
        Ok(())
    }

    async fn create_query<'e, E>(&self, executor: E) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>;
}

pub trait Retriever {
    async fn retrieve_query<'e, E>(executor: E, id: i64) -> Result<AnyRow, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>;
}

pub trait Updater {
    async fn update_is_valid<'e, E>(&self, executor: E) -> Result<(), String>
        where E: Executor<'e, Database = sqlx::Any>
    {
        let _ = executor;
        Ok(())
    }

    async fn update_query<'e, E>(&self, executor: E) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>;
}

pub trait Deleter {
    async fn delete_is_valid<'e, E>(&self, executor: E) -> Result<(), String>
        where E: Executor<'e, Database = sqlx::Any>
    {
        let _ = executor;
        Ok(())
    }

    async fn delete_query<'e, E>(executor: E, id: i64) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>;
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
