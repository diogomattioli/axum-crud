use serde::{ Deserialize, Serialize };
use sqlx::{ any::{ AnyArguments, AnyRow }, Row };

use crate::traits::{ Creator, Deleter, Retriever, Updater };

#[derive(Debug, Serialize, Deserialize)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
}

impl Creator for Dummy {
    fn create_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2)").bind(self.id_dummy).bind(self.name.clone())
    }
}

impl Retriever for Dummy {
    fn retrieve_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("SELECT * FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl Updater<Self> for Dummy {
    fn update_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(self.name.clone())
    }
}

impl Deleter for Dummy {
    fn delete_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl From<AnyRow> for Dummy {
    fn from(row: AnyRow) -> Self {
        Dummy {
            id_dummy: row.try_get("id_dummy").unwrap(),
            name: row.try_get("name").unwrap(),
        }
    }
}
