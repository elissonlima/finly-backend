use sqlx::PgPool;

use crate::{controller::{llm_processors::errors::LLMProcessorError, CategoryController, LLMController}, model::{CategoryIcon, CommandType, GoogleServiceToken, LLMGeneratedResponse, XmlIcon}};


pub struct LLMCategoryProcessor<'a> {
    db_conn: &'a PgPool,
    user_id: &'a i32,
    user_input: &'a str,
    service_token: &'a GoogleServiceToken
}

impl <'a> LLMCategoryProcessor<'a> {

    pub fn new(
        db_conn: &'a PgPool,
        user_id: &'a i32,
        service_token: &'a GoogleServiceToken,
        user_input: &'a str) -> LLMCategoryProcessor<'a> {
        LLMCategoryProcessor { db_conn, user_id, user_input, service_token }
    }

    pub async fn process(&self, llm_response: &LLMGeneratedResponse) -> Result<serde_json::Value, LLMProcessorError> {
        let res = match llm_response.command {
            CommandType::Create => serde_json::json!(self.create(llm_response).await?),
            CommandType::List => serde_json::json!({"message": "Not implemented"})
        };

        Ok(res)
    }

    async fn create(&self, llm_response: &LLMGeneratedResponse) -> Result<CategoryIcon, LLMProcessorError> {

        let category_controller = CategoryController::new(self.db_conn, self.user_id);
        let category_icons = match category_controller.get_category_icons().await {
            Ok(v) => {
               v
            }
            Err(e) => {
                log::warn!("It wasn't possible to get list of category icons from database. Applying default value of 'question-mark-circle': {}", e);
                vec![XmlIcon {
                    id: 30,
                    name: String::from("question-mark-circle"),
                    xml: String::from("")
                }]
            }
        };

        let names_s: Vec<String> = category_icons.iter().map(|x| x.name.clone()).collect();
        let names_f =  names_s.join("\n");

        let llm_controller = LLMController::new(self.service_token);
        let icon_result = match llm_controller.exec_cat_icon(self.user_input, &names_f).await {
            Ok(i) => i,
            Err(e) => {
                log::warn!("Error while trying to build guess category icon from user input. Set as question-mark-circle. {}", e);
                "question-mark-circle".to_string()
            }
        };

        let color = match category_controller.get_random_category_color().await {
            Ok(c) => c,
            Err(e) => {
                log::warn!("It wasn't possible to get random color number for new category: {}", e);
                String::from("#E53E3E")
            }
        };

        let cat = match category_controller.create_from_prompt(
            &llm_response.description,
            &icon_result,
            &color).await{
                Ok(c) => c,
                Err(e) => {
                    log::error!("Could not insert category from prompt into database: {}", e);
                    return Err(LLMProcessorError::DatabaseError)
                }
        };

        let xml_icon = category_icons
            .iter()
            .find(|&x| x.id == cat.icon_id)
            .unwrap_or(&category_icons[0])
            .xml
            .clone()
            ;

        let res = CategoryIcon {
            id: cat.id,
            user_id: cat.user_id,
            name: cat.name,
            xml_icon: xml_icon,
            color: cat.color,
            created_at: cat.created_at,
            updated_at: cat.updated_at
        };
        
        Ok(res)
    }

    /*async fn list(&self, llm_response: LLMGeneratedResponse) -> Result<Vec<Category>, LLMProcessorError> {
        
        Err(LLMProcessorError::NotImplementedError)
    }*/
}