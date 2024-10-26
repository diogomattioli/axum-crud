use std::error::Error;

pub type SqlxPool = sqlx::pool::Pool<sqlx::Any>;

pub trait Database<Pool> {
    type Item;

    async fn create(&self, pool: &Pool) -> Result<i64, impl Error>;
    async fn retrieve(pool: &Pool, id: i64) -> Result<Self::Item, impl Error>;
    async fn update(&self, pool: &Pool) -> Result<(), impl Error>;
    async fn delete(pool: &Pool, id: i64) -> Result<(), impl Error>;
    async fn count(pool: &Pool) -> Result<i64, impl Error>;
    async fn list(pool: &Pool, offset: i64, limit: i64) -> Result<Vec<Self::Item>, impl Error>;
}
