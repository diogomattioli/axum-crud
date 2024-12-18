use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use validator::Validate;

use crate::{prelude::*, router::Pool};

#[derive(Debug, Serialize, Deserialize, Validate, FromRow)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Database<Pool> for Dummy {
    async fn insert(&self, pool: &Pool) -> Result<i64, impl Error> {
        sqlx::query("INSERT INTO dummy VALUES (?, ?) RETURNING id_dummy")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn update(&self, pool: &Pool) -> Result<(), impl Error> {
        sqlx::query("UPDATE dummy SET name = ? WHERE id_dummy = ?")
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn delete(pool: &Pool, id: i64) -> Result<(), impl Error> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = ?")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn fetch_one(pool: &Pool, id: i64) -> Result<Self, impl Error> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = ?")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn count(pool: &Pool) -> Result<i64, impl Error> {
        sqlx::query("SELECT count(id_dummy) FROM dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
    }
}

impl DatabaseFetchAll<Pool> for Dummy {
    const FIELDS_TEXT: &'static [&'static str] = &["name"];
    const FIELDS_NUMERIC: &'static [&'static str] = &["id_dummy"];

    const FIELDS_ORDER: &'static [&'static str] = &["id_dummy", "name"];

    async fn fetch_all(
        pool: &Pool,
        search: Option<String>,
        order: Option<String>,
        _parent_id: Option<i64>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, impl Error> {
        let tokens = Self::tokens(search.unwrap_or_default());

        let sql_where = Self::create_query_where(&tokens).unwrap_or_default();
        let sql_order = Self::create_query_order(order.unwrap_or_default()).unwrap_or_default();
        let sql = format!("SELECT * FROM dummy {} {} limit ?, ?", sql_where, sql_order);

        let mut query = sqlx::query_as(&sql);
        if !tokens.is_empty() {
            query = Self::fill_query_where(tokens, query, |query, token| match token {
                QueryToken::Text(value) => query.bind(value),
                QueryToken::Numeric(value) => query.bind(value),
                QueryToken::Float(value) => query.bind(value),
            });
        }
        query.bind(offset).bind(limit).fetch_all(pool).await
    }
}

impl Check for Dummy {
    fn check_create(&mut self) -> Result<(), Vec<&str>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(vec![]),
        }
    }

    fn check_update(&mut self, _old: Self) -> Result<(), Vec<&str>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(vec![]),
        }
    }

    fn check_delete(&self) -> Result<(), Vec<&str>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(vec![]),
        }
    }
}
