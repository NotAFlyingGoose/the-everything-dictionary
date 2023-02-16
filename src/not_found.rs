use askama::Template;
use rand::Rng;
use rocket::response::content::{RawJson, RawHtml};

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {
    emoji: String,
}

const EMOJIS: &[&str] = &[
    "pwp",
    "TwT",
    "x-x",
    "<(X_X)>",
    "-w-",
    "(>_<)",
    "(·.·)",
    "(≥o≤)",
    "(·_·)",
    "\\(o_o)/",
    "(;-;)",
];

#[catch(404)]
pub(crate) fn general_not_found() -> Option<RawHtml<String>> {
    let mut rng = rand::thread_rng();
    let emoji = EMOJIS[rng.gen_range(0..EMOJIS.len())].to_string();
    NotFoundTemplate { emoji }.render().ok().map(|text| RawHtml(text))
}

#[catch(404)]
pub(crate) fn api_not_found() -> RawJson<&'static str> {
    RawJson("{}")
}
