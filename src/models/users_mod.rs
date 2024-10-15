#![allow(unreachable_code)]

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use mongodb::bson::{bson, oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

// const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Users {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub profile_pic: Option<String>,
    pub is_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<BsonDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<BsonDateTime>,
}

impl From<HashMap<String, String>> for Users {
    fn from(data: HashMap<String, String>) -> Self {
        Users {
            id: data.get("_id").and_then(|s| ObjectId::parse_str(s).ok()),
            username: data.get("username").cloned(),
            email: data.get("email").cloned(),
            password: data.get("password").cloned().unwrap_or_default(),
            profile_pic: data.get("profile_pic").cloned(),
            is_admin: data.get("is_admin").and_then(|s| s.parse::<bool>().ok()),
            created_at: todo!(),
            updated_at: todo!(),
        }
    }
}

