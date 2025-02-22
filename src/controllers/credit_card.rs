use chrono::{FixedOffset, NaiveDate};
use uuid::Uuid;

use crate::model::{credit_card::CreditCard, credit_card_bill::CreditCardBill};

pub async fn upsert_credit_card<'a, T>(
    credit_card: &CreditCard,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            INSERT INTO credit_card
                (id, user_id, name, icon_name,
                limit_value, closing_day)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(id, user_id)
            DO UPDATE
                SET name = $3,
                    icon_name = $4,
                    limit_value = $5,
                    updated_at = (now() at time zone 'utc')
        "#,
        credit_card.id,
        credit_card.user_id,
        credit_card.name,
        credit_card.icon_name,
        credit_card.limit_value,
        credit_card.closing_day
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn delete_credit_card<'a, T>(
    credit_card_id: Uuid,
    user_id: i32,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            UPDATE credit_card
                SET is_active = false,
                    updated_at = (now() at time zone 'utc')
                WHERE id = $1 and user_id = $2 and is_active = true
        "#,
        credit_card_id,
        user_id
    )
    .execute(con)
    .await?;
    Ok(())
}

pub async fn get_credit_card_by_id<'a, T>(
    credit_card_id: &Uuid,
    user_id: i32,
    con: T,
) -> Result<Option<CreditCard>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
            SELECT
                id,
                user_id,
                name,
                icon_name,
                limit_value,
                closing_day
            FROM credit_card
            WHERE user_id = $1 and is_active = true and 
            id = $2 LIMIT 1;
        "#,
        user_id,
        credit_card_id
    )
    .fetch_optional(con)
    .await?;

    let card = match res {
        Some(r) => Some(CreditCard {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            icon_name: r.icon_name,
            limit_value: r.limit_value,
            closing_day: r.closing_day,
        }),
        None => None,
    };

    Ok(card)
}

pub async fn get_credit_cards_by_user_id<'a, T>(
    user_id: i32,
    con: T,
) -> Result<Vec<CreditCard>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
            SELECT
                id,
                user_id,
                name,
                icon_name,
                limit_value,
                closing_day
            FROM credit_card
            WHERE user_id = $1 AND is_active = true
        "#,
        user_id
    )
    .fetch_all(con)
    .await?;

    let mut res: Vec<CreditCard> = Vec::new();

    for row in rows {
        res.push(CreditCard {
            id: row.id,
            user_id: row.user_id,
            name: row.name.clone(),
            icon_name: row.icon_name.clone(),
            limit_value: row.limit_value,
            closing_day: row.closing_day,
        });
    }

    Ok(res)
}

pub async fn create_bill_of_date<'a, T>(
    credit_card_id: &Uuid,
    credit_card_closing_day: i16,
    dt: NaiveDate,
    offset: FixedOffset,
    con: T,
) -> Result<CreditCardBill, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let bill = CreditCardBill::of_date(credit_card_id, credit_card_closing_day, dt, offset);

    let _ = sqlx::query!(
        r#"
        INSERT INTO credit_card_bill
            (id, credit_card_id, start_at, end_at)
        VALUES
            ($1, $2, $3, $4);
    "#,
        bill.id,
        credit_card_id,
        bill.start_at.naive_utc(),
        bill.end_at.naive_utc()
    )
    .execute(con)
    .await?;

    Ok(bill)
}

pub async fn get_credit_card_bills<'a, T>(
    credit_card_id: &Uuid,
    user_id: i32,
    con: T,
) -> Result<Vec<CreditCardBill>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
        SELECT
            b.id,
            b.credit_card_id,
            b.start_at,
            b.end_at
        FROM credit_card_bill b
        INNER JOIN credit_card c ON
            c.id = b.credit_card_id 
        WHERE b.credit_card_id = $1 AND c.user_id = $2
            AND c.is_active = true
    "#,
        credit_card_id,
        user_id
    )
    .fetch_all(con)
    .await?;
    let mut res: Vec<CreditCardBill> = Vec::new();
    for row in rows {
        res.push(CreditCardBill {
            id: row.id,
            credit_card_id: row.credit_card_id,
            start_at: row.start_at.and_utc(),
            end_at: row.end_at.and_utc(),
        });
    }

    Ok(res)
}
