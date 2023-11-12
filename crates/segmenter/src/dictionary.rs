use std::collections::HashMap;

bitflags::bitflags! {
    ///
    #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PartOfSpeech: u32 {
        const NONE = 0;
        /// Noun or verb acting prenominally
        const ADJECTIVE_PRENOMINAL = 1 << 0;
        /// Adjective
        const ADJECTIVE = 1 << 1;
        /// Nouns which may take the genitive case particle 'no'
        const ADJECTIVE_NO = 1 << 2;
        /// Adverb (fukushi)
        const ADVERB = 1 << 3;
        /// Adverb taking the 'to' particle
        const ADVERB_TO = 1 << 4;
        /// Auxiliary
        const AUXILIARY = 1 << 5;
        /// Auxiliary adjective
        const AUXILIARY_ADJECTIVE = 1 << 6;
        /// Auxiliary verb
        const AUXILIARY_VERB = 1 << 7;
        /// Conjunction
        const CONJUNCTION = 1 << 8;
        /// Copula
        const COPULA = 1 << 9;
        /// Counter
        const COUNTER = 1 << 10;
        /// Expressions (phrases, clauses, etc.)
        const EXPRESSION = 1 << 11;
        /// Interjection (kandoushi)
        const INTERJECTION = 1 << 12;
        /// Noun (common) (futsuumeishi)
        const NOUN = 1 << 13;
        /// Adverbial noun (fukushitekimeishi)
        const NOUN_ADVERB = 1 << 14;
        /// Proper noun
        const NOUN_PROPER = 1 << 15;
        /// Noun, used as a prefix
        const NOUN_PREFIX = 1 << 16;
        /// Noun, used as a suffix
        const NOUN_SUFFIX = 1 << 17;
        /// Noun (temporal) (jisoumeishi)
        const NOUN_TEMPORAL = 1 << 18;
        /// Numeric
        const NUMERIC = 1 << 19;
        /// Pronoun
        const PRONOUN = 1 << 20;
        /// Prefix
        const PREFIX = 1 << 21;
        /// Particle
        const PARTICLE = 1 << 22;
        /// Suffix
        const SUFFIX = 1 << 23;
        /// Ichidan verb
        const ICHIDAN_VERB = 1 << 24;
        /// Godan verb
        const GODAN_VERB = 1 << 25;
        /// Intransitive verb
        const INTRANSITIVE_VERB = 1 << 26;
        /// Kuru verb - special class
        const KURU_VERB = 1 << 27;
        /// Suru verb
        const SURU_VERB = 1 << 28;
        /// Transitive verb
        const TRANSITIVE_VERB = 1 << 29;
        /// pre-noun adjectival
        const PRE_NOUN_ADJECTIVAL = 1 << 30;
    }
}

impl PartOfSpeech {
    pub fn is_particle(&self) -> bool {
        self.contains(Self::PARTICLE)
    }
}

bitflags::bitflags! {
    ///
    #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Tag: u16 {
        const NONE = 0;
        /// Word usually written using kana alone
        const USUALLY_KANA = 1 << 0;
        /// Abbreviation
        const ABBREVIATION = 1 << 1;
        /// Archaic
        const ARCHAIC = 1 << 2;
        /// Dated term
        const DATED_TERM = 1 << 3;
        /// Historical term
        const HISTORICAL_TERM = 1 << 4;
        /// Honorific or respectful (sonkeigo) language
        const SONKEIGO = 1 << 5;
        /// humble (kenjougo) language
        const KENJOUGO = 1 << 6;
        /// Polite (teineigo) language
        const TEINEIGO = 1 << 7;
        /// Idiomatic expression
        const IDIOMATIC_EXPRESSION = 1 << 8;
        /// Obsolete term
        const OBSOLETE_TERM = 1 << 9;
        /// Rare term
        const RARE = 1 << 10;
        /// yojijukugo
        const YOJIJUKUGO = 1 << 11;
    }
}

///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum InflectionType {
    DictionaryForm,
    Negative,
    Te,
    NegativeTe,
    Past,
    NegativePast,
    Potential,
    NegativePotential,
    Imperative,
    ImperativeNegative,
    Causative,
    CausativePassive,
    NegativeCausativePassive,
    NegativeCausative,
    Passive,
    NegativePassive,
}

///
#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct DictionaryEntry {
    pub term_id: u32,
    pub pos: PartOfSpeech,
    pub tag: Tag,
}

///
#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct TermEntry {
    pub entry_index: u32,
    pub inflection_type: InflectionType,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
    pub kanji: HashMap<String, Vec<TermEntry>>,
    pub kana: HashMap<String, Vec<TermEntry>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            kanji: HashMap::new(),
            kana: HashMap::new(),
        }
    }
}
