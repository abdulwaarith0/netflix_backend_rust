use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub profile_pic: Option<String>,
    pub is_admin: Option<bool>,
}