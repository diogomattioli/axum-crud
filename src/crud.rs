use axum::{
    extract::{ Path, State },
    http::StatusCode,
    response::{ IntoResponse, Response },
    Json,
};

use serde::Serialize;
use sqlx::{ any::AnyRow, Any, Executor, Pool };

use crate::traits::{ Creator, Deleter, Retriever, Updater };

pub async fn create<T>(State(pool): State<Pool<Any>>, Json(mut new): Json<T>) -> StatusCode
    where T: Creator
{
    if let Err(_) = T::validate_create(&mut new) {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match pool.execute(T::prepare_create(&new)).await {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::NOT_ACCEPTABLE,
    }
}

pub async fn retrieve<T>(State(pool): State<Pool<Any>>, Path(id): Path<i64>) -> Response
    where T: TryFrom<AnyRow> + Retriever + Serialize
{
    let Ok(row) = pool.fetch_one(T::prepare_retrieve(id)).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(old) = TryInto::<T>::try_into(row) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (StatusCode::OK, Json(old)).into_response()
}

pub async fn update<T>(
    State(pool): State<Pool<Any>>,
    Path(id): Path<i64>,
    Json(mut new): Json<T>
) -> StatusCode
    where T: TryFrom<AnyRow> + Retriever + Updater<T>
{
    let Ok(row) = pool.fetch_one(T::prepare_retrieve(id)).await else {
        return StatusCode::NOT_FOUND;
    };

    let Ok(old) = row.try_into() else {
        return StatusCode::NOT_FOUND;
    };

    if let Err(_) = T::validate_update(&mut new, old) {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match pool.execute(T::prepare_update(&new)).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_ACCEPTABLE,
    }
}

pub async fn delete<T>(State(pool): State<Pool<Any>>, Path(id): Path<i64>) -> StatusCode
    where T: TryFrom<AnyRow> + Retriever + Deleter
{
    let Ok(row) = pool.fetch_one(T::prepare_retrieve(id)).await else {
        return StatusCode::NOT_FOUND;
    };

    let Ok(old) = row.try_into() else {
        return StatusCode::NOT_FOUND;
    };

    if let Err(_) = T::validate_delete(&old) {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match pool.execute(T::prepare_delete(id)).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_ACCEPTABLE,
    }
}

#[cfg(test)]
mod tests {
    use axum::{ http::{ self, Request, StatusCode }, routing::{ delete, get, post, put }, Router };

    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt;
    use sqlx::{ any::AnyPoolOptions, Any, Executor, Pool };
    use crate::{ crud, traits::{ Creator, Retriever }, types::dummy::Dummy };

    async fn setup_db(size: i64) -> Pool<Any> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1) // needs to be 1, otherwise memory database is gone
            .connect("sqlite::memory:").await
            .unwrap();

        let _ = pool.execute(
            sqlx::raw_sql("CREATE TABLE dummy (id_dummy bigint, name text);")
        ).await;

        for i in 1..=size {
            let _ = pool.execute(
                Dummy::prepare_create(
                    &(Dummy { id_dummy: i, name: format!("name-{}", i), is_valid: Some(true) })
                )
            ).await;
        }

        pool
    }

    async fn app(pool: Pool<Any>) -> axum::Router {
        Router::new()
            .route("/dummy/", post(crud::create::<Dummy>))
            .route("/dummy/:id", get(crud::retrieve::<Dummy>))
            .route("/dummy/:id", put(crud::update::<Dummy>))
            .route("/dummy/:id", delete(crud::delete::<Dummy>))
            .with_state(pool)
    }

    #[tokio::test]
    async fn create_ok() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        let dummy: Dummy = pool
            .fetch_one(Dummy::prepare_retrieve(1)).await
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name");
    }

    #[tokio::test]
    async fn create_empty() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_invalid() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name", "is_valid": false}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn create_bad_json() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = r#"{"id_dummy": 1, "name": "name" 123, "is_valid": false}:"#.to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_no_content_type() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name"}).to_string();

        let response = app
            .oneshot(
                Request::builder().method(http::Method::POST).uri("/dummy/").body(body).unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_wrong_content_type() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::CSV.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn retrieve_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummy: Dummy = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-1");
    }

    #[tokio::test]
    async fn retrieve_not_found() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn retrieve_bad_id() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/a")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        let dummy: Dummy = pool
            .fetch_one(Dummy::prepare_retrieve(1)).await
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-new");
    }

    #[tokio::test]
    async fn update_no_content_type() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new"}).to_string();

        let response = app
            .oneshot(
                Request::builder().method(http::Method::PUT).uri("/dummy/1").body(body).unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_wrong_content_type() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::CSV.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_empty() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_bad_id() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/a")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_bad_json() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = r#"{"id_dummy": 1, "name": "name" 123, "is_valid": false}:"#.to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_inexistent() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new"}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_invalid() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name", "is_valid": false}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn delete_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        let dummy = pool.fetch_one(Dummy::prepare_retrieve(1)).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(dummy.is_err());
    }

    #[tokio::test]
    async fn delete_bad_id() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/a")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_inexistent() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_invalid() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap()
            ).await
            .unwrap();

        let dummy = pool.fetch_one(Dummy::prepare_retrieve(1)).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(dummy.is_err());
    }
}
