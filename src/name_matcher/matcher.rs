use crate::name_matcher::{
    move_names::MOVE_NAMES, pokemon_names::POKEMON_NAMES, type_names::TYPE_NAMES,
};

use ngrammatic::{Corpus, CorpusBuilder, Pad};
use once_cell::sync::Lazy;

static MIN_SIMILARITY: f32 = 0.4;

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

pub fn pokemon_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&POKEMON_NAMES).to_owned())
}

pub fn type_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&TYPE_NAMES).to_owned())
}

pub fn move_matcher() -> NameMatcher {
    NameMatcher::new(Lazy::force(&MOVE_NAMES).to_owned())
}
