use super::FormatModel;
use crate::formatter::utils::{extract_effect, formatln, split_and_capitalise, white};

use rustemon::model::items::Item;

pub struct FormatItem {
    item: Item,
    verbose: bool,
}

impl FormatItem {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    fn build_category(&self, output: &mut String) {
        let category_name = split_and_capitalise(&self.item.category.name);
        output.push_str(&formatln(&white("Category"), &category_name));
    }

    fn build_effect(&self, output: &mut String) {
        let effect_entries = &self.item.effect_entries;
        let effect = extract_effect(effect_entries, self.verbose);

        if let Some(effect) = effect {
            output.push_str(&formatln(&white("Effect"), &effect));
        }
    }
}

impl FormatModel for FormatItem {
    fn format(&self) -> String {
        let mut output = String::new();

        let item_name = split_and_capitalise(&self.item.name);
        output.push_str(&formatln(&white("Name"), &item_name));

        self.build_category(&mut output);
        self.build_effect(&mut output);

        output
    }
}
