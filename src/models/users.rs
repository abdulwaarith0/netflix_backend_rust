use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Users {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub username: Option<String>,
    pub email: Option<String>,

    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub password: String,

    pub profile_pic: Option<String>,

    #[serde(default)]
    pub is_admin: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<BsonDateTime>,
}

/// Converts JWT claims (HashMap<String, String>) back into a `Users` instance.
impl From<HashMap<String, String>> for Users {
    fn from(data: HashMap<String, String>) -> Self {
        Users {
            id: data.get("_id").and_then(|s| ObjectId::parse_str(s).ok()),
            username: data.get("username").cloned(),
            email: data.get("email").cloned(),
            password: data.get("password").cloned().unwrap_or_default(),
            profile_pic: data.get("profile_pic").cloned(),
            is_admin: data
                .get("is_admin")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(false),
            created_at: None, 
            updated_at: None, 
        }
    }
}