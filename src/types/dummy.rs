use serde::{ Deserialize, Serialize };
use sqlx::{ any::{ AnyArguments, AnyRow }, Row };

use crate::traits::{ Creator, Deleter, Retriever, Updater };

#[derive(Debug, Serialize, Deserialize)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    pub is_valid: Option<bool>,
}

impl Creator for Dummy {
    fn validate_create(&mut self) -> Result<(), String> {
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_create<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2)").bind(self.id_dummy).bind(self.name.clone())
    }
}

impl Retriever for Dummy {
    fn prepare_retrieve<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("SELECT * FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl Updater<Self> for Dummy {
    fn validate_update(&mut self, old: Self) -> Result<(), String> {
        let _ = old;
        match self.is_valid {
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_update<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(self.name.clone())
    }
}

impl Deleter for Dummy {
    fn validate_delete(&self) -> Result<(), String> {
        match self.is_valid {
            Some(true) => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_delete<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl From<AnyRow> for Dummy {
    fn from(row: AnyRow) -> Self {
        Dummy {
            id_dummy: row.try_get("id_dummy").unwrap(),
            name: row.try_get("name").unwrap(),
            is_valid: Some(true),
        }
    }
}
