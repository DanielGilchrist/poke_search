use crate::{
    formatter::capitalise,
    name_matcher::{
        move_damage_class_names::MOVE_DAMAGE_CLASS_NAMES, move_names::MOVE_NAMES,
        pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES,
    },
};

use ngrammatic::{Corpus, CorpusBuilder, Pad};
use once_cell::sync::Lazy;

static MIN_CERTAIN_SIMILARITY: f32 = 0.74;

pub enum MatcherType {
    Pokemon,
    Move,
    MoveDamageCategory,
    Type,
}

pub struct SuccessfulMatch {
    pub original_name: String,
    pub suggested_name: String,
    pub keyword: String,
}

impl SuccessfulMatch {
    fn new(original_name: String, keyword: String, suggestion: Suggestion) -> Self {
        Self {
            original_name,
            suggested_name: suggestion.name,
            keyword,
        }
    }
}

pub struct NoMatch(pub String);

impl NoMatch {
    fn new(message: String) -> Self {
        Self(message)
    }
}

enum Certainty {
    Certain,
    Uncertain,
}

struct Suggestion {
    name: String,
    certainty: Certainty,
}

impl Suggestion {
    fn new(name: String, certainty: Certainty) -> Self {
        Self { name, certainty }
    }

    fn certain(name: String) -> Self {
        Self::new(name, Certainty::Certain)
    }
}

struct NameMatcher {
    names: Vec<String>,
}

impl NameMatcher {
    fn new(names: Vec<String>) -> Self {
        NameMatcher { names }
    }

    fn find_match(&self, name: &str) -> Option<Suggestion> {
        let corpus = self.build_corpus();
        let search_results = corpus.search(name, 0.25);

        #[cfg(debug_assertions)]
        println!("\n[DEBUG] Similar Results: {:?}\n", search_results);

        let search_result = search_results.first().map(ToOwned::to_owned)?;

        let certainty = if search_result.similarity > MIN_CERTAIN_SIMILARITY {
            Certainty::Certain
        } else {
            Certainty::Uncertain
        };

        Some(Suggestion::new(search_result.text, certainty))
    }

    fn build_corpus(&self) -> Corpus {
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

        for name in &self.names {
            corpus.add_text(name);
        }

        corpus
    }
}

pub fn match_name(name: &str, matcher_type: MatcherType) -> Result<SuccessfulMatch, NoMatch> {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);

    if name_is_already_valid(&name_matcher.names, &name.to_owned()) {
        let suggestion = Suggestion::certain(name.to_owned());
        let successful_match = SuccessfulMatch::new(name.to_owned(), keyword, suggestion);

        return Ok(successful_match);
    }

    match name_matcher.find_match(name) {
        Some(suggestion) => match suggestion.certainty {
            Certainty::Certain => {
                let successful_match = SuccessfulMatch::new(name.to_owned(), keyword, suggestion);

                Ok(successful_match)
            }
            Certainty::Uncertain => Err(NoMatch::new(build_suggested_name(
                &keyword,
                name,
                &suggestion.name,
            ))),
        },

        None => Err(NoMatch::new(build_unknown_name(&keyword, name))),
    }
}

pub fn build_suggested_name(keyword: &str, name: &str, suggestion: &str) -> String {
    format!("Unknown {keyword} \"{name}\"\nDid you mean \"{suggestion}\"?")
}

pub fn build_unknown_name(keyword: &str, name: &str) -> String {
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
        MatcherType::MoveDamageCategory => (&MOVE_DAMAGE_CLASS_NAMES, "move damage category"),
    };

    (
        NameMatcher::new(Lazy::force(names).to_owned()),
        String::from(keyword),
    )
}
