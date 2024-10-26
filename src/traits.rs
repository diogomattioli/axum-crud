pub type ResultErr<T> = std::result::Result<(), T>;

pub type Pool = sqlx::pool::Pool<sqlx::Any>;

pub trait Creator {
    fn validate_create(&mut self) -> ResultErr<String> {
        Ok(())
    }
}

pub trait Updater<T> {
    fn validate_update(&mut self, old: T) -> ResultErr<String> {
        let _ = old;
        Ok(())
    }
}

pub trait Deleter {
    fn validate_delete(&self) -> ResultErr<String> {
        Ok(())
    }
}
