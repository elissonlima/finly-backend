use rand::seq::IndexedRandom;
use sqlx::PgPool;

use crate::model::{Category, CategoryIcon, XmlIcon};

pub struct CategoryController<'a> {
    db_conn: &'a PgPool,
    user_id: &'a i32,
}

impl<'a> CategoryController<'a> {
    pub fn new(db_conn: &'a PgPool, user_id: &'a i32) -> CategoryController<'a> {
        CategoryController { db_conn, user_id }
    }

    pub async fn get_category_icons(&self) -> Result<Vec<XmlIcon>, sqlx::Error> {
        let rec = sqlx::query_as!(
            XmlIcon,
            r#"
                SELECT id, name, xml FROM xml_icon WHERE "class" = 'CATEGORY';
            "#
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)   
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

    pub async fn create_from_prompt(&self, name: &str, icon_name: &str, color: &str) -> Result<Category, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "category" (user_id, name, icon_id, color)
                SELECT
                    $1 AS "user_id",
                    $2 AS "name",
                    x.id AS icon_id,
                    $3 AS "color"
                FROM xml_icon x
                WHERE x.name = $4
                LIMIT 1
                RETURNING id, icon_id, created_at, updated_at;
            "#,
            self.user_id,
            name,
            color,
            icon_name
        )
        .fetch_one(self.db_conn)
        .await?;

        let cat = Category {
            id: rec.id,
            user_id: *self.user_id,
            name: String::from(name),
            icon_id: rec.icon_id,
            color: String::from(color),
            created_at: rec.created_at,
            updated_at: rec.updated_at,
        };

        return Ok(cat);
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

    pub async fn list(&self) -> Result<Vec<CategoryIcon>, sqlx::Error> {
        let rec = sqlx::query_as!(
            CategoryIcon,
            r#"
                SELECT 
                    c.id,
                    c.user_id,
                    x.xml as xml_icon,
                    c.name,
                    c.color,
                    c.created_at,
                    c.updated_at
                FROM "category" c
                INNER JOIN "xml_icon" x
                    ON x.id = c.icon_id
                WHERE user_id = $1 and is_active = true;
            "#,
            self.user_id
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)
    }

    pub async fn list_category_icons(&self) -> Result<Vec<XmlIcon>, sqlx::Error> {
        let rec = sqlx::query_as!(
            XmlIcon,
            r#"
                SELECT
                    id,
                    name,
                    xml
                FROM "xml_icon" x
                WHERE x.class = 'CATEGORY';
            "#
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)
    }

    pub async fn list_category_icons_id(&self) -> Result<Vec<i32>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                SELECT
                    id
                FROM "xml_icon" x
                WHERE x.class = 'CATEGORY';
            "#
        )
        .fetch_all(self.db_conn)
        .await?;

        let res = rec.iter().map(|i| i.id).collect();

        Ok(res)
    }

    pub async fn update(&self, category_id: &i32, name: &str, icon_id: &i32, color: &str) -> Result<Category, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                UPDATE  "category"
                SET name = $1,
                    icon_id = $2,
                    color  = $3,
                    updated_at = now()
                WHERE id = $4 AND user_id = $5
                RETURNING id, created_at, updated_at;
            "#,
            name,
            icon_id,
            color,
            category_id,
            self.user_id
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

    pub async fn list_category_colors(&self) -> Result<Vec<String>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                SELECT
                    color_hex
                FROM "category_possible_colors";
            "#
        )
        .fetch_all(self.db_conn)
        .await?;

        let res = rec.iter().map(|c| c.color_hex.clone()).collect();

        Ok(res)
    }

    pub async fn delete(&self, category_id: &i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE  "category"
                SET is_active = false,
                    updated_at = now()
                WHERE id = $1 AND user_id = $2;
            "#,
            category_id,
            self.user_id
        )
        .execute(self.db_conn)
        .await?;
    
        Ok(())
    }

}

