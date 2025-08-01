use actix_web::{http::{header::ContentType, StatusCode}, HttpResponse};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{controller::{CategoryController, CreditCardController}, 
handler::errors::AppError, model::{CommandType, LLMGeneratedResponse, ObjectType}};

pub struct LLMGenericProcessor<'a> {
    db_conn: &'a PgPool,
    generated_object: &'a LLMGeneratedResponse,
    user_id: &'a i32
}

impl<'a> LLMGenericProcessor<'a> {
    pub fn new(db_conn: &'a PgPool, generated_object: &'a LLMGeneratedResponse, user_id: &'a i32) -> LLMGenericProcessor<'a> {
        LLMGenericProcessor { db_conn, generated_object, user_id }
    }

    async fn process_create(&self) -> Result<HttpResponse, AppError> {

        let response_body = match self.generated_object.object {
            ObjectType::Category => {
                let category_controller = CategoryController::new(self.db_conn, self.user_id);
                let cat = match category_controller.create(&self.generated_object.description).await {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Error trying to create a category from this LLM Object:{:?}, Error: {}", 
                            self.generated_object,
                            e
                        );
                        return Err(AppError::LLMObjectProcessingError)
                    }
                };
                json!(cat)
            },
            ObjectType::CreditCard => {
                let credit_card_controller = CreditCardController::new(self.db_conn, self.user_id);
                let card = match credit_card_controller.create(&self.generated_object.description).await {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Error trying to create a Credit Card from this LLM Object:{:?}, Error: {}", 
                            self.generated_object,
                            e
                        );
                        return Err(AppError::LLMObjectProcessingError)
                    }
                };
                json!(card)
            },
            ObjectType::Expense => json!({"id": Uuid::new_v4(),"message": "Hello",})
        };


        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::json())
            .body(json!({
                "status": "OK",
                "generated_object": response_body, 
                "type": self.generated_object.object}).to_string()))
    }

    async fn process_list(&self) -> Result<HttpResponse, AppError> {
        let response_body = match self.generated_object.object {
            ObjectType::Category => {
                let category_controller = CategoryController::new(self.db_conn, self.user_id);
                match category_controller.list().await {
                    Ok(v) => json!(v),
                    Err(e) => {
                        log::error!("Error trying to list Categories from this LLM Object:{:?}, Error: {}", 
                            self.generated_object,
                            e
                        );
                        return Err(AppError::LLMObjectProcessingError);
                    }
                }
            },
            ObjectType::CreditCard => {
                let credit_card_controller = CreditCardController::new(self.db_conn, self.user_id);
                match credit_card_controller.list().await {
                    Ok(v) => json!(v),
                    Err(e) => {
                        log::error!("Error trying to list Categories from this LLM Object:{:?}, Error: {}", 
                            self.generated_object,
                            e
                        );
                        return Err(AppError::LLMObjectProcessingError);
                    }
                }
            },
            ObjectType::Expense => json!({"id": Uuid::new_v4(),"message": "Hello",})
        };

        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::json())
            .body(json!({
                "status": "OK",
                "generated_object": response_body, 
                "type": format!("LIST<{:?}>", self.generated_object.object)}).to_string()))
    }

    pub async fn process(&self) -> Result<HttpResponse, AppError> {

        match self.generated_object.command {
            CommandType::Create => self.process_create().await,
            CommandType::List => self.process_list().await
        }
        
    }

}