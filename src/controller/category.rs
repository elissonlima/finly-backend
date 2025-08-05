use rand::seq::IndexedRandom;
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

    pub async fn get_random_category_color(&self) -> Result<String, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                SELECT color_hex FROM category_possible_colors;
            "#
        )
        .fetch_all(self.db_conn)
        .await?;

        let random_color = match rec.choose(&mut rand::rng()) {
            Some(i)  => i.color_hex.as_str(),
            None => rec[0].color_hex.as_str()
        };

        return Ok(String::from(random_color));
    }

    pub async fn create(&self, name: &str, icon_id: &i32, color: &str) -> Result<Category, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "category" (user_id, name, icon_id, color)
                VALUES ($1, $2, $3, $4) RETURNING id, created_at, updated_at;
            "#,
            self.user_id,
            name,
            icon_id,
            color
        )
        .fetch_one(self.db_conn)
        .await?;

        let cat = Category {
            id: rec.id,
            user_id: *self.user_id,
            name: String::from(name),
            icon_id: *icon_id,
            color: String::from(color),
            created_at: rec.created_at,
            updated_at: rec.updated_at,
        };

        return Ok(cat);
    }

    pub async fn list(&self) -> Result<Vec<Category>, sqlx::Error> {
        let rec = sqlx::query_as!(
            Category,
            r#"
                SELECT id, user_id, icon_id, name, color, created_at, updated_at
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

