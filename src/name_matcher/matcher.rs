use crate::{
    formatter::capitalise,
    name_matcher::{
        ability_names::ABILITY_NAMES, generation_names::GENERATION_NAMES, item_names::ITEM_NAMES,
        move_damage_class_names::MOVE_DAMAGE_CLASS_NAMES, move_names::MOVE_NAMES,
        pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES,
    },
};

use std::sync::LazyLock;

use ngrammatic::{Corpus, CorpusBuilder, Pad, SearchResult};

static MIN_CERTAIN_SIMILARITY: f32 = 0.71;

pub enum MatcherType {
    Ability,
    Generation,
    Item,
    Pokemon,
    Move,
    MoveDamageCategory,
    Type,
}

pub struct SuccessfulMatch {
    pub suggested_name: String,
    pub keyword: String,
}

impl SuccessfulMatch {
    fn new(keyword: String, suggestion: Suggestion) -> Self {
        Self {
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

impl From<&SearchResult> for Certainty {
    fn from(search_result: &SearchResult) -> Self {
        if search_result.similarity > MIN_CERTAIN_SIMILARITY {
            Certainty::Certain
        } else {
            Certainty::Uncertain
        }
    }
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
        println!("\n[DEBUG] Similar Results: {search_results:?}\n");

        let search_result = search_results.into_iter().next()?;
        let certainty = Certainty::from(&search_result);

        Some(Suggestion::new(search_result.text, certainty))
    }

    fn build_corpus(&self) -> Corpus {
        CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .fill(&self.names)
            .finish()
    }
}

pub fn match_name(name: &str, matcher_type: MatcherType) -> Result<SuccessfulMatch, NoMatch> {
    let (name_matcher, keyword) = matcher_and_keyword(matcher_type);

    if name_is_already_valid(&name_matcher.names, &name.to_owned()) {
        let suggestion = Suggestion::certain(name.to_owned());
        return Ok(SuccessfulMatch::new(keyword, suggestion));
    }

    let suggestion = name_matcher
        .find_match(name)
        .ok_or_else(|| NoMatch::new(build_unknown_name(&keyword, name)))?;

    match suggestion.certainty {
        Certainty::Certain => Ok(SuccessfulMatch::new(keyword, suggestion)),
        Certainty::Uncertain => Err(NoMatch::new(build_suggested_name(
            &keyword,
            name,
            &suggestion.name,
        ))),
    }
}

pub fn is_valid(name: &str, matcher_type: MatcherType) -> bool {
    let (name_matcher, _) = matcher_and_keyword(matcher_type);
    name_is_already_valid(&name_matcher.names, &name.to_owned())
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
        MatcherType::Ability => (&ABILITY_NAMES, "ability"),
        MatcherType::Generation => (&GENERATION_NAMES, "generation"),
        MatcherType::Item => (&ITEM_NAMES, "item"),
        MatcherType::Move => (&MOVE_NAMES, "move"),
        MatcherType::MoveDamageCategory => (&MOVE_DAMAGE_CLASS_NAMES, "move damage category"),
        MatcherType::Pokemon => (&POKEMON_NAMES, "pokemon"),
        MatcherType::Type => (&TYPE_NAMES, "type"),
    };

    (
        NameMatcher::new(LazyLock::force(names).to_owned()),
        String::from(keyword),
    )
}
