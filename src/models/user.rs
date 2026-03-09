use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub username: Option<String>,

    pub email: String, 

    #[serde(skip_serializing)]
    pub password: String,

    pub profile_pic: Option<String>,

    #[serde(default)] 
    pub is_admin: bool,
}