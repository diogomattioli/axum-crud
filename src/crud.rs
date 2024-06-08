use axum::{ extract::Path, http::StatusCode, response::IntoResponse, Json };

use std::fmt::Debug;

pub trait Creator {
    fn validate_create(&self) -> Result<(), String> {
        Ok(())
    }
    fn sql_insert(&self) -> &str;
}

pub trait Retriever {
    fn sql_retrieve(&self) -> &str;
}

pub trait Updater {
    fn validate_update(&self) -> Result<(), String> {
        Ok(())
    }
    fn sql_update(&self) -> &str;
}

pub trait Deleter {
    fn validate_delete(&self) -> Result<(), String> {
        Ok(())
    }
    fn sql_delete(&self) -> &str;
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
