use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use askama::Template;
use rocket::{
    fs::NamedFile,
    response::content::{RawHtml, RawJson},
};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use tokio::sync::OnceCell;

use crate::{
    dict::{update_word, Word, RESTRICTOR},
    Redis,
};

#[derive(Debug)]
struct WordRanking {
    name: String,
    count: i32,
}

#[derive(Template)]
#[template(path = "guantanamo_bay.html")]
struct GuantanamoBayTemplate<'a> {
    words: &'a Vec<WordRanking>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "define.html")]
struct DefineTemplate {
    word: String,
}

static TOP_WORDS: OnceCell<Vec<WordRanking>> = OnceCell::const_new();

#[get("/")]
pub(crate) async fn guantanamo_bay(mut db: Connection<Redis>) -> Option<RawHtml<String>> {
    let words = match TOP_WORDS.get() {
        Some(words) => words,
        None => {
            let keys: Vec<String> = db.keys("lookups:*".to_string()).await.unwrap();
            let lookups: Vec<String> = db.get(&keys).await.unwrap();

            let mut words: Vec<WordRanking> = keys
                .iter()
                .zip(lookups.iter())
                .map(|(key, lookups)| WordRanking {
                    name: key.split_once(':').unwrap().1.to_string(),
                    count: lookups.parse().unwrap(),
                })
                .filter(|word| !RESTRICTOR.is_restricted(&word.name))
                .collect();
            words.sort_by_key(|word| word.count);
            words.reverse();

            TOP_WORDS.set(words).unwrap();

            TOP_WORDS.get().unwrap()
        }
    };

    GuantanamoBayTemplate { words }.render().ok().map(RawHtml)
}

#[get("/")]
pub(crate) fn index() -> Option<RawHtml<String>> {
    IndexTemplate.render().ok().map(RawHtml)
}

#[get("/define/<word>")]
pub(crate) fn define(word: String) -> Option<RawHtml<String>> {
    DefineTemplate { word }.render().ok().map(RawHtml)
}

#[get("/api/define/<word>")]
pub(crate) async fn api(mut db: Connection<Redis>, word: String) -> Option<RawJson<String>> {
    let db_key = format!("word:{}", &word);
    let word_data = {
        if !db.exists(&db_key).await.unwrap_or_else(|err| {
            println!("exists error: {}", err);
            false
        }) {
            update_word(&mut db, &db_key, &word).await?
        } else {
            let json: String = db.get(&db_key).await.ok().unwrap();

            if let Ok(old_word) = serde_json::from_str::<Word>(&json) {
                let last_updated: u128 = old_word.last_updated.parse().ok().unwrap();
                let now = SystemTime::now();
                let now = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
                let update_period = if cfg!(debug_assertions) {
                    0
                } else {
                    1000 * 60 * 60 * 24 * 30 // one full month
                };

                if now.as_millis() - last_updated > update_period {
                    update_word(&mut db, &db_key, &word).await?
                } else {
                    json
                }
            } else {
                update_word(&mut db, &db_key, &word).await?
            }
        }
    };

    db.incr(format!("lookups:{}", &word), 1)
        .await
        .unwrap_or_else(|err| {
            println!("hset error: {}", err);
        });

    Some(RawJson(word_data))
}

#[get("/<file>")]
pub(crate) async fn res(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}
