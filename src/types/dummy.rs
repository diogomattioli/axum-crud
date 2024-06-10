use serde::{ Deserialize, Serialize };
use sqlx::{ any::AnyArguments, query::{ Query, QueryAs }, Any, FromRow };

use crate::traits::{ Creator, Deleter, Retriever, Updater };

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
    #[sqlx(default)]
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
        sqlx::query("INSERT INTO dummy VALUES ($1, $2) RETURNING id_dummy")
            .bind(self.id_dummy)
            .bind(self.name.clone())
    }
}

impl Retriever<Self> for Dummy {
    fn prepare_retrieve<'a>(id: i64) -> QueryAs<'a, Any, Self, AnyArguments<'a>> {
        sqlx::query_as("SELECT * FROM dummy WHERE id_dummy = $1").bind(id)
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
            Some(true) | None => Ok(()),
            _ => Err("".to_string()),
        }
    }

    fn prepare_delete<'a>(id: i64) -> Query<'a, Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id)
    }
}
