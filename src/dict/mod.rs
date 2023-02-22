
mod restrictor;
mod scrape;

use std::{fmt::Display, time::{SystemTime, UNIX_EPOCH}};

use rocket_db_pools::{Connection, deadpool_redis::redis::AsyncCommands};
use serde::{Serialize, Deserialize};

pub(crate) use scrape::*;

use crate::Redis;

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
}

impl Word {
    pub(crate) async fn scrape(word: &str) -> Option<Self> {
        let mut sources = Vec::new();

        let (short, long, vocab_defs, source) = 
            scrape_vocab(&word)
                .await
                .unwrap_or((None, None, Vec::new(), ""));
        
        let mut overview = Vec::new();
        if let Some(short) = short { overview.push(short) }
        if let Some(long) = long { overview.push(long) }
        if !source.is_empty() { sources.push(source.to_string()) }

        let (macmillan_defs, source) = 
            scrape_macmillan(&word)
                .await
                .unwrap_or((Vec::new(), ""));

        if !source.is_empty() { sources.push(source.to_string()) }
        
        let (wiki_origins, wiki_defs, source) = 
            scrape_wiki(&word)
                .await
                .unwrap_or((Vec::new(), Vec::new(), ""));
        
        if !source.is_empty() { sources.push(source.to_string()) }

        // check for no defs
        if vocab_defs.is_empty() && wiki_defs.is_empty() {
            return None;
        }
        
        let (etym_origins, source) = 
            scrape_etym(&word)
                .await
                .unwrap_or((Vec::new(), ""));

        if !source.is_empty() { sources.push(source.to_string()) }
        
        let (stock_images, source) = 
            scrape_stock(&word)
                .await
                .unwrap_or((Vec::new(), ""));
        
        if !source.is_empty() { sources.push(source.to_string()) }

        let now = SystemTime::now();
        let now = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

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
        })
    }
}

pub(crate) async fn update_word(mut db: Connection<Redis>, db_key: &str, word: &str) -> Option<String> {
    let new_word = Word::scrape(word).await?;

    let json = serde_json::to_string(&new_word).unwrap_or_else(|err| {
        println!("Error parsing word into json: {}", err);
        "{}".to_string()
    });

    db.set(db_key, &json)
        .await
        .unwrap_or_else(|err| {
            println!("hset error: {}", err);
        });
    
    Some(json)
}