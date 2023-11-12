use crate::{
    dictionary::{Dictionary, DictionaryEntry, PartOfSpeech, Tag},
    lattice::{Lattice, LatticeNode},
};
use regex::RegexSet;
use std::sync::Arc;

///
#[derive(Clone, Copy, PartialEq, Eq)]
enum WordCategory {
    Katakana,
    Kana,
    Word,
    NonWord,
}

///
const REGEX_CATEGORIES: [(&str, WordCategory); 4] = [
    (r"^\p{Katakana}+$", WordCategory::Katakana),
    // (r"^\p{Hiragana}+$", WordCategory::Hiragana),
    // (r"^\p{Han}+$", WordCategory::Kanji),
    (r"^(\p{Katakana}|\p{Hiragana})+$", WordCategory::Kana),
    (r"^(\p{Han}|\p{Hiragana})+$", WordCategory::Word),
    (r"^[^々一-龯ァ-ヺヽヾぁ-ゔゝゞー]+$", WordCategory::NonWord),
];

///
fn categorize_word(word: &str) -> WordCategory {
    lazy_static::lazy_static! {
        static ref REGEX: RegexSet =
            RegexSet::new(REGEX_CATEGORIES.iter().map(|(r, _)| r)).unwrap();
    }

    let matches = REGEX.matches(word);
    for (i, (_, category)) in REGEX_CATEGORIES.iter().enumerate() {
        if matches.matched(i) {
            return *category;
        }
    }

    WordCategory::NonWord
}

///
#[derive(Debug, Copy, Clone)]
pub struct Token<'a> {
    pub term_id: Option<u32>,
    pub token: &'a str,
}

///
pub struct Tokenizer {
    dictionary: Arc<Dictionary>,
}

///
impl Tokenizer {
    ///
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    fn inner_loop<'a, Fn>(text: &'a str, start: usize, length: usize, mut inner: Fn)
    where
        Fn: FnMut(&'a str, usize, usize),
    {
        let start_pos = text.char_indices().nth(start).map(|(n, _)| n).unwrap();
        for end in (start + 1)..length {
            let end_pos = text.char_indices().nth(end).map(|(n, _)| n).unwrap();
            let substring = &text[start_pos..end_pos];

            inner(substring, start, end);
        }
    }

    fn inner_loop_unknown_term<'a, Fn>(
        force: bool,
        text: &'a str,
        start: usize,
        length: usize,
        mut inner: Fn,
    ) where
        Fn: FnMut(&'a str, usize, usize),
    {
        if (start + 1) >= length {
            return;
        }

        struct Category {
            invoke: bool,
            group: bool,
            func: fn(char) -> bool,
        }

        const CATEGORIES: &[Category] = &[
            // Space
            Category {
                invoke: false,
                group: true,
                func: |c| matches!(c as u32, 0x0020 | 0x00D0 | 0x0009 | 0x000B | 0x000A),
            },
            // Kanji
            Category {
                invoke: false,
                group: false,
                func: |c| {
                    matches!(c as u32,
                        0x2E80..=0x2EF3
                        | 0x2F00..=0x2FD5
                        | 0x3005
                        | 0x3007
                        | 0x3400..=0x4DB5
                        | 0x4E00..=0x9FA5
                        | 0xF900..=0xFA2D
                        | 0xFA30..=0xFA6A
                    )
                },
            },
            // Symbol
            Category {
                invoke: true,
                group: true,
                func: |c| {
                    matches!(c as u32,
                        0x0021..=0x002F
                        | 0x003A..=0x0040
                        | 0x005B..=0x0060
                        | 0x007B..=0x007E
                        | 0x00A1..=0x00BF
                        | 0xFF01..=0xFF0F
                        | 0xFF1A..=0xFF1F
                        | 0xFF3B..=0xFF40
                        | 0xFF5B..=0xFF65
                        | 0xFFE0..=0xFFEF
                        | 0x2000..=0x206F
                        | 0x20A0..=0x20CF
                        | 0x20D0..=0x20FF
                        | 0x2100..=0x214F
                        | 0x2190..=0x21FF
                        | 0x2200..=0x22FF
                        | 0x2300..=0x23FF
                        | 0x2460..=0x24FF
                        | 0x2501..=0x257F
                        | 0x2580..=0x259F
                        | 0x25A0..=0x25FF
                        | 0x2600..=0x26FE
                        | 0x2700..=0x27BF
                        | 0x27F0..=0x27FF
                        | 0x27C0..=0x27EF
                        | 0x2800..=0x28FF
                        | 0x2900..=0x297F
                        | 0x2B00..=0x2BFF
                        | 0x2A00..=0x2AFF
                        | 0x3300..=0x33FF
                        | 0x3200..=0x32FE
                        | 0x3000..=0x303F
                        | 0xFE30..=0xFE4F
                        | 0xFE50..=0xFE6B
                    )
                },
            },
            // Numeric
            Category {
                invoke: true,
                group: true,
                func: |c| {
                    matches!(c as u32,
                        0x0030..=0x0039
                        | 0xFF10..=0xFF19
                        | 0x2070..=0x209F
                        | 0x2150..=0x218F
                    )
                },
            },
            // Alpha
            Category {
                invoke: false,
                group: true,
                func: |c| {
                    matches!(c as u32,
                        0x0041..=0x005A
                        | 0x0061..=0x007A
                        | 0x00C0..=0x00FF
                        | 0x0100..=0x017F
                        | 0x0180..=0x0236
                        | 0x1E00..=0x1EF9
                        | 0xFF21..=0xFF3A
                        | 0xFF41..=0xFF5A
                    )
                },
            },
            // Hiragana
            Category {
                invoke: false,
                group: true,
                func: |c| matches!(c as u32, 0x3041..=0x309F),
            },
            // Katakana
            Category {
                invoke: true,
                group: true,
                func: |c| {
                    matches!(c as u32,
                        0x30A1..=0x30FF
                        | 0x31F0..=0x31FF
                        | 0xFF66..=0xFF9D
                        | 0xFF9E..=0xFF9F
                    )
                },
            },
            // Greek
            Category {
                invoke: true,
                group: true,
                func: |c| matches!(c as u32, 0x0374..=0x03FB),
            },
            // Cyrillic
            Category {
                invoke: true,
                group: true,
                func: |c| matches!(c as u32, 0x0400..=0x04F9 | 0x0500..=0x050F),
            },
        ];

        for category in CATEGORIES.iter() {
            if !force && !category.invoke {
                continue;
            }

            let start_pos = text.char_indices().nth(start).map(|(n, _)| n).unwrap();
            if category.group {
                let (count, end_pos, end) = {
                    let mut count: usize = 0;
                    let mut end_pos: usize = 0;
                    let mut end = start;

                    let iter = ((start + 1)..length).zip(text.char_indices().skip(start));
                    for (end_idx, (char_idx, c)) in iter {
                        if (category.func)(c) {
                            count += 1;
                            end_pos = char_idx;
                            end = end_idx;
                        } else {
                            break;
                        }
                    }

                    (count, end_pos, end)
                };

                if count == 0 {
                    continue;
                }

                let substring = &text[start_pos..end_pos];
                inner(substring, start, end);
            } else if let Some((end_pos, c)) = text.char_indices().nth(start) {
                if !(category.func)(c) {
                    continue;
                }
                let end = start + 1;
                let substring = &text[start_pos..end_pos];
                inner(substring, start, end);
            }
        }
    }

    ///
    pub fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let length = text.chars().count();
        let node_count = ((length * (length + 1)) as f32 / 2.0).ceil() as usize;
        let mut lattice = Lattice::new(node_count, length);

        for start in 0..length {
            let mut found_any_term = false;
            Self::inner_loop(text, start, length, |substring, start, end| {
                let category = categorize_word(substring);
                let term_entry = match category {
                    WordCategory::Kana | WordCategory::Katakana => {
                        self.dictionary.kana.get(substring)
                    }
                    WordCategory::Word => self.dictionary.kanji.get(substring),
                    WordCategory::NonWord => None,
                };

                if let Some(term_entries) = term_entry {
                    for term_entry in term_entries.iter() {
                        let dictionary_entry =
                            &self.dictionary.entries[term_entry.entry_index as usize];

                        let term_id = dictionary_entry.term_id;
                        let score = self.get_score(
                            end - start,
                            category,
                            &Some(dictionary_entry),
                        );
                        lattice.add_node(LatticeNode {
                            term_id: Some(term_id),
                            start,
                            end,
                            score,
                        });
                        found_any_term |= true;
                    }
                }
            });

            Self::inner_loop_unknown_term(
                !found_any_term,
                text,
                start,
                length,
                |substring, start, end| {
                    let category = categorize_word(substring);
                    let score = self.get_score(end - start, category, &None);
                    lattice.add_node(LatticeNode {
                        term_id: None,
                        start,
                        end,
                        score,
                    });
                },
            );

            // for end in (start + 1)..length {
            //     let start_pos = text.char_indices().nth(start).map(|(n, _)| n).unwrap();
            //     let end_pos = text.char_indices().nth(end).map(|(n, _)| n).unwrap();
            //     let substring = &text[start_pos..end_pos];

            //     let category = categorize_word(substring);

            //     let term_entry = match category {
            //         WordCategory::Kana | WordCategory::Katakana => {
            //             self.dictionary.kana.get(substring)
            //         }
            //         WordCategory::Word => self.dictionary.kanji.get(substring),
            //         WordCategory::NonWord => None,
            //     };

            //     let dictionary_entry = term_entry.map(|term_entry| {
            //         &self.dictionary.entries[term_entry.entry_index as usize]
            //     });

            //     let term_id =
            //         dictionary_entry.map(|dictionary_entry| dictionary_entry.term_id);
            //     let score = self.get_score(end - start, category, &dictionary_entry);
            //     lattice.add_node(LatticeNode {
            //         term_id,
            //         start,
            //         end,
            //         score,
            //     });
            // }
        }

        // #TODO: Avoid unnecessary memory allocation when creating a path?
        lattice
            .find_path()
            .iter()
            .map(|node| {
                let start_pos =
                    text.char_indices().nth(node.start).map(|(n, _)| n).unwrap();
                let end_pos = text.char_indices().nth(node.end).map(|(n, _)| n).unwrap();

                Token {
                    term_id: node.term_id,
                    token: &text[start_pos..end_pos],
                }
            })
            .collect()
    }

    ///
    fn get_score(
        &self,
        text_len: usize,
        category: WordCategory,
        dictionary_entry: &Option<&DictionaryEntry>,
    ) -> f32 {
        let mut score = 1.0f32;
        // If it's written only in katakana, then most likely it is a word.
        if category == WordCategory::Katakana {
            score += 15.0;
        }

        if let Some(dictionary_entry) = dictionary_entry {
            // Boost terms that exist in the dictionary
            score += 5.0;

            if dictionary_entry.pos.is_particle() {
                score += 4.0;
            }

            if dictionary_entry.pos.contains(PartOfSpeech::EXPRESSION) {
                score += 2.0;
            }

            if dictionary_entry.tag.contains(Tag::IDIOMATIC_EXPRESSION) {
                score += 8.0;
            }

            // if dictionary_entry
            //     .pos
            //     .intersects(PartOfSpeech::NOUN | PartOfSpeech::ADJECTIVE)
            // {
            //     score += 4.0;
            // }

            // if dictionary_entry.pos.contains(PartOfSpeech::INTERJECTION) {
            //     score += 4.0;
            // }

            // if dictionary_entry.pos.contains(PartOfSpeech::SUFFIX) {
            //     score += 4.0;
            // }

            // if dictionary_entry.pos.contains(PartOfSpeech::INTERJECTION) {
            //     score += 4.0;
            // }

            // if dictionary_entry
            //     .pos
            //     .contains(PartOfSpeech::PRE_NOUN_ADJECTIVAL)
            // {
            //     score += 4.0;
            // }

            // if dictionary_entry.pos.contains(PartOfSpeech::TRANSITIVE_VERB) {
            //     score += 4.0;
            // }
        }

        let power = if category == WordCategory::Word {
            3.0
        } else {
            2.0
        };

        score *= (text_len as f32).powf(power);

        score
    }
}
