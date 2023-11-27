use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use askama::Template;
use rocket::{
    fs::NamedFile,
    response::content::{RawHtml, RawJson},
};
use tokio::sync::OnceCell;

use crate::dict::{Word, RESTRICTOR};

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
pub(crate) async fn guantanamo_bay() -> Option<RawHtml<String>> {
    /* let words = match TOP_WORDS.get() {
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
    }; */
    let words = Vec::new();

    GuantanamoBayTemplate { words: &words }
        .render()
        .ok()
        .map(RawHtml)
}

#[get("/")]
pub(crate) fn index() -> Option<RawHtml<String>> {
    IndexTemplate.render().ok().map(RawHtml)
}

#[get("/define/<word>")]
pub(crate) fn define(word: String) -> Option<RawHtml<String>> {
    DefineTemplate { word }.render().ok().map(RawHtml)
}

#[get("/<file>")]
pub(crate) async fn res(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}
