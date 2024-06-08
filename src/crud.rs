use axum::{ extract::Path, http::StatusCode, response::IntoResponse, Json };

use std::fmt::Debug;

pub async fn create<T: Debug>(Json(payload): Json<T>) -> StatusCode {
    println!("{:?}", payload);
    StatusCode::CREATED
}

pub async fn retrieve<T: Debug>(Path(id): Path<i64>) -> impl IntoResponse {
    println!("{:?}", id);
    (StatusCode::OK, "")
}

pub async fn update<T: Debug>(Path(id): Path<i64>, Json(payload): Json<T>) -> StatusCode {
    println!("{:?}", id);
    println!("{:?}", payload);
    StatusCode::OK
}

pub async fn delete<T: Debug>(Path(id): Path<i64>) -> StatusCode {
    println!("{:?}", id);
    StatusCode::NO_CONTENT
}
