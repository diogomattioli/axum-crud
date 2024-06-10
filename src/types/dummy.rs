use serde::{ Deserialize, Serialize };
use sqlx::{ any::{ AnyArguments, AnyRow }, query::Query, Any, Error, Row };

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

    fn prepare_create<'a>(&self) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2)").bind(self.id_dummy).bind(self.name.clone())
    }
}

impl Retriever for Dummy {
    fn prepare_retrieve<'a>(id: i64) -> Query<'a, Any, AnyArguments<'a>> {
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

    fn prepare_update<'a>(&self) -> Query<'a, Any, AnyArguments<'a>> {
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

    fn prepare_delete<'a>(id: i64) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl TryFrom<AnyRow> for Dummy {
    type Error = Error;

    fn try_from(row: AnyRow) -> Result<Self, Self::Error> {
        Ok(Dummy {
            id_dummy: row.try_get("id_dummy")?,
            name: row.try_get("name")?,
            is_valid: Some(true),
        })
    }
}
