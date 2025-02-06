use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::model::{category::Category, subcategory::Subcategory};

pub async fn upsert_category<'a, T>(
    category: &Category,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
        INSERT INTO category
            (id, user_id, name, color, icon_name,
             created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5,
                (now() at time zone 'utc'),
                (now() at time zone 'utc'))
        ON CONFLICT(id, user_id)
        DO UPDATE
            SET name = $3,
                color = $4,
                icon_name = $5,
                updated_at = (now() at time zone 'utc')
        "#,
        category.id,
        category.user_id,
        category.name,
        category.color,
        category.icon_name,
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn delete_category<'a, T>(
    category_id: &Uuid,
    user_id: i32,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let now = Utc::now().naive_utc();

    let _ = sqlx::query!(
        r#"
            UPDATE category
                SET is_active = false,
                    updated_at = $3
            WHERE id = $1 and user_id = $2 and is_active = true
        "#,
        category_id,
        user_id,
        now
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn upsert_subcategory<'a, T>(
    subcategory: &Subcategory,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
        INSERT INTO subcategory
            (id, category_id, name, color, icon_name,
             created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5,
                (now() at time zone 'utc'),
                (now() at time zone 'utc'))
        ON CONFLICT(id, category_id)
            DO UPDATE
            SET name = $3,
                color = $4,
                icon_name = $5,
                updated_at = (now() at time zone 'utc')
    "#,
        subcategory.id,
        subcategory.category_id,
        subcategory.name,
        subcategory.color,
        subcategory.icon_name,
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn delete_subcategory<'a, T>(
    subcategory_id: &Uuid,
    category_id: &Uuid,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
        UPDATE subcategory
            SET is_active = false, updated_at = (now() at time zone 'utc')
        WHERE id = $1 AND category_id = $2 
            AND is_active = true
    "#,
        subcategory_id,
        category_id,
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn get_categories_by_user_id<'a, T>(
    user_id: i32,
    con: T,
) -> Result<Vec<Category>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
            SELECT
                id,
                user_id,
                name,
                color,
                icon_name
            FROM category
            WHERE user_id = $1 AND is_active = true
        "#,
        user_id
    )
    .fetch_all(con)
    .await?;

    let mut res: Vec<Category> = Vec::new();

    for row in rows {
        let cat = Category {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            color: row.color,
            icon_name: row.icon_name,
            subcategories: Vec::new(),
        };
        res.push(cat);
    }

    Ok(res)
}

pub async fn get_subcategories_by_user_id<'a, T>(
    user_id: i32,
    con: T,
) -> Result<HashMap<String, Vec<Subcategory>>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
            SELECT
                s.id,
                s.category_id,
                s.name,
                s.color,
                s.icon_name
            FROM subcategory s
            INNER JOIN category c
                ON c.id = s.category_id
            WHERE
                c.user_id = $1 AND s.is_active = true
                AND c.is_active = true
        "#,
        user_id
    )
    .fetch_all(con)
    .await?;

    let mut res: HashMap<String, Vec<Subcategory>> = HashMap::new();

    for row in rows {
        let subc = Subcategory {
            id: row.id,
            category_id: row.category_id.clone(),
            name: row.name,
            color: row.color,
            icon_name: row.icon_name,
        };

        res.entry(row.category_id.to_string())
            .and_modify(|c| c.push(subc.clone()))
            .or_insert_with(|| {
                let mut v: Vec<Subcategory> = Vec::new();
                v.push(subc);
                v
            });
    }

    Ok(res)
}
