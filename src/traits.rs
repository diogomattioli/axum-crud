use sqlx::any::AnyArguments;

pub trait Creator {
    fn create_is_valid(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn create_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Retriever {
    fn retrieve_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}

pub trait Updater<T> {
    fn update_is_valid(&mut self, old: T) -> Result<(), String> {
        let _ = old;
        Ok(())
    }

    fn update_query<'e>(&self) -> sqlx::query::Query<'e, sqlx::Any, AnyArguments<'e>>;
}

pub trait Deleter {
    fn delete_is_valid(&self) -> Result<(), String> {
        Ok(())
    }

    fn delete_query<'a>(id: i64) -> sqlx::query::Query<'a, sqlx::Any, AnyArguments<'a>>;
}
