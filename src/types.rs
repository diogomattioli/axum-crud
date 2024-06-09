use sqlx::any::AnyArguments;

use crate::crud;

#[derive(Debug, serde::Deserialize)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
}

impl crud::Creator for Dummy {
    fn create_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("INSERT INTO dummy VALUES ($1, $2)").bind(self.id_dummy).bind(self.name.clone())
    }
}

impl crud::Retriever for Dummy {
    fn retrieve_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("SELECT * FROM dummy WHERE id_dummy = $1").bind(id)
    }
}

impl crud::Updater for Dummy {
    fn update_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>> {
        sqlx::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(self.name.clone())
    }
}

impl crud::Deleter for Dummy {
    fn delete_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>> {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id)
    }
}
