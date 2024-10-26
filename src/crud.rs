use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use serde::Serialize;

use crate::{
    prelude::*,
    traits::{Creator, Deleter, Updater},
};

pub async fn create<P, T>(uri: Uri, State(pool): State<P>, Json(mut new): Json<T>) -> Response
where
    T: Database<P, Item = T> + Creator,
{
    if T::validate_create(&mut new).is_err() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    match T::create(&new, &pool).await {
        Ok(id) => (
            StatusCode::CREATED,
            [
                ("Location", format!("{}{}", uri.path(), id)),
                ("X-Item-ID", format!("{}", id)),
            ],
        )
            .into_response(),
        Err(_) => StatusCode::NOT_ACCEPTABLE.into_response(),
    }
}

pub async fn retrieve<P, T>(State(pool): State<P>, Path(id): Path<i64>) -> Response
where
    T: Database<P, Item = T> + Serialize,
{
    match T::retrieve(&pool, id).await {
        Ok(old) => (StatusCode::OK, Json(old)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn update<P, T>(
    State(pool): State<P>,
    Path(id): Path<i64>,
    Json(mut new): Json<T>,
) -> StatusCode
where
    T: Database<P, Item = T> + Updater<T>,
{
    let Ok(old) = T::retrieve(&pool, id).await else {
        return StatusCode::NOT_FOUND;
    };

    if T::validate_update(&mut new, old).is_err() {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match T::update(&new, &pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_ACCEPTABLE,
    }
}

pub async fn delete<P, T>(State(pool): State<P>, Path(id): Path<i64>) -> StatusCode
where
    T: Database<P, Item = T> + Deleter,
{
    let Ok(old) = T::retrieve(&pool, id).await else {
        return StatusCode::NOT_FOUND;
    };

    if T::validate_delete(&old).is_err() {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match T::delete(&pool, id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_ACCEPTABLE,
    }
}

pub async fn sub_create<P, T, T2>(
    uri: Uri,
    State(pool): State<P>,
    Path(id): Path<i64>,
    Json(new): Json<T2>,
) -> Response
where
    T: Database<P, Item = T>,
    T2: Database<P, Item = T2> + Creator,
{
    if T::retrieve(&pool, id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    create::<P, T2>(uri, State(pool), Json(new)).await
}

pub async fn sub_retrieve<P, T, T2>(
    State(pool): State<P>,
    Path((id, sub_id)): Path<(i64, i64)>,
) -> Response
where
    T: Database<P, Item = T>,
    T2: Database<P, Item = T2> + Serialize,
{
    if T2::parent(&pool, id, sub_id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    retrieve::<P, T2>(State(pool), Path(sub_id)).await
}

pub async fn sub_update<P, T, T2>(
    State(pool): State<P>,
    Path((id, sub_id)): Path<(i64, i64)>,
    Json(new): Json<T2>,
) -> StatusCode
where
    T: Database<P, Item = T>,
    T2: Database<P, Item = T2> + Updater<T2>,
{
    if T2::parent(&pool, id, sub_id).await.is_err() {
        return StatusCode::NOT_FOUND;
    }

    update::<P, T2>(State(pool), Path(sub_id), Json(new)).await
}

pub async fn sub_delete<P, T, T2>(
    State(pool): State<P>,
    Path((id, sub_id)): Path<(i64, i64)>,
) -> StatusCode
where
    T: Database<P, Item = T>,
    T2: Database<P, Item = T2> + Deleter,
{
    if T2::parent(&pool, id, sub_id).await.is_err() {
        return StatusCode::NOT_FOUND;
    }

    delete::<P, T2>(State(pool), Path(sub_id)).await
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{self, Request, StatusCode},
        routing::{delete, get, post, put},
        Router,
    };

    use crate::{
        crud,
        prelude::*,
        types::{dummy::Dummy, sub_dummy::SubDummy},
    };
    use http_body_util::BodyExt;
    use serde_json::json;
    use sqlx::{any::AnyPoolOptions, Any, Executor, Pool};
    use tower::ServiceExt;

    async fn setup_db(size: i64) -> Pool<Any> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1) // needs to be 1, otherwise memory database is gone
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let _ = pool
            .execute(sqlx::raw_sql(
                "CREATE TABLE dummy (id_dummy bigint, name text);",
            ))
            .await;

        let _ = pool
            .execute(sqlx::raw_sql(
                "CREATE TABLE sub_dummy (id_sub_dummy bigint, name text, id_dummy bigint);",
            ))
            .await;

        for i in 1..=size {
            let _ = Dummy::create(
                &(Dummy {
                    id_dummy: i,
                    name: format!("name-{}", i),
                    is_valid: Some(true),
                }),
                &pool,
            )
            .await;
            let _ = SubDummy::create(
                &(SubDummy {
                    id_sub_dummy: i,
                    id_dummy: i,
                    name: format!("name-{}", i),
                    is_valid: Some(true),
                }),
                &pool,
            )
            .await;
        }

        pool
    }

    async fn app(pool: Pool<Any>) -> axum::Router {
        Router::new()
            .route("/dummy/", post(crud::create::<SqlxPool, Dummy>))
            .route("/dummy/:id", get(crud::retrieve::<SqlxPool, Dummy>))
            .route("/dummy/:id", put(crud::update::<SqlxPool, Dummy>))
            .route("/dummy/:id", delete(crud::delete::<SqlxPool, Dummy>))
            .route(
                "/dummy/:id/subdummy/",
                post(crud::sub_create::<SqlxPool, Dummy, SubDummy>),
            )
            .route(
                "/dummy/:id/subdummy/:id",
                get(crud::sub_retrieve::<SqlxPool, Dummy, SubDummy>),
            )
            .route(
                "/dummy/:id/subdummy/:id",
                put(crud::sub_update::<SqlxPool, Dummy, SubDummy>),
            )
            .route(
                "/dummy/:id/subdummy/:id",
                delete(crud::sub_delete::<SqlxPool, Dummy, SubDummy>),
            )
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
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = Dummy::retrieve(&pool, 1).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name");
    }

    #[tokio::test]
    async fn create_ok_location() {
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("Location")
                .map(|v| v.to_str().unwrap()),
            Some("/dummy/1")
        );
    }

    #[tokio::test]
    async fn create_ok_x_item_id() {
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("X-Item-ID")
                .map(|v| v.to_str().unwrap()),
            Some("1")
        );
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/")
                    .body(body)
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_sub_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name", "id_sub_dummy": 2}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/1/subdummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 2).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name");
        assert_eq!(dummy.id_sub_dummy, 2);
    }

    #[tokio::test]
    async fn create_sub_not_found() {
        let pool = setup_db(0).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name", "id_sub_dummy": 2}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/dummy/1/subdummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(SubDummy::retrieve(&pool, 2).await.is_err());
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummy: Dummy = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-1");
    }

    #[tokio::test]
    async fn retrieve_inexsistent() {
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn retrieve_sub_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/1/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummy: Dummy = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-1");
    }

    #[tokio::test]
    async fn retrieve_sub_not_found() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn retrieve_sub_mismatch() {
        let pool = setup_db(2).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = Dummy::retrieve(&pool, 1).await.unwrap();

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
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1")
                    .body(body)
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn update_sub_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new", "id_sub_dummy": 1}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/1/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(dummy.id_sub_dummy, 1);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-new");
    }

    #[tokio::test]
    async fn update_sub_not_found() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new", "id_sub_dummy": 1}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(dummy.id_sub_dummy, 1);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-1");
    }

    #[tokio::test]
    async fn update_sub_mismatch() {
        let pool = setup_db(2).await;

        let app = app(pool.clone()).await;

        let body = json!({"id_dummy": 1, "name": "name-new", "id_sub_dummy": 1}).to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(dummy.id_sub_dummy, 1);
        assert_eq!(dummy.id_dummy, 1);
        assert_eq!(dummy.name, "name-1");
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
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = Dummy::retrieve(&pool, 1).await;

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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
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
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = Dummy::retrieve(&pool, 1).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(dummy.is_err());
    }

    #[tokio::test]
    async fn delete_sub_ok() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/1/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(dummy.is_err());
    }

    #[tokio::test]
    async fn delete_sub_not_found() {
        let pool = setup_db(1).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(dummy.is_ok());
    }

    #[tokio::test]
    async fn delete_sub_mismatch() {
        let pool = setup_db(2).await;

        let app = app(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/dummy/2/subdummy/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let dummy = SubDummy::retrieve(&pool, 1).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(dummy.is_ok());
    }
}
