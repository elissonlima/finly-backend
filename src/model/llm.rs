use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CommandType {
    Create,
    List
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObjectType {
    Category,
    Expense,
    CreditCard
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMGeneratedResponse {
    pub command: CommandType,
    pub object: ObjectType,
    pub description: String,
    pub value: Option<f32>,
    pub category: Option<String>,
    pub credit_card: Option<String>,
    pub period: Option<String>,
    pub frequency: Option<String>
}
