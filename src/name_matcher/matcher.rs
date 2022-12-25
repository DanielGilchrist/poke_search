use crate::{
    formatter::capitalise,
    name_matcher::{move_names::MOVE_NAMES, pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES},
};

use std::process::exit;

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

pub fn try_suggest_name(name: &str, matcher_type: MatcherType) -> ! {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);

    match name_matcher.find_match(name) {
        Some(similar_name) => {
            println!("Unknown {} \"{}\"", keyword, name);
            println!("Did you mean \"{}\"?", similar_name);
            exit(1);
        }
        None => {
            println!("{} \"{}\" doesn't exist", capitalise(&keyword), name);
            exit(1);
        }
    }
}

fn matcher_and_keyword(matcher_type: MatcherType) -> (NameMatcher, String) {
    match matcher_type {
        MatcherType::Move => (move_matcher(), String::from("move")),
        MatcherType::Pokemon => (pokemon_matcher(), String::from("pokemon")),
        MatcherType::Type => (type_matcher(), String::from("type")),
    }
}

fn pokemon_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&POKEMON_NAMES).to_owned())
}

fn type_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&TYPE_NAMES).to_owned())
}

fn move_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&MOVE_NAMES).to_owned())
}
