use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, Row };

use crate::traits::{ Creator, Deleter, Pool, ResultErr, Retriever, Sub, Updater };
use crate::traits::Result;

use super::dummy::Dummy;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SubDummy {
    pub id_sub_dummy: i64,
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Creator for SubDummy {
    fn validate_create(&mut self) -> ResultErr<String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    async fn prepare_create(&self, pool: &Pool) -> Result<i64> {
        sqlx::query("INSERT INTO sub_dummy VALUES ($1, $2, $3) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .fetch_one(pool).await?
            .try_get(0)
    }
}

impl Retriever<Self> for SubDummy {
    async fn prepare_retrieve(pool: &Pool, id: i64) -> Result<Self> {
        sqlx
            ::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .fetch_one(pool).await
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

    async fn prepare_update(&self, pool: &Pool) -> Result<()> {
        sqlx::query("UPDATE sub_dummy SET name = $2, id_dummy = $3 WHERE id_sub_dummy = $1")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
            .execute(pool).await
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

    async fn prepare_delete(pool: &Pool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = $1")
            .bind(id)
            .execute(pool).await
            .map(|_| ())
    }
}

impl Sub<Dummy> for SubDummy {
    async fn prepare_sub_match(pool: &Pool, id: i64, sub_id: i64) -> Result<Dummy> {
        sqlx
            ::query_as(
                "SELECT * FROM sub_dummy a INNER JOIN dummy b ON a.id_dummy = b.id_dummy WHERE a.id_dummy = $1 AND a.id_sub_dummy = $2"
            )
            .bind(id)
            .bind(sub_id)
            .fetch_one(pool).await
    }
}
