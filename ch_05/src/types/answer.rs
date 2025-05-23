use serde::{Serialize, Deserialize};
use crate::types::question::QuestionId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub String);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}