use std::{error::Error, io::ErrorKind};

use validator::Validate;

pub trait Database<P> {
    type Item;
    type Parent;

    async fn insert(&self, pool: &P) -> Result<i64, impl Error>;
    async fn update(&self, pool: &P) -> Result<(), impl Error>;
    async fn delete(pool: &P, id: i64) -> Result<(), impl Error>;
    async fn fetch_one(pool: &P, id: i64) -> Result<Self::Item, impl Error>;
    async fn fetch_all(pool: &P, offset: i64, limit: i64) -> Result<Vec<Self::Item>, impl Error>;
    async fn fetch_parent(
        _pool: &P,
        _parent_id: i64,
        _id: i64,
    ) -> Result<Self::Parent, impl Error> {
        Err(std::io::Error::new(ErrorKind::Other, ""))
    }
    async fn count(pool: &P) -> Result<i64, impl Error>;
}

pub trait Check: Validate
where
    Self: Sized,
{
    fn check_create(&mut self) -> Result<(), Vec<&str>> {
        self.check_validate()?;
        Ok(())
    }

    fn check_update(&mut self, _old: Self) -> Result<(), Vec<&str>> {
        self.check_validate()?;
        Ok(())
    }

    fn check_delete(&self) -> Result<(), Vec<&str>> {
        Ok(())
    }

    fn check_validate(&self) -> Result<(), Vec<&str>> {
        self.validate()
            .map_err(|err| err.into_errors().into_keys().collect::<Vec<_>>())
    }
}
