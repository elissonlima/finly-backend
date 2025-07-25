use sqlx::{Postgres, pool::PoolConnection};

use crate::model::Category;

pub struct CategoryController<'a> {
    db_conn: &'a PoolConnection<Postgres>,
}

impl<'a> CategoryController<'a> {
    pub fn new(db_conn: &'a PoolConnection<Postgres>) -> CategoryController<'a> {
        CategoryController { db_conn }
    }

    pub fn create_category(&self, category: &Category) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
