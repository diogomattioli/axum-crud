use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{router::Pool, Database, DatabaseFetchAll, MatchParent};

#[derive(Deserialize)]
pub struct QueryParams {
    search: Option<String>,
    order: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
}

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 250;

pub async fn list<T>(
    State(pool): State<Pool>,
    parent_id: Option<Path<i64>>,
    Query(query): Query<QueryParams>,
) -> Response
where
    T: Database<Pool> + DatabaseFetchAll<Pool> + Serialize,
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

    let list = T::fetch_all(
        &pool,
        query.search,
        query.order,
        parent_id.map(|Path(v)| v),
        offset,
        limit,
    )
    .await;
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

pub async fn sub_list<T>(
    State(pool): State<Pool>,
    Path(parent_id): Path<i64>,
    Query(query): Query<QueryParams>,
) -> Response
where
    T: Database<Pool> + DatabaseFetchAll<Pool> + MatchParent<Pool> + Serialize,
    T::Parent: Database<Pool>,
{
    if T::Parent::fetch_one(&pool, parent_id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    list::<T>(State(pool), Some(Path(parent_id)), Query(query)).await
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

    async fn database(size: i64) -> Pool<Any> {
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
                    name: format!("sub-name-{}", i),
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
            .route("/dummy/:id/sub_dummy/", get(super::sub_list::<SubDummy>))
            .with_state(pool)
    }

    #[tokio::test]
    async fn list_ok() {
        let pool = database(10).await;

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
        let pool = database(0).await;

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
        let pool = database(100).await;

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
        let pool = database(100).await;

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
        let pool = database(100).await;

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
        let pool = database(100).await;

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
        let pool = database(100).await;

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

    #[tokio::test]
    async fn list_search() {
        let pool = database(10).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/?search=-9")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 1);

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| r.id_dummy)
                .collect::<Vec<i64>>(),
            [9]
        )
    }

    #[tokio::test]
    async fn list_order() {
        let pool = database(10).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/?order=name")
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

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| r.id_dummy)
                .collect::<Vec<i64>>(),
            [1, 10, 2, 3, 4, 5, 6, 7, 8, 9]
        )
    }

    #[tokio::test]
    async fn list_sub_ok() {
        let pool = database(10).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/1/sub_dummy/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<Dummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 1);
    }

    #[tokio::test]
    async fn list_sub_search() {
        let pool = database(10).await;

        let app = router(pool.clone()).await;

        let body = "".to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/dummy/1/sub_dummy/?search=sub-name-1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dummies: Vec<SubDummy> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dummies.len(), 1);

        assert_eq!(
            dummies
                .into_iter()
                .map(|r| (r.id_dummy, r.id_sub_dummy))
                .collect::<Vec<_>>(),
            [(1, 1)]
        )
    }
}
