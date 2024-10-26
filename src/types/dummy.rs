use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use validator::Validate;

use crate::{prelude::*, router::SqlxPool};

#[derive(Debug, Serialize, Deserialize, Validate, FromRow)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Database<SqlxPool> for Dummy {
    async fn insert(&self, pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2) RETURNING id_dummy")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn update(&self, pool: &SqlxPool) -> Result<(), impl Error> {
        sqlx::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn delete(pool: &SqlxPool, id: i64) -> Result<(), impl Error> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn fetch_one(pool: &SqlxPool, id: i64) -> Result<Self, impl Error> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn fetch_all(pool: &SqlxPool, offset: i64, limit: i64) -> Result<Vec<Self>, impl Error> {
        sqlx::query_as("SELECT * FROM dummy limit $1, $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    async fn count(pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("SELECT count(id_dummy) FROM dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
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
