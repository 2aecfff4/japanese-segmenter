use quick_xml::de::{Deserializer, EntityResolver};
use quick_xml::events::BytesText;
use regex::bytes::Regex;
use segmenter::dictionary::{Dictionary, DictionaryEntry, InflectionType, TermEntry};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::string::FromUtf8Error;

struct DocTypeEntityResolver {
    re: Regex,
    map: HashMap<String, String>,
}

impl DocTypeEntityResolver {
    fn new() -> Self {
        Self {
            // We do not focus on true parsing in this example
            // You should use special libraries to parse DTD
            re: Regex::new(r#"<!ENTITY\s+([^ \t\r\n]+)\s+"([^"]*)"\s*>"#).unwrap(),
            map: HashMap::new(),
        }
    }
}

impl EntityResolver for DocTypeEntityResolver {
    type Error = FromUtf8Error;

    fn capture(&mut self, doctype: BytesText) -> Result<(), Self::Error> {
        for cap in self.re.captures_iter(&doctype) {
            self.map.insert(
                String::from_utf8(cap[1].to_vec())?,
                String::from_utf8(cap[1].to_vec())?,
            );
        }
        Ok(())
    }

    fn resolve(&self, entity: &str) -> Option<&str> {
        self.map.get(entity).map(|s| s.as_str())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JMdict {
    #[serde(rename = "entry")]
    entries: Vec<Entry>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "entry")]
struct Entry {
    ent_seq: i64,
    #[serde(rename = "k_ele")]
    kanji_elements: Option<Vec<KanjiElement>>,
    #[serde(rename = "r_ele")]
    reading_elements: Option<Vec<ReadingElement>>,
    #[serde(rename = "sense")]
    senses: Option<Vec<Sense>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "ent_seq")]
struct EntSeq {
    #[serde(rename = "$value")]
    pub body: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "k_ele")]
struct KanjiElement {
    keb: Option<String>,
    ke_pri: Option<Vec<String>>,
    ke_inf: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "r_ele")]
struct ReadingElement {
    reb: Option<String>,
    keb: Option<String>,
    re_inf: Option<Vec<String>>,
    re_pri: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "pos")]
struct PoS {
    #[serde(rename = "$value")]
    pub body: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "sense")]
struct Sense {
    pos: Vec<String>,
    stagk: Option<Vec<String>>,
    stagr: Option<Vec<String>>,
    xref: Option<Vec<String>>,
    ant: Option<Vec<String>>,
    field: Option<Vec<String>>,
    misc: Option<Vec<String>>,
    s_inf: Option<String>,
    dial: Option<Vec<String>>,
    gloss: Vec<String>,
}

// http://ftp.usf.edu/pub/ftp.monash.edu.au/pub/nihongo/00INDEX.html

fn main() {
    let f = fs::File::open("JMdict_e/JMdict_e.xml").unwrap();
    let reader = std::io::BufReader::with_capacity(1024 * 1024 * 128, f);
    let mut de = Deserializer::with_resolver(reader, DocTypeEntityResolver::new());
    let dict = JMdict::deserialize(&mut de).unwrap();

    let mut dictionary = Dictionary::new();

    for entry in dict.entries.iter() {
        let mut kanji_words = Vec::new();
        let mut kana_words = Vec::new();
        let mut part_of_speeches = HashSet::new();
        let mut tags = HashSet::new();

        if let Some(ref kanji_elements) = entry.kanji_elements {
            for kanji_element in kanji_elements.iter() {
                if let Some(ref keb) = kanji_element.keb {
                    kanji_words.push(keb.clone());
                }
            }
        }

        if let Some(ref reading_elements) = entry.reading_elements {
            for reading_element in reading_elements.iter() {
                if let Some(ref reb) = reading_element.reb {
                    kana_words.push(reb.clone());
                }
            }
        }
        // <!ENTITY rK "rarely-used kanji form">
        if let Some(ref senses) = entry.senses {
            for sense in senses.iter() {
                for pos in sense.pos.iter() {
                    part_of_speeches.insert(pos.clone());
                }

                if let Some(ref miscs) = sense.misc {
                    for misc in miscs.iter() {
                        tags.insert(misc.clone());
                    }
                }
            }
        }

        let is_godan = part_of_speeches.iter().any(|pos| pos.starts_with("v5"));
        let is_ichidan = part_of_speeches.iter().any(|pos| pos.starts_with("v1"));
        let dictionary_entry_index = dictionary.entries.len() as u32;

        dictionary.entries.push(DictionaryEntry {
            term_id: 0,
            pos: segmenter::dictionary::PartOfSpeech::empty(),
            tag: segmenter::dictionary::Tag::empty(),
        });

        use wana_kana::ConvertJapanese;

        for kanji in kanji_words.iter() {
            dictionary
                .kanji
                .entry(kanji.clone())
                .and_modify(|v| {
                    v.push(TermEntry {
                        entry_index: dictionary_entry_index,
                        inflection_type: InflectionType::DictionaryForm,
                    })
                })
                .or_insert_with(|| {
                    vec![segmenter::dictionary::TermEntry {
                        entry_index: dictionary_entry_index,
                        inflection_type:
                            segmenter::dictionary::InflectionType::DictionaryForm,
                    }]
                });

            if is_godan || is_ichidan {
                for kana in kana_words.iter() {
                    let kana = kana.to_hiragana();
                    if is_godan {
                        add_conjugations(
                            &mut dictionary,
                            jp_inflections::VerbType::Godan,
                            &kana,
                            Some(kanji),
                            dictionary_entry_index,
                        );
                    } else if is_ichidan {
                        add_conjugations(
                            &mut dictionary,
                            jp_inflections::VerbType::Ichidan,
                            &kana,
                            Some(kanji),
                            dictionary_entry_index,
                        );
                    }
                }
            }
        }

        for kana in kana_words.iter() {
            dictionary
                .kana
                .entry(kana.clone())
                .and_modify(|v| {
                    v.push(TermEntry {
                        entry_index: dictionary_entry_index,
                        inflection_type: InflectionType::DictionaryForm,
                    })
                })
                .or_insert(vec![segmenter::dictionary::TermEntry {
                    entry_index: dictionary_entry_index,
                    inflection_type: InflectionType::DictionaryForm,
                }]);

            let kana = kana.to_hiragana();
            #[allow(clippy::collapsible_else_if)]
            if is_godan || is_ichidan {
                if !kanji_words.is_empty() {
                    for kanji in kanji_words.iter() {
                        if is_godan {
                            add_conjugations(
                                &mut dictionary,
                                jp_inflections::VerbType::Godan,
                                &kana,
                                Some(kanji),
                                dictionary_entry_index,
                            );
                        } else if is_ichidan {
                            add_conjugations(
                                &mut dictionary,
                                jp_inflections::VerbType::Ichidan,
                                &kana,
                                Some(kanji),
                                dictionary_entry_index,
                            );
                        }
                    }
                } else {
                    if is_godan {
                        add_conjugations(
                            &mut dictionary,
                            jp_inflections::VerbType::Godan,
                            &kana,
                            None,
                            dictionary_entry_index,
                        );
                    } else if is_ichidan {
                        add_conjugations(
                            &mut dictionary,
                            jp_inflections::VerbType::Ichidan,
                            &kana,
                            None,
                            dictionary_entry_index,
                        );
                    }
                }
            }
        }
    }

    let kanji_len = dictionary.kanji.len();
    let kana_len = dictionary.kana.len();
    let entries_len = dictionary.entries.len();
    println!("kanji len: {kanji_len}");
    println!("kana len: {kana_len}");
    println!("entries len: {entries_len}");
    let encoded: Vec<u8> = bincode::serialize(&dictionary).unwrap();
    std::fs::write("dictionary_test_sg_jp.bin", encoded).unwrap();
}

fn add_conjugations(
    dictionary: &mut Dictionary,
    verb_type: jp_inflections::VerbType,
    kana: &str,
    kanji: Option<&str>,
    entry_index: u32,
) {
    use jp_inflections::*;
    let verb = Word::new(kana, kanji).into_verb(verb_type).unwrap();

    let negative = verb.negative(WordForm::Short).unwrap();
    let negative_long = verb.negative(WordForm::Long).unwrap();

    let te = verb.te_form().unwrap();

    let negative_te = verb.negative_te_form().unwrap();

    let past = verb.past(WordForm::Short).unwrap();
    let past_long = verb.past(WordForm::Long).unwrap();

    let negative_past = verb.negative_past(WordForm::Short).unwrap();
    let negative_past_long = verb.negative_past(WordForm::Long).unwrap();

    let potential = verb.potential(WordForm::Short).unwrap();
    let potential_long = verb.potential(WordForm::Long).unwrap();

    let negative_potential = verb.negative_potential(WordForm::Short).unwrap();
    let negative_potential_long = verb.negative_potential(WordForm::Long).unwrap();

    let imperative = verb.imperative().unwrap();

    let imperative_negative = verb.imperative_negative().unwrap();

    let causative = verb.causative().unwrap();

    let causative_passive = verb.causative_passive().unwrap();

    let negative_causative_passive = verb.negative_causative_passive().unwrap();

    let negative_causative = verb.negative_causative().unwrap();

    let passive = verb.passive().unwrap();

    let negative_passive = verb.negative_passive().unwrap();

    let words = [
        negative,
        negative_long,
        te,
        negative_te,
        past,
        past_long,
        negative_past,
        negative_past_long,
        potential,
        potential_long,
        negative_potential,
        negative_potential_long,
        imperative,
        imperative_negative,
        causative,
        causative_passive,
        negative_causative_passive,
        negative_causative,
        passive,
        negative_passive,
    ];

    for word in words {
        if let Some(kanji) = word.kanji {
            if !dictionary.kanji.contains_key(&kanji) {
                dictionary
                    .kanji
                    .entry(kanji.clone())
                    .and_modify(|v| {
                        v.push(TermEntry {
                            entry_index,
                            inflection_type: InflectionType::DictionaryForm,
                        })
                    })
                    .or_insert_with(|| {
                        vec![segmenter::dictionary::TermEntry {
                            entry_index,
                            inflection_type:
                                segmenter::dictionary::InflectionType::DictionaryForm,
                        }]
                    });
            }
        }

        let kana = word.kana;
        if !dictionary.kana.contains_key(&kana) {
            dictionary
                .kana
                .entry(kana.clone())
                .and_modify(|v| {
                    v.push(TermEntry {
                        entry_index,
                        inflection_type: InflectionType::DictionaryForm,
                    })
                })
                .or_insert_with(|| {
                    vec![segmenter::dictionary::TermEntry {
                        entry_index,
                        inflection_type:
                            segmenter::dictionary::InflectionType::DictionaryForm,
                    }]
                });
        }
    }
}
