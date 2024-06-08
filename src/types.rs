use sqlx::{ any::{ AnyQueryResult, AnyRow }, Executor };

use crate::crud;

#[derive(Debug, serde::Deserialize)]
pub struct Dummy {
    pub id_dummy: i64,
    pub name: String,
}

impl crud::Creator for Dummy {
    async fn create_query<'e, E>(&self, executor: E) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>
    {
        sqlx
            ::query("INSERT INTO dummy VALUES ($1, $2)")
            .bind(self.id_dummy)
            .bind(&self.name)
            .execute(executor).await
    }
}

impl crud::Retriever for Dummy {
    async fn retrieve_query<'e, E>(executor: E, id: i64) -> Result<AnyRow, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>
    {
        sqlx::query("SELECT * FROM dummy WHERE id_dummy = $1").bind(id).fetch_one(executor).await
    }
}

impl crud::Updater for Dummy {
    async fn update_query<'e, E>(&self, executor: E) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>
    {
        sqlx
            ::query("UPDATE dummy SET name = $2 WHERE id_dummy = $1")
            .bind(self.id_dummy)
            .bind(&self.name)
            .execute(executor).await
    }
}

impl crud::Deleter for Dummy {
    async fn delete_query<'e, E>(executor: E, id: i64) -> Result<AnyQueryResult, sqlx::Error>
        where E: Executor<'e, Database = sqlx::Any>
    {
        sqlx::query("DELETE FROM dummy WHERE id_dummy = $1").bind(id).execute(executor).await
    }
}
