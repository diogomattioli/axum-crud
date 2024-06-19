use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, Row };

use crate::traits::{ Creator, Deleter, Pool, Result, ResultErr, Retriever, Updater };

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Creator for Dummy {
    fn validate_create(&mut self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn prepare_create(&self, pool: &Pool) -> Result<i64> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2) RETURNING id_dummy")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .fetch_one(pool).await?
            .try_get(0)
    }
}

impl Retriever<Self> for Dummy {
    async fn prepare_retrieve(pool: &Pool, id: i64) -> Result<Self> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = $1").bind(id).fetch_one(pool).await
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

    async fn prepare_update(&self, pool: &Pool) -> Result<()> {
        sqlx::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(self.name.clone())
            .execute(pool).await
            .map(|_| ())
    }
}

impl Deleter for Dummy {
    fn validate_delete(&self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn prepare_delete(pool: &Pool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1")
            .bind(id)
            .execute(pool).await
            .map(|_| ())
    }
}
