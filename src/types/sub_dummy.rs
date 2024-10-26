use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::{prelude::*, router::SqlxPool};

use super::dummy::Dummy;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SubDummy {
    pub id_sub_dummy: i64,
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Database<SqlxPool> for SubDummy {
    type Item = Self;
    type Parent = Dummy;

    async fn insert(&self, pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("INSERT INTO sub_dummy VALUES ($1, $2, $3) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn update(&self, pool: &SqlxPool) -> Result<(), impl Error> {
        sqlx::query("UPDATE sub_dummy SET name = $2, id_dummy = $3 WHERE id_sub_dummy = $1")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn delete(pool: &SqlxPool, id: i64) -> Result<(), impl Error> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn fetch_one(pool: &SqlxPool, id: i64) -> Result<Self::Item, impl Error> {
        sqlx::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn fetch_all(
        pool: &SqlxPool,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self::Item>, impl Error> {
        sqlx::query_as("SELECT * FROM sub_dummy limit $1, $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    async fn fetch_parent(
        pool: &SqlxPool,
        parent_id: i64,
        id: i64,
    ) -> Result<Self::Parent, impl Error> {
        sqlx::query_as(
            "SELECT * FROM sub_dummy a INNER JOIN dummy b ON a.id_dummy = b.id_dummy WHERE a.id_dummy = $1 AND a.id_sub_dummy = $2"
        )
        .bind(parent_id)
        .bind(id)
        .fetch_one(pool).await
    }

    async fn count(pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("SELECT count(id_sub_dummy) FROM sub_dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
    }
}

impl Check for SubDummy {
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
