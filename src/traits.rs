use sqlx::{ any::AnyArguments, query::{ Query, QueryAs }, Any };

pub trait Creator {
    fn validate_create(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn prepare_create<'a>(&self) -> Query<'a, Any, AnyArguments<'a>>;
}

pub trait Retriever<T> {
    fn prepare_retrieve<'a>(id: i64) -> QueryAs<'a, Any, T, AnyArguments<'a>>;
}

pub trait Updater<T> {
    fn validate_update(&mut self, old: T) -> Result<(), String> {
        let _ = old;
        Ok(())
    }

    fn prepare_update<'a>(&self) -> Query<'a, Any, AnyArguments<'a>>;
}

pub trait Deleter {
    fn validate_delete(&self) -> Result<(), String> {
        Ok(())
    }

    fn prepare_delete<'a>(id: i64) -> Query<'a, Any, AnyArguments<'a>>;
}

pub trait Sub<T> {
    fn prepare_sub_match<'a>(id: i64, sub_id: i64) -> Query<'a, Any, AnyArguments<'a>>;
}
