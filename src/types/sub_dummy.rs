use serde::{ Deserialize, Serialize };
use sqlx::{ any::AnyArguments, query::{ Query, QueryAs }, Any, FromRow };

use crate::traits::{ Creator, Deleter, Retriever, Sub, Updater };

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SubDummy {
    pub id_sub_dummy: i64,
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
    pub is_valid: Option<bool>,
}

impl Creator for SubDummy {
    fn validate_create(&mut self) -> Result<(), String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_create<'a>(&self) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("INSERT INTO sub_dummy VALUES ($1, $2, $3) RETURNING id_sub_dummy")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
    }
}

impl Retriever<Self> for SubDummy {
    fn prepare_retrieve<'a>(id: i64) -> QueryAs<'a, Any, Self, AnyArguments<'a>> {
        sqlx::query_as("SELECT * FROM sub_dummy WHERE id_sub_dummy = $1").bind(id)
    }
}

impl Updater<Self> for SubDummy {
    fn validate_update(&mut self, old: Self) -> Result<(), String> {
        let _ = old;
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_update<'a>(&self) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("UPDATE sub_dummy SET name = $2, id_dummy = $3 WHERE id_sub_dummy = $1")
            .bind(self.id_sub_dummy)
            .bind(self.name.clone())
            .bind(self.id_dummy)
    }
}

impl Deleter for SubDummy {
    fn validate_delete(&self) -> Result<(), String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_delete<'a>(id: i64) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM sub_dummy WHERE id_sub_dummy = $1").bind(id)
    }
}

impl Sub for SubDummy {
    fn prepare_sub_match<'a>(id: i64, sub_id: i64) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query(
            "SELECT * FROM sub_dummy a INNER JOIN dummy b ON a.id_dummy = b.id_dummy WHERE a.id_dummy = $1 AND a.id_sub_dummy = $2"
        )
            .bind(id)
            .bind(sub_id)
    }
}
