use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::prelude::*;
use crate::traits::Result;
use crate::traits::{Creator, Deleter, Pool, ResultErr, Retriever, Sub, Updater};

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

    async fn create(&self, pool: &SqlxPool) -> std::result::Result<i64, impl Error> {
        sqlx::query("INSERT INTO sub_dummy VALUES ($1, $2, $3) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn retrieve(pool: &SqlxPool, id: i64) -> std::result::Result<Self::Item, impl Error> {
        sqlx::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    async fn update(&self, pool: &SqlxPool) -> std::result::Result<(), impl Error> {
        sqlx::query("UPDATE sub_dummy SET name = $2, id_dummy = $3 WHERE id_sub_dummy = $1")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn delete(pool: &SqlxPool, id: i64) -> std::result::Result<(), impl Error> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    async fn count(pool: &SqlxPool) -> std::result::Result<i64, impl Error> {
        sqlx::query("SELECT count(id_sub_dummy) FROM sub_dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn list(
        pool: &SqlxPool,
        offset: i64,
        limit: i64,
    ) -> std::result::Result<Vec<Self::Item>, impl Error> {
        sqlx::query_as("SELECT * FROM sub_dummy limit $1, $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
    }
}

impl Creator for SubDummy {
    fn validate_create(&mut self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn database_create(&self, pool: &Pool) -> Result<i64> {
        sqlx::query("INSERT INTO sub_dummy VALUES ($1, $2, $3) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .fetch_one(pool)
            .await?
            .try_get(0)
    }
}

impl Retriever<Self> for SubDummy {
    async fn database_retrieve(pool: &Pool, id: i64) -> Result<Self> {
        sqlx::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }
}

impl Updater<Self> for SubDummy {
    fn validate_update(&mut self, old: Self) -> ResultErr<String> {
        let _ = old;
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn database_update(&self, pool: &Pool) -> Result<()> {
        sqlx::query("UPDATE sub_dummy SET name = $2, id_dummy = $3 WHERE id_sub_dummy = $1")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .execute(pool)
            .await
            .map(|_| ())
    }
}

impl Deleter for SubDummy {
    fn validate_delete(&self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn database_delete(pool: &Pool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
    }
}

impl Sub<Dummy> for SubDummy {
    async fn database_match_sub(pool: &Pool, id: i64, sub_id: i64) -> Result<Dummy> {
        sqlx
            ::query_as(
                "SELECT * FROM sub_dummy a INNER JOIN dummy b ON a.id_dummy = b.id_dummy WHERE a.id_dummy = $1 AND a.id_sub_dummy = $2"
            )
            .bind(id)
            .bind(sub_id)
            .fetch_one(pool).await
    }
}
