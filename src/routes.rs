use std::{path::Path, time::{UNIX_EPOCH, SystemTime}};

use askama::Template;
use rocket::{fs::NamedFile, response::content::{RawHtml, RawJson}};
use rocket_db_pools::{Connection, deadpool_redis::redis::AsyncCommands};

use crate::{dict::{Word, update_word}, Redis};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "define.html")]
struct DefineTemplate {
    word: String,
}

#[get("/")]
pub(crate) fn index() -> Option<RawHtml<String>> {
    IndexTemplate.render().ok().map(|text| RawHtml(text))
}

#[get("/define/<word>")]
pub(crate) fn define(word: String) -> Option<RawHtml<String>> {
    DefineTemplate { word }.render().ok().map(|text| RawHtml(text))
}

#[get("/api/define/<word>")]
pub(crate) async fn api(mut db: Connection<Redis>, word: String) -> Option<RawJson<String>> {
    db.incr(format!("lookups:{}", &word), 1)
        .await
        .unwrap_or_else(|err| {
            println!("hset error: {}", err);
        });

    let db_key = format!("word:{}", &word);
    let word_data = {
        if !db.exists(&db_key).await.unwrap_or_else(|err| {
            println!("exists error: {}", err);
            false
        }) {
            update_word(db, &db_key, &word).await
        } else {
            let json: String = db.get(&db_key).await.ok().unwrap();

            if let Ok(old_word) = serde_json::from_str::<Word>(&json) {
                let last_updated: u128 = old_word.last_updated.parse().ok().unwrap();
                let now = SystemTime::now();
                let now = now
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                let update_period = 1000 * 60 * 60 * 24 * 30; // one full month
    
                if now.as_millis() - last_updated > update_period {
                    update_word(db, &db_key, &word).await
                } else {
                    json
                }
            } else {
                update_word(db, &db_key, &word).await
            }

        }
    };
    
    Some(RawJson(word_data))
}

#[get("/<file>")]
pub(crate) async fn res(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}
