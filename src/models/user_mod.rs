use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile_pic: String,
    pub is_admin: bool,
}