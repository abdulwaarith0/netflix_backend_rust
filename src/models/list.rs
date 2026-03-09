use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_list: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,

    pub content: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}