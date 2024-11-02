use std::error::Error;

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

pub trait DatabaseWhere<DB>
where
    Self: DatabaseSearch<DB>,
{
    async fn fetch_where(
        pool: &DB,
        offset: i64,
        limit: i64,
        query: String,
    ) -> Result<Vec<Self>, impl Error>;
}

#[derive(PartialEq, Debug, Clone)]
pub enum QueryToken {
    Text(String),
    Numeric(i64),
    Float(f64),
}

pub trait DatabaseSearch<DB>
where
    Self: Sized,
{
    const FIELDS_TEXT: &'static [&'static str] = &[];
    const FIELDS_NUMERIC: &'static [&'static str] = &[];
    const FIELDS_FLOAT: &'static [&'static str] = &[];

    fn get_field_array(token: &QueryToken) -> &'static [&'static str] {
        match token {
            QueryToken::Text(_) => Self::FIELDS_TEXT,
            QueryToken::Numeric(_) => Self::FIELDS_NUMERIC,
            QueryToken::Float(_) => Self::FIELDS_FLOAT,
        }
    }

    fn tokens(query: String) -> Vec<QueryToken> {
        let tokens = query.split_whitespace().map(|s| s.trim().to_string());

        let iter = tokens
            .clone()
            .take_while(|_| !Self::FIELDS_TEXT.is_empty())
            .map(|token| QueryToken::Text(token.trim().to_string()));

        let iter = tokens
            .clone()
            .take_while(|_| !Self::FIELDS_NUMERIC.is_empty())
            .filter_map(|token| Some(QueryToken::Numeric(token.parse::<i64>().ok()?)))
            .chain(iter);

        let iter = tokens
            .take_while(|_| !Self::FIELDS_FLOAT.is_empty())
            .filter_map(|token| Some(QueryToken::Float(token.parse::<f64>().ok()?)))
            .chain(iter);

        iter.collect::<Vec<_>>()
    }

    fn create_where(tokens: &Vec<QueryToken>) -> String {
        format!(
            "WHERE {}",
            tokens
                .iter()
                .flat_map(|token| {
                    Self::get_field_array(token).iter().map(move |field| {
                        if let QueryToken::Text(_) = token {
                            format!("{field} LIKE '%?%'")
                        } else {
                            format!("{field} = ?")
                        }
                    })
                })
                .collect::<Vec<_>>()
                .join(" OR ")
        )
    }

    fn fill_where<Q, F>(tokens: Vec<QueryToken>, mut query: Q, mut f: F) -> Q
    where
        F: FnMut(Q, QueryToken) -> Q,
    {
        for token in tokens {
            for _ in 0..Self::get_field_array(&token).len() {
                query = f(query, token.clone());
            }
        }

        query
    }
}

pub trait MatchParent<DB> {
    type Parent;

    async fn fetch_parent(pool: &DB, parent_id: i64, id: i64) -> Result<Self::Parent, impl Error>;
}

pub trait Check
where
    Self: Sized,
{
    fn check_create(&mut self) -> Result<(), Vec<&str>> {
        Ok(())
    }

    fn check_update(&mut self, _old: Self) -> Result<(), Vec<&str>> {
        Ok(())
    }

    fn check_delete(&self) -> Result<(), Vec<&str>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::router::Pool;

    use super::*;

    struct QueryStruct;
    impl DatabaseSearch<Pool> for QueryStruct {
        const FIELDS_TEXT: &'static [&'static str] = &["title", "name"];
        const FIELDS_NUMERIC: &'static [&'static str] = &["id", "size"];
        const FIELDS_FLOAT: &'static [&'static str] = &["lat", "lon"];
    }

    #[test]
    fn query_tokens() {
        let mut tokens = QueryStruct::tokens("name 1 1.23".to_string()).into_iter();

        assert_eq!(tokens.next(), Some(QueryToken::Float(1.0)));
        assert_eq!(tokens.next(), Some(QueryToken::Float(1.23)));
        assert_eq!(tokens.next(), Some(QueryToken::Numeric(1)));
        assert_eq!(tokens.next(), Some(QueryToken::Text("name".to_string())));
        assert_eq!(tokens.next(), Some(QueryToken::Text("1".to_string())));
        assert_eq!(tokens.next(), Some(QueryToken::Text("1.23".to_string())));
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn query_create_where() {
        let tokens = QueryStruct::tokens("name 1 1.23".to_string());

        let sql = QueryStruct::create_where(&tokens);

        assert_eq!(sql, "WHERE lat = ? OR lon = ? OR lat = ? OR lon = ? OR id = ? OR size = ? OR title LIKE '%?%' OR name LIKE '%?%' OR title LIKE '%?%' OR name LIKE '%?%' OR title LIKE '%?%' OR name LIKE '%?%'")
    }
}
