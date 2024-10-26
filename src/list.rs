use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{router::Pool, Database};

#[derive(Deserialize)]
pub struct QueryParams {
    offset: Option<i64>,
    limit: Option<i64>,
}

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 250;

pub async fn list<T>(State(pool): State<Pool>, Query(query): Query<QueryParams>) -> Response
where
    T: Database<Pool, Item = T> + Serialize,
{
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);

    if offset < 0 || limit <= 0 || limit > MAX_LIMIT {
        StatusCode::BAD_REQUEST.into_response();
    }

    let total = T::count(&pool).await;
    match total {
        Ok(total) if total <= 0 => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        _ => {}
    }

    let list = T::fetch_all(&pool, offset, limit).await;
    match list {
        Ok(v) if v.len() > 0 => (
            StatusCode::OK,
            [("X-Paging-MaxLimit", format!("{}", MAX_LIMIT))],
            [("X-Paging-Total", format!("{}", total.unwrap_or(0)))],
            [("X-Paging-Size", format!("{}", v.len()))],
            serde_json::to_string(&v).unwrap_or(String::new()),
        )
            .into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{self, Request, StatusCode},
        routing::get,
        Router,
    };

    use crate::{
        prelude::*,
        types::{dummy::Dummy, sub_dummy::SubDummy},
    };
    use http_body_util::BodyExt;
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
            let _ = Dummy::insert(
                &(Dummy {
                    id_dummy: i,
                    name: format!("name-{}", i),
                    is_valid: Some(true),
                }),
                &pool,
            )
            .await;
            let _ = SubDummy::insert(
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

    async fn router(pool: Pool<Any>) -> axum::Router {
        Router::new()
            .route("/dummy/", get(super::list::<Dummy>))
            // .route("/dummy/:id/subdummy/", get(crud::sub_retrieve::<Dummy, SubDummy>))
            .with_state(pool)
    }

    #[tokio::test]
    async fn list_ok() {
        let pool = setup_db(10).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 10);
    }

    #[tokio::test]
    async fn list_empty() {
        let pool = setup_db(0).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_paging_total() {
        let pool = setup_db(100).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("X-Paging-Total")
                .map(|v| v.to_str().unwrap()),
            Some("100")
        );
    }

    #[tokio::test]
    async fn list_paging_size() {
        let pool = setup_db(100).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("X-Paging-Size")
                .map(|v| v.to_str().unwrap()),
            Some("50")
        );
    }

    #[tokio::test]
    async fn list_offset_and_limit() {
        let pool = setup_db(100).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/?offset=5&limit=5")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 5);

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| r.id_dummy)
                .collect::<Vec<i64>>(),
            [6, 7, 8, 9, 10]
        )
    }

    #[tokio::test]
    async fn list_offset() {
        let pool = setup_db(100).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/?offset=5")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 50);

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| r.id_dummy)
                .collect::<Vec<i64>>()[0..5],
            [6, 7, 8, 9, 10]
        )
    }

    #[tokio::test]
    async fn list_limit() {
        let pool = setup_db(100).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/?limit=5")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 5);

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| r.id_dummy)
                .collect::<Vec<i64>>(),
            [1, 2, 3, 4, 5]
        )
    }
}
