use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::prelude::*;
use crate::traits::{Creator, Deleter, ResultErr, Updater};

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

    async fn create(&self, pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2) RETURNING id_dummy")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn retrieve(pool: &SqlxPool, id: i64) -> Result<Self::Item, impl Error> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = $1")
            .bind(id)
            .fetch_one(pool)
            .await
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

    async fn count(pool: &SqlxPool) -> Result<i64, impl Error> {
        sqlx::query("SELECT count(id_dummy) FROM dummy")
            .fetch_one(pool)
            .await?
            .try_get(0)
    }

    async fn list(pool: &SqlxPool, offset: i64, limit: i64) -> Result<Vec<Self::Item>, impl Error> {
        sqlx::query_as("SELECT * FROM dummy limit $1, $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
    }
}

impl Creator for Dummy {
    fn validate_create(&mut self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }
}

impl Updater<Self> for Dummy {
    fn validate_update(&mut self, old: Self) -> ResultErr<String> {
        let _ = old;
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }
}

impl Deleter for Dummy {
    fn validate_delete(&self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }
}
