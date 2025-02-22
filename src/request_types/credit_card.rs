use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpsertCreditCardReq {
    pub id: Uuid,
    pub name: String,
    pub icon_name: String,
    pub limit_value: i64,
    pub closing_day: i16,
}

impl From<web::Json<UpsertCreditCardReq>> for UpsertCreditCardReq {
    fn from(value: web::Json<UpsertCreditCardReq>) -> Self {
        return UpsertCreditCardReq {
            id: value.id,
            name: value.name.clone(),
            icon_name: value.icon_name.clone(),
            limit_value: value.limit_value,
            closing_day: value.closing_day,
        };
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteCreditCardReq {
    pub credit_card_id: Uuid,
}

impl From<web::Json<DeleteCreditCardReq>> for DeleteCreditCardReq {
    fn from(value: web::Json<DeleteCreditCardReq>) -> Self {
        DeleteCreditCardReq {
            credit_card_id: value.credit_card_id,
        }
    }
}

#[derive(Serialize)]
pub struct ListCreditCardsRes {
    pub id: Uuid,
    pub name: String,
    pub icon_name: String,
    pub limit_value: i64,
    pub closing_day: i16,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateBillAtDateReq {
    pub credit_card_id: Uuid,
    pub date: String,
}

impl From<web::Json<CreateBillAtDateReq>> for CreateBillAtDateReq {
    fn from(value: web::Json<CreateBillAtDateReq>) -> Self {
        CreateBillAtDateReq {
            credit_card_id: value.credit_card_id,
            date: value.date.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct CreateBillRes {
    pub id: Uuid,
    pub credit_card_id: Uuid,
    pub start_at: String,
    pub end_at: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListCreditCardBillsReq {
    pub credit_card_id: Uuid,
}

impl From<web::Json<ListCreditCardBillsReq>> for ListCreditCardBillsReq {
    fn from(value: web::Json<ListCreditCardBillsReq>) -> Self {
        ListCreditCardBillsReq {
            credit_card_id: value.credit_card_id.clone(),
        }
    }
}
