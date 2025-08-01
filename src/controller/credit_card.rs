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
    ) -> Result<CreditCard, sqlx::Error> {
        let default_closing_day: i16 = 1;
        let rec = sqlx::query!(
            r#"
                INSERT INTO "credit_card" (user_id, name, closing_day)
                VALUES ($1, $2, $3) RETURNING id, created_at, updated_at;
            "#,
            self.user_id,
            name,
            default_closing_day)
        .fetch_one(self.db_conn)
        .await?;

        let card = CreditCard {
            id: rec.id,
            user_id: *self.user_id,
            name: String::from(name),
            closing_day: default_closing_day,
            created_at: rec.created_at,
            updated_at: rec.updated_at
        };

        return Ok(card);
    }

    pub async fn list(&self) -> Result<Vec<CreditCard>, sqlx::Error> {
        let rec = sqlx::query_as!(
            CreditCard,
            r#"
                SELECT id, user_id, name, closing_day, created_at, updated_at
                FROM "credit_card" WHERE user_id = $1;
            "#,
            self.user_id
        )
        .fetch_all(self.db_conn)
        .await?;

        Ok(rec)
    }
}