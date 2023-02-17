use std::{path::Path, time::{UNIX_EPOCH, SystemTime}};

use askama::Template;
use rocket::{fs::NamedFile, response::content::{RawHtml, RawJson}};
use rocket_db_pools::{Connection, deadpool_redis::redis::AsyncCommands};

use crate::{dict::Word, Redis};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "define.html")]
struct DefineTemplate {
    word: String,
    is_capital: bool,
}

#[get("/")]
pub(crate) fn index() -> Option<RawHtml<String>> {
    IndexTemplate.render().ok().map(|text| RawHtml(text))
}

#[get("/define/<word>")]
pub(crate) fn define(word: String) -> Option<RawHtml<String>> {
    let is_capital = word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
    let def = DefineTemplate {
        word,
        is_capital,
    };
    def.render().ok().map(|text| RawHtml(text))
}

#[get("/api/<word>")]
pub(crate) async fn api(mut db: Connection<Redis>, word: String) -> Option<RawJson<String>> {
    let db_key = format!("word:{}", &word);

    let word_data = {
        if !db.exists(&db_key).await.unwrap_or_else(|err| {
            println!("exists error: {}", err);
            false
        }) {
            let new_word = Word::scrape(&word).await?;

            let json = serde_json::to_string(&new_word).unwrap_or_else(|err| {
                println!("Error parsing word into json: {}", err);
                "{}".to_string()
            });

            db.set(&db_key, &json)
                .await
                .unwrap_or_else(|err| {
                    println!("hset error: {}", err);
                });
            
            json
        } else {
            let json: String = db.get(&db_key).await.ok()?;

            let old_word: Word = serde_json::from_str(&json).ok()?;
            let last_updated: u128 = old_word.last_updated.parse().ok()?;
            let now = SystemTime::now();
            let now = now
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let update_period = 1000 * 60 * 60 * 24; // one full day

            if now.as_millis() - last_updated > update_period {
                println!("updating entry");
                let new_word = Word::scrape(&word).await?;

                let json = serde_json::to_string(&new_word).unwrap_or_else(|err| {
                    println!("Error parsing word into json: {}", err);
                    "{}".to_string()
                });
    
                db.set(&db_key, &json)
                    .await
                    .unwrap_or_else(|err| {
                        println!("hset error: {}", err);
                    });
                
                json
            } else {
                json
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
