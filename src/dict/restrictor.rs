use serde::Deserialize;
use lazy_static::lazy_static;

#[derive(Deserialize)]
pub(crate) struct Restrictor {
    pub(crate) blocked_keywords: Vec<String>,
}

impl Restrictor {
    pub(crate) fn is_restricted(&self, text: &str) -> bool {
        for keyword in &self.blocked_keywords {
            if text.contains(keyword) { return true }
        }
        false
    }
}

lazy_static! {
    pub(crate) static ref RESTRICTOR: Restrictor = {
        let json = include_str!("../../restricted.json");
        serde_json::from_str(json).expect("invalid restricted.json")
    };
}