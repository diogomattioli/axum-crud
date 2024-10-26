use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Database<SqlxPool> for Dummy {
    type Item = Self;
    type Parent = Self;

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

    async fn fetch_one(pool: &SqlxPool, id: i64) -> Result<Self::Item, impl Error> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn fetch_all(
        pool: &SqlxPool,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self::Item>, impl Error> {
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
    type Item = Self;

    fn check_create(&mut self) -> Result<(), HashMap<String, String>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(HashMap::new()),
        }
    }

    fn check_update(&mut self, _old: Self::Item) -> Result<(), HashMap<String, String>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(HashMap::new()),
        }
    }

    fn check_delete(&self) -> Result<(), HashMap<String, String>> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err(HashMap::new()),
        }
    }
}
