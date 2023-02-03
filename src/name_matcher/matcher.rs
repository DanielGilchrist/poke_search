use crate::{
    formatter::capitalise,
    name_matcher::{move_names::MOVE_NAMES, pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES},
};

use ngrammatic::{Corpus, CorpusBuilder, Pad};
use once_cell::sync::Lazy;

static MIN_SIMILARITY: f32 = 0.4;

pub enum MatcherType {
    Pokemon,
    Move,
    Type,
}

pub struct NameMatcher {
    names: Vec<String>,
}

impl NameMatcher {
    pub fn new(names: Vec<String>) -> Self {
        NameMatcher { names }
    }

    pub fn find_match(&self, name: &str) -> Option<String> {
        let corpus = self.build_corpus();
        let search_results = corpus.search(name, 0.25);
        let search_result = search_results.first().map(|r| r.to_owned())?;

        if search_result.similarity > MIN_SIMILARITY {
            Some(search_result.text)
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

pub fn try_suggest_name(name: &str, matcher_type: MatcherType) -> String {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);
    let mut output = String::new();

    match name_matcher.find_match(name) {
        Some(similar_name) => output.push_str(&format!(
            "Unknown {keyword} \"{name}\"\nDid you mean \"{similar_name}\"?"
        )),

        None => output.push_str(&format!(
            "{} \"{}\" doesn't exist",
            capitalise(&keyword),
            name
        )),
    }

    output
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
