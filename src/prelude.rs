use std::error::Error;

use validator::Validate;

pub trait Database<DB>
where
    Self: Sized,
{
    async fn insert(&self, pool: &DB) -> Result<i64, impl Error>;
    async fn update(&self, pool: &DB) -> Result<(), impl Error>;
    async fn delete(pool: &DB, id: i64) -> Result<(), impl Error>;
    async fn fetch_one(pool: &DB, id: i64) -> Result<Self, impl Error>;
    async fn fetch_all(pool: &DB, offset: i64, limit: i64) -> Result<Vec<Self>, impl Error>;
    async fn count(pool: &DB) -> Result<i64, impl Error>;
}

pub trait MatchParent<DB> {
    type Parent;

    async fn fetch_parent(pool: &DB, parent_id: i64, id: i64) -> Result<Self::Parent, impl Error>;
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
