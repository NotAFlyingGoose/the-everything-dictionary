mod restrictor;
mod scrape;

use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

pub(crate) use restrictor::*;
pub(crate) use scrape::*;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Origin {
    pub(crate) part_of_speech: String,
    pub(crate) origin: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Word {
    pub(crate) overview: Vec<String>,
    pub(crate) vocab_defs: Vec<Definition>,
    pub(crate) macmillan_defs: Vec<Vec<Definition>>,
    pub(crate) wiki_defs: Vec<Definition>,

    pub(crate) etym_origins: Vec<Origin>,
    pub(crate) wiki_origins: Vec<Origin>,

    pub(crate) stock_images: Vec<String>,

    pub(crate) sources: Vec<String>,

    pub(crate) last_updated: String,
    version_0_1_2: String,
}

impl Word {
    pub(crate) async fn scrape(word: &str) -> Option<Self> {
        let mut sources = Vec::new();

        let (short, long, vocab_defs, source) =
            scrape_vocab(&word)
                .await
                .unwrap_or((None, None, Vec::new(), ""));

        let mut overview = Vec::new();
        if let Some(short) = short {
            overview.push(short)
        }
        if let Some(long) = long {
            overview.push(long)
        }
        if !source.is_empty() {
            sources.push(source.to_string())
        }

        let (macmillan_defs, source) = scrape_macmillan(&word).await.unwrap_or((Vec::new(), ""));

        if !source.is_empty() {
            sources.push(source.to_string())
        }

        let (wiki_origins, wiki_defs, source) =
            scrape_wiki(&word)
                .await
                .unwrap_or((Vec::new(), Vec::new(), ""));

        if !source.is_empty() {
            sources.push(source.to_string())
        }

        // check for no defs
        if vocab_defs.is_empty() && macmillan_defs.is_empty() && wiki_defs.is_empty() {
            return None;
        }

        let (etym_origins, source) = scrape_etym(&word).await.unwrap_or((Vec::new(), ""));

        if !source.is_empty() {
            sources.push(source.to_string())
        }

        let (stock_images, source) = scrape_stock(&word).await.unwrap_or((Vec::new(), ""));

        if !source.is_empty() {
            sources.push(source.to_string())
        }

        let now = SystemTime::now();
        let now = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        Some(Word {
            overview,
            vocab_defs,
            macmillan_defs,
            wiki_defs,

            wiki_origins,
            etym_origins,

            stock_images,

            sources,

            last_updated: now.as_millis().to_string(),
            version_0_1_2: String::new(),
        })
    }
}
