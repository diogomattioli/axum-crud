use crate::crud;

#[derive(Debug, serde::Deserialize)]
pub struct Dummy {
    pub name: String,
}

impl crud::Creator for Dummy {
    fn sql_insert(&self) -> &str {
        "INSERT INTO dummy VALUES (?)"
    }
}

impl crud::Retriever for Dummy {
    fn sql_retrieve(&self) -> &str {
        "SELECT * FROM dummy WHERE id_dummy = ?"
    }
}

impl crud::Updater for Dummy {
    fn sql_update(&self) -> &str {
        "UPDATE dummy SET name = ? WHERE id_dummy = ?"
    }
}

impl crud::Deleter for Dummy {
    fn sql_delete(&self) -> &str {
        "DELETE FROM dummy WHERE id_dummy = ?"
    }
}
