use axum::{
    extract::{ Path, State },
    http::StatusCode,
    response::{ IntoResponse, Response },
    Json,
};

use serde::Serialize;
use sqlx::{ any::AnyRow, Any, Executor, Pool };

use crate::traits::{ Creator, Deleter, Retriever, Updater };

pub async fn create<'a, T>(State(pool): State<Pool<Any>>, Json(new): Json<T>) -> StatusCode
    where T: Creator
{
    if let Err(_) = new.create_is_valid() {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    if let Err(_) = pool.execute(T::create_query(&new)).await {
        return StatusCode::NOT_ACCEPTABLE;
    }

    StatusCode::CREATED
}

pub async fn retrieve<'a, T>(State(pool): State<Pool<Any>>, Path(id): Path<i64>) -> Response
    where T: From<AnyRow> + Retriever + Serialize
{
    match pool.fetch_one(T::retrieve_query(id)).await {
        Ok(row) =>
            (
                StatusCode::OK,
                Json(serde_json::to_value(<AnyRow as Into<T>>::into(row)).unwrap()),
            ).into_response(),
        Err(_) => { StatusCode::NOT_FOUND.into_response() }
    }
}

pub async fn update<'a, T>(
    State(pool): State<Pool<Any>>,
    Path(id): Path<i64>,
    Json(new): Json<T>
) -> StatusCode
    where T: From<AnyRow> + Retriever + Updater<T>
{
    let Ok(row) = pool.fetch_one(T::retrieve_query(id)).await else {
        return StatusCode::NOT_FOUND;
    };

    let old: T = row.into();

    if let Err(_) = new.update_is_valid(old) {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    if let Err(_) = pool.execute(T::update_query(&new)).await {
        return StatusCode::NOT_ACCEPTABLE;
    }

    StatusCode::OK
}

pub async fn delete<'a, T>(State(pool): State<Pool<Any>>, Path(id): Path<i64>) -> StatusCode
    where T: From<AnyRow> + Retriever + Deleter
{
    let Ok(row) = pool.fetch_one(T::retrieve_query(id)).await else {
        return StatusCode::NOT_FOUND;
    };

    let old: T = row.into();

    if let Err(_) = old.delete_is_valid() {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    if let Err(_) = pool.execute(T::delete_query(id)).await {
        return StatusCode::NOT_ACCEPTABLE;
    }

    StatusCode::NO_CONTENT
}
