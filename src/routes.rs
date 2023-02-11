use std::path::Path;

use askama::{Template};
use rocket::{fs::NamedFile, response::content::{RawHtml, RawJson}};
use urlencoding::encode;

use crate::dict;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "define.html")]
struct DefineTemplate {
    title: String,
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
        title: word,
        is_capital,
    };
    def.render().ok().map(|text| RawHtml(text))
}

#[get("/api/<word>")]
pub(crate) async fn api(word: String) -> RawJson<String> {
    let mut sources = Vec::new();

    let mut json = format!("{{\"{}\":{{", word);

    if let Some((short, long, defs, source)) = dict::scrape_vocab(&word).await {
        sources.push(source);

        json.push_str("\"overview\":[");
        if let Some(short) = short {
            json.push_str(&format!("\"{}\"", encode(&short).replace("%20", " ")));
        }
        if let Some(long) = long {
            json.push_str(&format!(",\"{}\"", encode(&long).replace("%20", " ")));
        }
        json.push_str("],");

        json.push_str("\"vocab_defs\":[");
        for (idx, def) in defs.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            json.push_str(&format!("{{\"part_of_speech\":\"{}\",\"meaning\":\"{}\",\"examples\":[", def.part_of_speech, def.meaning));

            for (idx, example) in def.examples.iter().enumerate() {
                if idx != 0 {
                    json.push(',');
                }
                json.push_str(&format!("\"{}\"", encode(&example).replace("%20", " ")));
            }

            json.push_str("]}");
        }
        json.push_str("],");
    }

    if let Some((origins, defs, source)) = dict::scrape_wiki(&word).await {
        sources.push(source);

        json.push_str("\"wiki_defs\":[");
        for (idx, def) in defs.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            json.push_str(&format!("{{\"part_of_speech\":\"{}\",\"meaning\":\"{}\",\"examples\":[", def.part_of_speech, encode(&def.meaning).replace("%20", " ")));

            for (idx, example) in def.examples.iter().enumerate() {
                if idx != 0 {
                    json.push(',');
                }
                json.push_str(&format!("\"{}\"", encode(&example).replace("%20", " ")));
            }

            json.push_str("]}");
        }
        json.push_str("],");
    
        json.push_str("\"wiki_origins\":[");
        for (idx, origin) in origins.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"part_of_speech\":\"{}\",\"origin\":\"{}\"}}", 
                origin.part_of_speech, 
                encode(&origin.origin).replace("%20", " ")
            ));
        }
        json.push_str("],");
    }

    if sources.is_empty() {
        if let Some((defs, source)) = dict::scrape_slang(&word).await {
            sources.push(source);

            json.push_str("\"urban_defs\":[");
            for (idx, def) in defs.iter().enumerate() {
                if idx != 0 {
                    json.push(',');
                }
                json.push_str(&format!("{{\"part_of_speech\":\"{}\",\"meaning\":\"{}\",\"examples\":[", def.part_of_speech, encode(&def.meaning).replace("%20", " ")));

                for (idx, example) in def.examples.iter().enumerate() {
                    if idx != 0 {
                        json.push(',');
                    }
                    json.push_str(&format!("\"{}\"", encode(&example).replace("%20", " ")));
                }

                json.push_str("]}");
            }
            json.push_str("],");
        }
    }

    if let Some((origins, source)) = dict::scrape_etym(&word).await {
        sources.push(source);

        json.push_str("\"etym_origins\":[");
        for (idx, origin) in origins.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"part_of_speech\":\"{}\",\"origin\":\"{}\"}}", 
                origin.part_of_speech, 
                encode(&origin.origin).replace("%20", " ")
            ));
        }
        json.push_str("],");
    }

    if let Some((imgs, source)) = dict::scrape_stock(&word).await {
        sources.push(source);

        json.push_str("\"stock_images\":[");
        for (idx, img) in imgs.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            json.push_str(&format!("\"{}\"", img));
        }
        json.push_str("],");
    }

    json.push_str("\"sources\":[");
    for (idx, source) in sources.iter().enumerate() {
        if idx != 0 {
            json.push(',');
        }
        json.push_str(&format!("\"{}\"", source));
    }
    json.push_str("]}}");
    
    RawJson(json)
}

#[get("/<file>")]
pub(crate) async fn res(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}
