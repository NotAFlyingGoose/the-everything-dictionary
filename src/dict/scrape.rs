
use ego_tree::NodeRef;
use regex::Regex;
use scraper::{Html, Selector, ElementRef, Node, node::Text};

use super::{Origin, Definition, restrictor::RESTRICTOR};

macro_rules! find {
    ($element: expr, $selector: literal) => {
        {
            let sel = Selector::parse($selector).unwrap();
            $element.select(&sel).nth(0)
        }
    };
}

macro_rules! loop_find_all {
    ($element: expr, $selector: literal, $name: ident, $for_each: expr) => {
        {
            let sel = Selector::parse($selector).unwrap();
            for $name in $element.select(&sel) {
                $for_each
            }
        }
    };
}

const PROTOCOL: &str = "https";
const VOCAB_URL_BASE: &str = "www.vocabulary.com";
const MACMILLAN_URL_BASE: &str = "www.macmillandictionary.com";
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

    let short_overview = Some(el_to_string_with(*find!(word_area, ".short")?, &[], true, &[INCLUDED_TAGS, &["i"]].concat()))
        .filter(|s| !s.is_empty());
    let long_overview = Some(el_to_string_with(*find!(word_area, ".short")?, &[], true, &[INCLUDED_TAGS, &["i"]].concat()))
        .filter(|s| !s.is_empty());

    let ol = find!(find!(doc, ".word-definitions")?, "ol")?;

    let mut definitions = Vec::new();

    loop_find_all!(ol, "li", item, {
        let def_area = find!(item, ".definition")?;

        let part_of_speech = el_to_string(*find!(def_area, ".pos-icon")?);

        let meaning = el_to_string(*def_area);

        let mut examples = Vec::new();

        loop_find_all!(item, ".example", example, {
            examples.push(el_to_string(*example).replace("\n", ""));
        });

        definitions.push(Definition {
            part_of_speech,
            meaning,
            examples,
        });
    });

    Some((short_overview, long_overview, definitions, VOCAB_URL_BASE))
}

pub(crate) async fn scrape_macmillan(word: &str) -> Option<(Vec<Vec<Definition>>, &str)> {
    let body = reqwest::get(&format!(
        "{}://{}/us/dictionary/american/{}",
        PROTOCOL,
        MACMILLAN_URL_BASE,
        word,
    ))
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let doc = Html::parse_document(&body);

    let word_area = find!(doc, ".left-content")?;

    let real_word = el_to_string(word_area
        .first_child()?
        .first_child()?
        .first_child()?
        .first_child()?);
    if real_word != word {
        println!("the real word was {}", real_word);
    }

    let definition_area = word_area.children()
        .find(|child| child.value().is_element() 
            && child.value().as_element().unwrap().attrs.is_empty())?
        .children()
        .find(|child| child.value().is_element() 
            && child.value().as_element().unwrap().attrs.is_empty())?;
    
    let ol = find!(ElementRef::wrap(definition_area).unwrap(), "ol")?;

    let mut definitions = Vec::new();

    loop_find_all!(ol, "li", item, {
        let sense_body = if let Some(el) = find!(item, ".SENSE-BODY") {
            el
        } else {
            continue;
        };

        let mut sense = Vec::new();

        loop_find_all!(sense_body, ".dflex", body, {
            let meaning = if let Some(el) = find!(body, ".DEFINITION") {
                el
            } else {
                continue;
            };

            let meaning = el_to_string_with(*meaning, &["a", "span"], false, INCLUDED_TAGS);

            let mut examples = Vec::new();
            if let Some(examples_el) = find!(body, ".EXAMPLES") {
                for example in examples_el.children() {
                    if !example.value().is_element() || example.value().as_element().unwrap().name() != "p" {
                        continue;
                    }
                    examples.push(el_to_string_with(example, &["a", "span"], false, INCLUDED_TAGS).replace("\n", ""));
                }
            }

            sense.push(Definition {
                part_of_speech: "noun".to_string(),
                meaning,
                examples,
            });
        });

        definitions.push(sense);
    });

    Some((definitions, MACMILLAN_URL_BASE))
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

                    let meaning = el_to_string_with(grandchild, &["a"], false, &[INCLUDED_TAGS, &["span"]].concat());
                    if meaning.is_empty() { continue }

                    let mut examples = Vec::new();

                    let examples_list = find!(ElementRef::wrap(grandchild).unwrap(), "dl");
                    if examples_list.is_some() {
                        for el in examples_list.unwrap().children() {
                            if !el.value().is_element() || el.value().as_element().unwrap().name() != "dd" { continue }
                            examples.push(el_to_string_with(el, &["span"], true, INCLUDED_TAGS))
                        }
                    }

                    definitions.push(Definition {
                        part_of_speech: match last_title.as_str() {
                            "numeral" | "number" | "letter" => "noun",
                            text => text,
                        }.to_owned(),
                        meaning,
                        examples,
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

                working_origin_p.push_str(&el_to_string_with(child, &["span"], false, INCLUDED_TAGS));

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
        
        let re = Regex::new(r"[0-9]+").unwrap();
        let part_of_speech = match re.replace_all(word_name_text.iter().last()?, "")
            .to_string().as_str() {
            "(n.)" => "noun",
            "(v.)" => "verb",
            "(adj.)" => "adjective",
            "(adv.)" => "adverb",
            "(interj.)" => "interjection",
            "(prep.)" => "preposition",
            "(pron.)" => "pronoun",
            text if text.trim().is_empty() => "",
            text => {
                println!("NOT YET IMPLEMENTED: `{}` ({:?})", text, text.as_bytes());
                text
            }
        }.to_string();

        let mut origin = String::new();

        loop_find_all!(find!(ElementRef::wrap(word_name.parent()?)?, "section")?, "p", p, {
            if !origin.is_empty() {
                origin.push_str("<br>");
            }
            origin.push_str(&&el_to_string_with(*p, &["span", "a"], true, INCLUDED_TAGS));
        });

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
    if RESTRICTOR.is_restricted(word.to_lowercase().as_str()) { return None }

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

    loop_find_all!(doc, ".search-result-cell", img_div, {
        let img_el = find!(img_div, "img")?.value();

        if RESTRICTOR.is_restricted(img_el.attr("alt")?.to_lowercase().as_str()) { continue }

        imgs.push(img_el.attr("src")?.to_string());

        if imgs.len() == 6 {
            break
        }
    });

    if imgs.is_empty() {
        None
    } else {
        Some((imgs, STOCK_URL_BASE))
    }
}

// utility functions for getting text from the dom

const INCLUDED_TAGS: &[&str] = &[
    "b",
    "strong",
    "em",
    "mark",
    "cite",
    "dfn",
];

fn el_to_string(node: NodeRef<Node>) -> String {
    el_to_string_with(node, &[], true, INCLUDED_TAGS)
}

fn el_to_string_with(node: NodeRef<Node>, pass_through: &[&str], pass_replace: bool, included: &[&str]) -> String {
    let mut res = String::new();
    for item in node.children() {
        match item.value() {
            scraper::Node::Text(text) if !text.trim().is_empty() => {
                res.push_str(text);
            }
            scraper::Node::Element(el) if included.contains(&el.name()) => {
                res.push('<'); res.push_str(el.name()); res.push('>');
                res.push_str(&el_to_string_with(item, pass_through, pass_replace, included));
                res.push_str("</"); res.push_str(el.name()); res.push('>');
                res.push(' ');
            }
            scraper::Node::Element(el) if pass_through.contains(&el.name()) => {
                if pass_replace {
                    res.push_str("<i>");
                }
                res.push_str(&el_to_string_with(item, pass_through, pass_replace, included));
                if pass_replace {
                    res.push_str("</i>");
                }
                res.push(' ');
            }
            _ => {},
        }
    }
    res.trim()
        .replace("  ", " ")
        .replace(" ,", ",")
        .replace(" !", "!")
        .replace(" ?", "?")
        .replace(" .", ".")
        .replace(" )", ")")
}