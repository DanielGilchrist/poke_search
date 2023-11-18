use crate::{
    formatter::capitalise,
    name_matcher::{move_names::MOVE_NAMES, pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES},
};

use ngrammatic::{Corpus, CorpusBuilder, Pad};
use once_cell::sync::Lazy;

static MIN_SIMILARITY: f32 = 0.5;

pub enum MatcherType {
    Pokemon,
    Move,
    Type,
}

pub struct SuccessfulMatch {
    pub original_name: String,
    pub suggested_name: String,
    pub keyword: String,
}

impl SuccessfulMatch {
    pub fn new(original_name: String, keyword: String, suggestion: Suggestion) -> Self {
        Self {
            original_name,
            suggested_name: suggestion.0,
            keyword,
        }
    }
}

pub struct NoMatch(pub String);

impl NoMatch {
    pub fn new(message: String) -> Self {
        Self(message)
    }
}

pub struct Suggestion(String);

impl Suggestion {
    pub fn new(name: String) -> Self {
        Self(name)
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
            Some(Suggestion::new(search_result.text))
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

    if name_is_already_valid(&name_matcher.names, &name.to_owned()) {
        let suggestion = Suggestion::new(name.to_owned());
        let successful_match = SuccessfulMatch::new(name.to_owned(), keyword, suggestion);

        return Ok(successful_match);
    }

    match name_matcher.find_match(name) {
        Some(suggestion) => {
            let successful_match = SuccessfulMatch::new(name.to_owned(), keyword, suggestion);
            Ok(successful_match)
        }

        None => Err(NoMatch::new(build_unknown_name(name, &keyword))),
    }
}

pub fn build_unknown_name(name: &str, keyword: &str) -> String {
    format!("{} \"{}\" doesn't exist", capitalise(keyword), name)
}

fn name_is_already_valid(names: &[String], item: &String) -> bool {
    names.binary_search(item).is_ok()
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
