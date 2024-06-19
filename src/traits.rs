pub type Result<T> = std::result::Result<T, sqlx::Error>;
pub type ResultErr<T> = std::result::Result<(), T>;

pub type Pool = sqlx::pool::Pool<sqlx::Any>;

pub trait Creator {
    fn validate_create(&mut self) -> ResultErr<String> {
        Ok(())
    }

    async fn database_create(&self, pool: &Pool) -> Result<i64>;
}

pub trait Retriever<T> {
    async fn database_retrieve(pool: &Pool, id: i64) -> Result<T>;
}

pub trait Updater<T> {
    fn validate_update(&mut self, old: T) -> ResultErr<String> {
        let _ = old;
        Ok(())
    }

    async fn database_update(&self, pool: &Pool) -> Result<()>;
}

pub trait Deleter {
    fn validate_delete(&self) -> ResultErr<String> {
        Ok(())
    }

    async fn database_delete(pool: &Pool, id: i64) -> Result<()>;
}

pub trait Sub<T> {
    async fn database_match_sub(pool: &Pool, id: i64, sub_id: i64) -> Result<T>;
}
