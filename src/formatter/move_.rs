use crate::{
    formatter::utils::{
        clean_and_wrap_text, formatln, parse_maybe_i64, split_and_capitalise, white,
    },
    type_badge::{self},
};

use super::FormatModel;

use rustemon::model::{
    moves::{Move, MoveLearnMethod},
    pokemon::PokemonMoveVersion,
};

struct MoveDetails {
    level_learned_at: i64,
    move_learn_method: MoveLearnMethod,
}

pub struct FormatMove {
    pub move_: Move,
    move_details: Option<MoveDetails>,
}

impl FormatMove {
    pub fn new(move_: Move) -> Self {
        FormatMove {
            move_,
            move_details: None,
        }
    }

    pub fn with_maybe_details(
        move_: Move,
        move_learn_method: Option<MoveLearnMethod>,
        version_group_details: Option<&PokemonMoveVersion>,
    ) -> Self {
        if let (Some(move_learn_method), Some(version_group_details)) =
            (move_learn_method, version_group_details)
        {
            return FormatMove {
                move_,
                move_details: Some(MoveDetails {
                    move_learn_method,
                    level_learned_at: version_group_details.level_learned_at,
                }),
            };
        }

        Self::new(move_)
    }

    fn build_summary(&self, output: &mut String) {
        let formatted_name = split_and_capitalise(&self.move_.name);

        output.push_str(&formatln(&white("Name"), &formatted_name));
        output.push_str(&formatln(
            &white("Type"),
            &type_badge::fetch(&self.move_.type_.name),
        ));
        output.push_str(&formatln(
            &white("Damage Type"),
            &self.move_.damage_class.name,
        ));
    }

    fn build_details(&self, output: &mut String) {
        let power = parse_maybe_i64(self.move_.power);

        output.push_str(&formatln(&white("Power"), &power));
        output.push_str(&formatln(
            &white("Accuracy"),
            &parse_maybe_i64(self.move_.accuracy),
        ));
        output.push_str(&formatln(&white("PP"), &parse_maybe_i64(self.move_.pp)));
        output.push_str(&formatln(
            &white("Priority"),
            &self.move_.priority.to_string(),
        ));

        if let Some(flavour_text) = self.flavour_text() {
            output.push_str(&formatln(&white("Description"), &flavour_text));
        }

        self.build_effects(power, output);
    }

    fn flavour_text(&self) -> Option<String> {
        let text = self.move_.flavor_text_entries.iter().find_map(|entry| {
            if entry.language.name == "en" {
                Some(&entry.flavor_text)
            } else {
                None
            }
        })?;

        Some(text.replace('\n', " "))
    }

    fn build_effects(&self, power: String, output: &mut String) {
        let effect_chance = format!("{}%", parse_maybe_i64(self.move_.effect_chance));
        self.move_.effect_entries.iter().for_each(|entry| {
            let description = if power == "-" {
                entry.effect.replace('\n', " ").replace("  ", " ")
            } else {
                entry
                    .short_effect
                    .replace("$effect_chance%", &effect_chance)
            };

            let wrapped_description = clean_and_wrap_text(&description, 4, 80);
            output.push_str(&formatln(&white("Effect"), &wrapped_description));
        });
    }

    fn build_move_learn_details(&self, output: &mut String) {
        if let Some(move_details) = &self.move_details {
            if let Some(description) =
                self.find_move_learn_description(&move_details.move_learn_method)
            {
                output.push_str(&formatln(&white("Learn Method"), &description));
            }

            let level_learned_at = move_details.level_learned_at;
            if level_learned_at > 0 {
                output.push_str(&formatln(
                    &white("Learn Level"),
                    &level_learned_at.to_string(),
                ));
            }
        }
    }

    fn find_move_learn_description(&self, move_learn_method: &MoveLearnMethod) -> Option<String> {
        let english_description = move_learn_method
            .descriptions
            .iter()
            .find(|description| description.language.name == "en")?;

        Some(english_description.description.clone())
    }
}

impl FormatModel for FormatMove {
    fn format(&self) -> String {
        let mut output = String::new();

        self.build_summary(&mut output);
        self.build_details(&mut output);
        self.build_move_learn_details(&mut output);

        output
    }
}
