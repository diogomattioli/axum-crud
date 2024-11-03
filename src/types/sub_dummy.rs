use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use validator::Validate;

use crate::{prelude::*, router::SqlxPool};

use super::dummy::Dummy;

#[derive(Debug, Serialize, Deserialize, Validate, FromRow)]
pub struct SubDummy {
    pub id_sub_dummy: i64,
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Database<SqlxPool> for SubDummy {
    async fn insert(&self, pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("INSERT INTO sub_dummy VALUES (?, ?, ?) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn update(&self, pool: &SqlxPool) -> Result<(), impl Error> {
        sqlx::query("UPDATE sub_dummy SET name = ?, id_dummy = ? WHERE id_sub_dummy = ?")
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .bind(self.id_sub_dummy)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn delete(pool: &SqlxPool, id: i64) -> Result<(), impl Error> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = ?")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn fetch_one(pool: &SqlxPool, id: i64) -> Result<Self, impl Error> {
        sqlx::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = ?")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn count(pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("SELECT count(id_sub_dummy) FROM sub_dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
    }
}

impl MatchParent<SqlxPool> for SubDummy {
    type Parent = Dummy;

    async fn fetch_parent(
        pool: &SqlxPool,
        parent_id: i64,
        id: i64,
    ) -> Result<Self::Parent, impl Error> {
        sqlx::query_as(
            "SELECT b.* FROM sub_dummy a INNER JOIN dummy b ON a.id_dummy = b.id_dummy WHERE a.id_dummy = ? AND a.id_sub_dummy = ?"
        )
        .bind(parent_id)
        .bind(id)
        .fetch_one(pool).await
    }
}

impl Check for SubDummy {
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
