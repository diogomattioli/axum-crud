use std::{error::Error, io::ErrorKind};

pub type SqlxPool = sqlx::pool::Pool<sqlx::Any>;

pub trait Database<Pool> {
    type Item;
    type Parent;

    async fn create(&self, pool: &Pool) -> Result<i64, impl Error>;
    async fn retrieve(pool: &Pool, id: i64) -> Result<Self::Item, impl Error>;
    async fn update(&self, pool: &Pool) -> Result<(), impl Error>;
    async fn delete(pool: &Pool, id: i64) -> Result<(), impl Error>;
    async fn count(pool: &Pool) -> Result<i64, impl Error>;
    async fn list(pool: &Pool, offset: i64, limit: i64) -> Result<Vec<Self::Item>, impl Error>;
    async fn parent(_pool: &Pool, _id: i64, _sub_id: i64) -> Result<Self::Parent, impl Error> {
        Err(std::io::Error::new(ErrorKind::Other, ""))
    }
}
