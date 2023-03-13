use crate::{
    formatter::capitalise,
    name_matcher::{move_names::MOVE_NAMES, pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES},
};

use ngrammatic::{Corpus, CorpusBuilder, Pad};
use once_cell::sync::Lazy;

static MIN_SIMILARITY: f32 = 0.4;
static CERTAIN_SIMILARITY: f32 = 0.7;

pub enum MatcherType {
    Pokemon,
    Move,
    Type,
}

pub enum Certainty {
    Positive,
    Neutral,
}

pub struct SuccessfulMatch {
    pub original_name: String,
    pub suggested_name: String,
    pub keyword: String,
    pub certainty: Certainty,
}

impl SuccessfulMatch {
    pub fn new(original_name: String, keyword: String, suggestion: Suggestion) -> Self {
        let certainty = if suggestion.similarity >= CERTAIN_SIMILARITY {
            Certainty::Positive
        } else {
            Certainty::Neutral
        };

        let suggested_name = suggestion.name;

        Self {
            original_name,
            suggested_name,
            keyword,
            certainty,
        }
    }
}

pub struct NoMatch {
    pub keyword: String,
}

impl NoMatch {
    pub fn new(keyword: String) -> Self {
        Self { keyword }
    }
}

pub struct Suggestion {
    name: String,
    similarity: f32,
}

impl Suggestion {
    pub fn new(name: String, similarity: f32) -> Self {
        Self { name, similarity }
    }
}

struct NameMatcher {
    names: Vec<String>,
}

impl NameMatcher {
    pub fn new(names: Vec<String>) -> Self {
        NameMatcher { names }
    }

    pub fn find_match(&self, name: &str) -> Option<Suggestion> {
        let corpus = self.build_corpus();
        let search_results = corpus.search(name, 0.25);
        let search_result = search_results.first().map(|r| r.to_owned())?;

        if search_result.similarity > MIN_SIMILARITY {
            let suggested_name = Suggestion::new(search_result.text, search_result.similarity);

            Some(suggested_name)
        } else {
            None
        }
    }

    fn build_corpus(&self) -> Corpus {
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

        self.names.iter().for_each(|name| corpus.add_text(name));

        corpus
    }
}

pub fn match_name(name: &str, matcher_type: MatcherType) -> Result<SuccessfulMatch, NoMatch> {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);

    match name_matcher.find_match(name) {
        Some(suggestion) => {
            let successful_match = SuccessfulMatch::new(name.to_owned(), keyword, suggestion);

            Ok(successful_match)
        }

        None => Err(NoMatch::new(keyword)),
    }
}

pub fn build_suggested_name(successful_match: &SuccessfulMatch) -> String {
    let keyword = &successful_match.keyword;
    let original_name = &successful_match.original_name;
    let similar_name = &successful_match.suggested_name;

    format!("Unknown {keyword} \"{original_name}\"\nDid you mean \"{similar_name}\"?")
}

pub fn build_unknown_name(name: &str, keyword: &str) -> String {
    format!("{} \"{}\" doesn't exist", capitalise(keyword), name)
}

pub fn try_suggest_name(name: &str, matcher_type: MatcherType) -> String {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);

    match name_matcher.find_match(name) {
        Some(suggestion) => {
            let similar_name = suggestion.name;

            format!("Unknown {keyword} \"{name}\"\nDid you mean \"{similar_name}\"?")
        }

        None => format!("{} \"{}\" doesn't exist", capitalise(&keyword), name),
    }
}

fn matcher_and_keyword(matcher_type: MatcherType) -> (NameMatcher, String) {
    let (names, keyword) = match matcher_type {
        MatcherType::Move => (&MOVE_NAMES, "move"),
        MatcherType::Pokemon => (&POKEMON_NAMES, "pokemon"),
        MatcherType::Type => (&TYPE_NAMES, "type"),
    };

    (
        NameMatcher::new(Lazy::force(names).to_owned()),
        String::from(keyword),
    )
}
