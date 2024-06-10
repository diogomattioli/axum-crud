use sqlx::any::AnyArguments;

pub trait Creator {
    fn validate_create(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn prepare_create<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Retriever {
    fn prepare_retrieve<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}

pub trait Updater<T> {
    fn validate_update(&mut self, old: T) -> Result<(), String> {
        let _ = old;
        Ok(())
    }

    fn prepare_update<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Deleter {
    fn validate_delete(&self) -> Result<(), String> {
        Ok(())
    }

    fn prepare_delete<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}
