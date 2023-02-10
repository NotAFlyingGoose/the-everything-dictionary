
mod scrape;

use std::fmt::Display;

pub(crate) use scrape::*;

#[derive(Debug)]
pub(crate) struct Definition {
    pub(crate) part_of_speech: String,
    pub(crate) meaning: String,
    pub(crate) examples: Vec<String>,
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.part_of_speech, self.meaning)?;
        for example in &self.examples {
            write!(f, "\n- {}", example)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Origin {
    pub(crate) part_of_speech: String,
    pub(crate) origin: String,
}
