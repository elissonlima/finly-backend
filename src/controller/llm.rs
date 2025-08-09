use std::fmt::Write;

use derive_more::Display;

use crate::{controller::{exec_prompt, PromptCase}, model::{GoogleServiceToken, LLMGeneratedResponse, LLMResponse, LLMCategoryIconResponse}};

#[derive(Debug,Display)]
pub enum LLMControllerError {
    ExecutionError,
    ParserError
}

pub struct LLMController<'a> {
    service_token: &'a GoogleServiceToken
}

impl<'a> LLMController<'a> {

    pub fn new(service_token: &'a GoogleServiceToken) -> LLMController<'a> {
        LLMController { service_token }
    }

    pub async fn exec_usr_json(&self, user_input: &str) -> Result<LLMGeneratedResponse, LLMControllerError> {

        let llm_res = match exec_prompt(
            user_input,
            self.service_token,
            PromptCase::UserInputToJson).await {
            Ok(r) => r.unwrap_or_default(),
            Err(e) => {
                log::error!("An error occurred while executing prompt: {}", e);
                return Err(LLMControllerError::ExecutionError)
            }
        };

        Ok(self.parse_llm_response(llm_res)?)
    }

    pub async fn exec_cat_icon(&self, user_input: &str, icon_list: &str) -> Result<String, LLMControllerError> {
        let llm_res = match exec_prompt(
            user_input,
            self.service_token,
            PromptCase::CategoryIconExtract(String::from(icon_list))).await {
            Ok(r) => r.unwrap_or_default(),
            Err(e) => {
                log::error!("An error occurred while executing prompt: {}", e);
                return Err(LLMControllerError::ExecutionError)
            }
        };

        let llm_str = self.build_json(llm_res)?;
        //log::debug!("LLM Response: \n\n{}\n\n", llm_str);
        let llm_obj: LLMCategoryIconResponse = match serde_json::from_str(&llm_str) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not deserializar LLM Category Icon Response: {}", e);
                return Err(LLMControllerError::ParserError);
            }
        };

        Ok(llm_obj.icon_name)
    }

    fn build_json(&self, llm_res: Vec<LLMResponse>) -> Result<String, LLMControllerError> {
        let mut llm_built_json = String::new();
        for res in llm_res {
            for candidate in res.candidates {
                for part in candidate.content.parts {
                    match write!(&mut llm_built_json, "{}", part.text) {
                        Ok(()) => {}
                        Err(_) => return Err(LLMControllerError::ParserError),
                    }
                }
            }
        }

        Ok(llm_built_json)
    }

    fn parse_llm_response(&self, llm_res: Vec<LLMResponse>) -> Result<LLMGeneratedResponse, LLMControllerError> {
        let llm_built_json = self.build_json(llm_res)?;
        let llm_response_object: LLMGeneratedResponse = match serde_json::from_str(&llm_built_json){
            Ok(e) => e,
            Err(e) => {
                log::error!("Could not serialize LLM Generated Response: {}", e);
                return Err(LLMControllerError::ParserError); 
            }
        };

        Ok(llm_response_object)
    } 
    
}