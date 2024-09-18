use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Movie {
    pub title: String,
    pub desc: String,
    pub img: String,
    pub img_title: String,
    pub img_sm: String,
    pub trailer: String,
    pub video: String,
    pub year: String,
    pub limit: String,
    pub genre: String,
    pub is_series: bool,
}
