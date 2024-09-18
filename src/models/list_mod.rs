use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct List {
    pub title: String,
    pub type_list: String,
    pub genre: String,
    pub content: Vec<String>,
}
