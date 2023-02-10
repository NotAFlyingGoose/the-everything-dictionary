
use ego_tree::NodeRef;
use scraper::{Html, Selector, ElementRef, Node, node::Text};

use super::{Origin, Definition};

macro_rules! find {
    ($element: expr, $selector: literal) => {
        {
            let sel = Selector::parse($selector).unwrap();
            $element.select(&sel).nth(0)
        }
    };
}

const PROTOCOL: &str = "https";
const VOCAB_URL_BASE: &str = "www.vocabulary.com";
const SLANG_URL_BASE: &str = "www.urbandictionary.com";
const WIKI_URL_BASE: &str = "en.wiktionary.org";
const ETYM_URL_BASE: &str = "www.etymonline.com";
const STOCK_URL_BASE: &str = "stock.adobe.com";

pub(crate) async fn scrape_vocab(word: &str) -> Option<(Option<String>, Option<String>, Vec<Definition>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/dictionary/definition.ajax?search={}&lang=en",
        PROTOCOL,
        VOCAB_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    let word_area = find!(doc, ".word-area")?;

    let real_word = el_to_string(*find!(word_area, "h1")?);
    if real_word != word {
        return None;
    }

    let short_overview = Some(el_to_string(*find!(word_area, ".short")?))
        .filter(|s| !s.is_empty());
    let long_overview = Some(el_to_string(*find!(word_area, ".long")?))
        .filter(|s| !s.is_empty());

    let ol = find!(find!(doc, ".word-definitions")?, "ol")?;

    let mut definitions = Vec::new();

    let sel = Selector::parse("li").unwrap();
    for item in ol.select(&sel) {
        let def_area = find!(item, ".definition")?;

        let part_of_speech = el_to_string(*find!(def_area, ".pos-icon")?);

        let meaning = el_to_string(*def_area);

        let mut examples = Vec::new();

        let sel = Selector::parse(".example").unwrap();
        for example in item.select(&sel) {
            examples.push(el_to_string(*example).replace("\n", ""));
        }

        definitions.push(Definition {
            part_of_speech,
            meaning,
            examples,
        });
    }

    Some((short_overview, long_overview, definitions, VOCAB_URL_BASE))
}

pub(crate) async fn scrape_slang(word: &str) -> Option<(Vec<Definition>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/define.php?term={}",
        PROTOCOL,
        SLANG_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    let mut definitions = Vec::new();

    let sel = Selector::parse(".definition").unwrap();
    for item in doc.select(&sel) {
        let defined_word = el_to_string(*find!(item, ".word")?);
        if defined_word != word { continue }

        let meaning = el_to_string(*find!(item, ".meaning")?);

        let example = el_to_string(*find!(item, ".example")?);

        definitions.push(Definition {
            part_of_speech: "slang".to_string(),
            meaning,
            examples: vec![example],
        });
    }

    if definitions.is_empty() {
        None
    } else {
        Some((definitions, SLANG_URL_BASE))
    }
}

pub(crate) async fn scrape_wiki(word: &str) -> Option<(Vec<Origin>, Vec<Definition>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/wiki/{}",
        PROTOCOL,
        WIKI_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    let eng = find!(doc, "#English")?;

    let mut origins = Vec::new();
    let mut working_origin_p = String::new();

    let mut definitions = Vec::new();

    let mut last_origin_title = String::new();
    let mut first_def_title = String::new();
    let mut last_title = String::new();

    let mut got_to_eng = false;
    for child in eng.parent().unwrap().parent().unwrap().children() {
        if !child.value().is_element() { continue }
        let el = child.value().as_element().unwrap();
        let el_ref = ElementRef::wrap(child).unwrap();

        if !got_to_eng {
            got_to_eng = find!(el_ref, "#English").is_some();
            continue;
        }

        match el.name() {
            "h3" | "h4" | "h5" => {
                let title = find!(el_ref, ".mw-headline")?;
                last_title = el_to_string(*title).to_lowercase();
            },
            "ol" => {
                for grandchild in child.children() {
                    if !grandchild.value().is_element() { continue }

                    let meaning = el_to_string_with_accepted(grandchild, &["span"], false);
                    if meaning.is_empty() { continue }

                    definitions.push(Definition {
                        part_of_speech: match last_title.as_str() {
                            "numeral" => "noun",
                            "letter" => "noun",
                            text => text,
                        }.to_owned(),
                        meaning,
                        examples: Vec::new(),
                    });

                    if first_def_title.is_empty() {
                        first_def_title = last_title.clone();
                    }
                }
            },
            "hr" => break,
            "p" if last_title.starts_with("etymology") => {
                if !working_origin_p.is_empty() 
                    && (origins.is_empty() || last_title != last_origin_title) 
                    && !first_def_title.is_empty() {
                    origins.push(Origin {
                        part_of_speech: first_def_title.clone(),
                        origin: working_origin_p.clone(),
                    });
                    working_origin_p.clear();
                    last_origin_title = last_title.clone();
                }

                // another paragraph to the working origin

                if !working_origin_p.is_empty() {
                    working_origin_p.push_str("<br>");
                }

                working_origin_p.push_str(&el_to_string_with_accepted(child, &["span"], false));

                first_def_title.clear();
            }
            _ => {},
        }
    }

    if !working_origin_p.is_empty() {
        origins.push(Origin {
            part_of_speech: first_def_title.clone(),
            origin: working_origin_p,
        });
    }

    Some((origins, definitions, WIKI_URL_BASE))
}

pub(crate) async fn scrape_etym(word: &str) -> Option<(Vec<Origin>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/search?q={}",
        PROTOCOL,
        ETYM_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    fn find_startswith<'a>(node: NodeRef<'a, Node>, pat: &str) -> Option<NodeRef<'a, Node>> {
        for child in node.children() {
            if !child.value().is_element() { continue }
            let el = child.value().as_element().unwrap();
            match el.attr("class") {
                Some(class) if class.starts_with(pat) => {
                    return Some(child);
                },
                _ => {
                    let rec = find_startswith(child, pat);
                    if rec.is_some() {
                        return rec;
                    }
                },
            }
        }
        None
    }

    let mut origins = Vec::new();

    let mut first = true;
    let word_name = find_startswith(doc.tree.root(), "word__name")?;

    let all_entries = word_name.parent()?.parent()?.parent()?;

    for word_entry in all_entries.children() {
        if !word_entry.value().is_element() {
            continue
        }
        let word_entry_el = word_entry.value().as_element()?;
        if !word_entry_el.attr("class").map(|class| class.starts_with("word")).unwrap_or(false) {
            continue
        }

        let word_name = {
            if first {
                first = false;
                word_name
            } else {
                find_startswith(word_entry, "word__name")?
            }
        };

        let word_name_text: Vec<&Text> = word_name
            .children()
            .filter_map(|child| match child.value() {
                Node::Text(text) => Some(text),
                _ => None,
            })
            .collect();
        
        let real_word_name = word_name_text
            .iter()
            .next()?
            .to_string();

        if real_word_name != word {
            break;
        }
        
        let part_of_speech = match word_name_text.iter().last()? as &str {
            "(n.)" => "noun",
            "(v.)" => "verb",
            "(adj.)" => "adjective",
            "(adv.)" => "adverb",
            "(interj.)" => "interjection",
            text if text.trim().is_empty() => "",
            text => {
                println!("NOT YET IMPLEMENTED: `{}` ({:?})", text, text.as_bytes());
                text
            }
        }.to_string();

        let mut origin = String::new();

        let sel = Selector::parse("p").unwrap();
        for p in find!(ElementRef::wrap(word_name.parent()?)?, "section")?.select(&sel) {
            if !origin.is_empty() {
                origin.push_str("<br>");
            }
            origin.push_str(&&el_to_string_with_accepted(*p, &["span", "a"], true));
        }

        origins.push(Origin {
            part_of_speech,
            origin,
        });
    }

    if origins.is_empty() {
        None
    } else {
        Some((origins, ETYM_URL_BASE))
    }
}

pub(crate) async fn scrape_stock(word: &str) -> Option<(Vec<String>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/search?k={}",
        PROTOCOL,
        STOCK_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    let mut imgs = Vec::new();

    let sel = Selector::parse(".search-result-cell").unwrap();
    for img_div in doc.select(&sel) {
        imgs.push(find!(img_div, "img")?.value().attr("src")?.to_string());

        if imgs.len() == 6 {
            break
        }
    }

    if imgs.is_empty() {
        None
    } else {
        Some((imgs, STOCK_URL_BASE))
    }
}

// utility functions for getting text from the dom

const PASSED_TAGS: &[&str] = &[
    "i",
    "b",
    "strong",
    "em",
    "mark",
    "cite",
    "dfn",
];

fn el_to_string(node: NodeRef<Node>) -> String {
    el_to_string_with_accepted(node, &[], false)
}

fn el_to_string_with_accepted(node: NodeRef<Node>, accepted: &[&str], add_i: bool) -> String {
    let mut res = String::new();
    let mut last_a = false;
    for item in node.children() {
        match item.value() {
            scraper::Node::Text(text) if !text.trim().is_empty() => {
                last_a = false;
                res.push_str(text);
            },
            scraper::Node::Element(el) if PASSED_TAGS.contains(&el.name()) => {
                last_a = false;
                res.push('<'); res.push_str(el.name()); res.push('>');
                res.push_str(&el_to_string_with_accepted(item, accepted, add_i));
                res.push_str("</"); res.push_str(el.name()); res.push('>');
            },
            scraper::Node::Element(el) if accepted.contains(&el.name()) => {
                last_a = false;
                if add_i {
                    res.push_str("<i>");
                }
                res.push_str(&el_to_string_with_accepted(item, accepted, add_i));
                if add_i {
                    res.push_str("</i>");
                }
            },
            scraper::Node::Element(el) if el.name() == "a" => {
                if last_a { res.push(' ') }
                last_a = true;
                res.push_str(&el_to_string_with_accepted(item, accepted, add_i));
            },
            _ => {},
        }
    }
    res.trim().to_string()
}