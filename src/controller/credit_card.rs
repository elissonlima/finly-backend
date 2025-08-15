use sqlx::PgPool;

use crate::model::CreditCard;



pub struct CreditCardController<'a> {
    db_conn: &'a PgPool,
    user_id: &'a i32
}

impl<'a> CreditCardController <'a> {
    pub fn new(db_conn: &'a PgPool, user_id: &'a i32) -> CreditCardController<'a> {
        CreditCardController { db_conn, user_id }
    }

    pub async fn create(
        &self,
        name: &str,
        color: &str,
        closing_day: &i16
    ) -> Result<CreditCard, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "credit_card" (user_id, name, closing_day, color)
                VALUES ($1, $2, $3, $4) RETURNING id, is_default, created_at, updated_at;
            "#,
            self.user_id,
            name,
            closing_day,
            color
        )
        .fetch_one(self.db_conn)
        .await?;

        let card = CreditCard {
            id: rec.id,
            user_id: *self.user_id,
            name: String::from(name),
            color: String::from(color),
            closing_day: *closing_day,
            is_default: rec.is_default,
            created_at: rec.created_at,
            updated_at: rec.updated_at
        };

        return Ok(card);
    }

    pub async fn list(&self) -> Result<Vec<CreditCard>, sqlx::Error> {
        let rec = sqlx::query_as!(
            CreditCard,
            r#"
                SELECT id, user_id, name, color, closing_day, is_default, created_at, updated_at
                FROM "credit_card" WHERE user_id = $1 AND is_active = true;
            "#,
            self.user_id
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)
    }
}