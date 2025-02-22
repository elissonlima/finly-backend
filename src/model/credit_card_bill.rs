use std::cmp::Ordering;

use chrono::{DateTime, Datelike, Days, FixedOffset, Months, NaiveDate, TimeDelta, Utc};
use uuid::Uuid;

pub struct CreditCardBill {
    pub id: Uuid,
    pub credit_card_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl CreditCardBill {
    pub fn of_date(
        credit_card_id: &Uuid,
        credit_card_closing_day: i16,
        dt: NaiveDate,
        offset: FixedOffset,
    ) -> Self {
        let first_day_current_month = NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1).unwrap();
        let last_day_previous_month = first_day_current_month - TimeDelta::days(1);
        let first_day_next_month = first_day_current_month + Months::new(1);
        let last_day_next_month = first_day_next_month + Months::new(1) - TimeDelta::days(1);
        let last_day_current_month = first_day_next_month - TimeDelta::days(1);

        let start_at_raw = match dt.day().cmp(&(credit_card_closing_day as u32)) {
            Ordering::Less => {
                Self::closing_day_date_compare(last_day_previous_month, credit_card_closing_day)
            }
            Ordering::Greater => {
                Self::closing_day_date_compare(last_day_current_month, credit_card_closing_day)
            }
            Ordering::Equal => {
                Self::closing_day_date_compare(last_day_previous_month, credit_card_closing_day)
            }
        };

        let start_at = match start_at_raw.checked_add_days(Days::new(1)) {
            Some(d) => d,
            None => start_at_raw,
        };

        let end_at = match dt.day().cmp(&(credit_card_closing_day as u32)) {
            Ordering::Less => {
                Self::closing_day_date_compare(last_day_current_month, credit_card_closing_day)
            }
            Ordering::Greater => {
                Self::closing_day_date_compare(last_day_next_month, credit_card_closing_day)
            }
            Ordering::Equal => {
                Self::closing_day_date_compare(last_day_current_month, credit_card_closing_day)
            }
        };

        let start_at_with_tz = start_at
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(offset)
            .unwrap();
        let start_at_utc = start_at_with_tz.to_utc();

        let end_at_with_tz = end_at
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(offset)
            .unwrap();
        let end_at_utc = end_at_with_tz.to_utc();

        CreditCardBill {
            id: Uuid::new_v4(),
            credit_card_id: credit_card_id.clone(),
            start_at: start_at_utc,
            end_at: end_at_utc,
        }
    }

    fn closing_day_date_compare(dt: NaiveDate, closing_day: i16) -> NaiveDate {
        match dt.day().cmp(&(closing_day as u32)) {
            Ordering::Less => dt,
            Ordering::Equal => dt,
            Ordering::Greater => {
                let sub_d = dt.day() - (closing_day as u32);
                match dt.checked_sub_days(Days::new(sub_d as u64)) {
                    Some(d) => d,
                    None => dt,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_credit_card_bill_of_date() {
        //  let closing_day = 20;
        //  let dt1 = NaiveDate::from_ymd_opt(2025, 2, 27).unwrap();
        //  let _ = CreditCardBill::of_date(closing_day, dt1);
        //  println!("------------------------");
        //  let dt2 = NaiveDate::from_ymd_opt(2025, 2, 28).unwrap();
        //  let _ = CreditCardBill::of_date(closing_day, dt2);
        //  println!("------------------------");
        //  let dt3 = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        //  let _ = CreditCardBill::of_date(closing_day, dt3);
        let now = Utc::now();
        println!("{}", now.to_rfc3339());
        println!("{}", now.naive_utc());
        println!("{}", now.naive_utc().and_utc().to_rfc3339());
    }
}
