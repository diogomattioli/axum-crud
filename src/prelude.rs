use std::error::Error;

pub trait Database<DB>
where
    Self: Sized,
{
    async fn insert(&self, pool: &DB) -> Result<i64, impl Error>;
    async fn update(&self, pool: &DB) -> Result<(), impl Error>;
    async fn delete(pool: &DB, id: i64) -> Result<(), impl Error>;
    async fn fetch_one(pool: &DB, id: i64) -> Result<Self, impl Error>;
    async fn count(pool: &DB) -> Result<i64, impl Error>;
}

#[derive(PartialEq, Debug, Clone)]
pub enum QueryToken {
    Text(String),
    Numeric(i64),
    Float(f64),
}

pub trait DatabaseFetchAll<DB>
where
    Self: Sized,
{
    const FIELD_PARENT: &'static str = "";

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
            .map(|token| QueryToken::Text(format!("%{}%", token.trim().to_string())));

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

    fn create_query_where(tokens: &Vec<QueryToken>) -> Option<String> {
        let mut pieces = vec![];

        if !Self::FIELD_PARENT.is_empty() {
            pieces.push(format!("{} = ?", Self::FIELD_PARENT));
        }

        if !tokens.is_empty() {
            pieces.push(format!(
                "({})",
                tokens
                    .iter()
                    .flat_map(|token| {
                        Self::get_field_array(token).iter().map(move |field| {
                            if let QueryToken::Text(_) = token {
                                format!("{field} LIKE ?")
                            } else {
                                format!("{field} = ?")
                            }
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(" OR ")
            ));
        }

        if !pieces.is_empty() {
            Some(format!("WHERE {}", pieces.join(" AND ")))
        } else {
            None
        }
    }

    fn fill_query_where<Q, F>(tokens: Vec<QueryToken>, mut query: Q, mut f: F) -> Q
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

    const FIELDS_ORDER: &'static [&'static str] = &[];

    fn create_query_order(order: String) -> Option<String> {
        if Self::FIELDS_ORDER.contains(&order.as_str()) {
            Some(format!("ORDER BY {order}"))
        } else {
            None
        }
    }

    async fn fetch_all(
        pool: &DB,
        search: Option<String>,
        order: Option<String>,
        parent_id: Option<i64>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, impl Error>;
}

pub trait MatchParent<DB> {
    type Parent;

    async fn fetch_parent(pool: &DB, parent_id: i64, id: i64) -> Result<Self::Parent, impl Error>;

    fn get_parent_id(&mut self) -> i64;
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
    use std::vec;

    use crate::router::Pool;

    use super::*;

    struct QueryStruct;
    impl DatabaseFetchAll<Pool> for QueryStruct {
        const FIELDS_TEXT: &'static [&'static str] = &["title", "name"];
        const FIELDS_NUMERIC: &'static [&'static str] = &["id", "size"];
        const FIELDS_FLOAT: &'static [&'static str] = &["lat", "lon"];

        async fn fetch_all(
            _pool: &Pool,
            _search: Option<String>,
            _order: Option<String>,
            _parent_id: Option<i64>,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<Self>, impl Error> {
            Ok::<Vec<Self>, std::io::Error>(vec![])
        }
    }

    #[test]
    fn query_tokens() {
        let mut tokens = QueryStruct::tokens("name 1 1.23".to_string()).into_iter();

        assert_eq!(tokens.next(), Some(QueryToken::Float(1.0)));
        assert_eq!(tokens.next(), Some(QueryToken::Float(1.23)));
        assert_eq!(tokens.next(), Some(QueryToken::Numeric(1)));
        assert_eq!(tokens.next(), Some(QueryToken::Text("%name%".to_string())));
        assert_eq!(tokens.next(), Some(QueryToken::Text("%1%".to_string())));
        assert_eq!(tokens.next(), Some(QueryToken::Text("%1.23%".to_string())));
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn query_create_where() {
        let tokens = QueryStruct::tokens("name 1 1.23".to_string());

        let sql = QueryStruct::create_query_where(&tokens);

        assert_eq!(sql, Some("WHERE (lat = ? OR lon = ? OR lat = ? OR lon = ? OR id = ? OR size = ? OR title LIKE ? OR name LIKE ? OR title LIKE ? OR name LIKE ? OR title LIKE ? OR name LIKE ?)".to_string()))
    }
}
