use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Display, Serialize)]
pub enum LLMProcessorError {
    DatabaseError
}