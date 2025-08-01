use sqlx::PgPool;

use crate::model::Category;

pub struct CategoryController<'a> {
    db_conn: &'a PgPool,
    user_id: &'a i32,
}

impl<'a> CategoryController<'a> {
    pub fn new(db_conn: &'a PgPool, user_id: &'a i32) -> CategoryController<'a> {
        CategoryController { db_conn, user_id }
    }

    pub async fn create(&self, name: &str) -> Result<Category, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "category" (user_id, name)
                VALUES ($1, $2) RETURNING id, created_at, updated_at;
            "#,
            self.user_id,
            name
        )
        .fetch_one(self.db_conn)
        .await?;

        let cat = Category {
            id: rec.id,
            user_id: *self.user_id,
            name: String::from(name),
            created_at: rec.created_at,
            updated_at: rec.updated_at,
        };

        return Ok(cat);
    }

    pub async fn list(&self) -> Result<Vec<Category>, sqlx::Error> {
        let rec = sqlx::query_as!(
            Category,
            r#"
                SELECT id, user_id, name, created_at, updated_at
                FROM "category"
                WHERE user_id = $1;
            "#,
            self.user_id
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)
    }
}

