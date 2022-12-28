use crate::{
    formatter::{self},
    name_matcher::{matcher, type_names},
    type_colours::{self},
};

use std::collections::{HashMap, HashSet};

use rustemon::{
    client::RustemonClient,
    model::{pokemon::Type, resource::NamedApiResource},
    pokemon::type_,
};

const TYPE_HEADERS: (&str, &str, &str, &str, &str) = ("0x\n", "0.25x\n", "0.5x\n", "2x\n", "4x\n");

pub struct TypeCommand {
    client: RustemonClient,
    type_name: String,
    second_type_name: Option<String>,
}

impl TypeCommand {
    pub async fn execute(
        client: RustemonClient,
        type_name: String,
        second_type_name: Option<String>,
    ) {
        TypeCommand {
            client,
            type_name,
            second_type_name,
        }
        ._execute()
        .await
    }

    async fn _execute(&self) {
        let type_ = self.fetch_type(&self.type_name).await;
        let second_type = if let Some(second_type_name) = &self.second_type_name {
            Some(self.fetch_type(second_type_name).await)
        } else {
            None
        };

        let mut output = String::from("\n");

        self.build_type_header(&type_, &second_type, &mut output);

        match second_type {
            Some(second_type) => {
                self.build_dual_damage_details(&type_, &second_type, &mut output);
            }
            None => {
                self.build_single_damage_details(&type_, &mut output);
            }
        }

        println!("{}", output);
    }

    async fn fetch_type(&self, type_name: &str) -> Type {
        match type_::get_by_name(type_name, &self.client).await {
            Ok(type_) => type_,
            Err(_) => matcher::try_suggest_name(type_name, matcher::MatcherType::Type),
        }
    }

    fn build_type_header(&self, type_: &Type, second_type: &Option<Type>, output: &mut String) {
        let formatted_type = self.formatted_type(type_);

        let result = match second_type {
            Some(second_type) => {
                let second_formatted_type = self.formatted_type(second_type);
                format!("{} | {}", formatted_type, second_formatted_type)
            }
            None => formatted_type,
        };

        output.push_str(&format!("{}\n\n", result));
    }

    fn build_single_type_header(&self, type_: &Type, output: &mut String) {
        self.build_type_header(type_, &None, output);
    }

    fn formatted_type(&self, type_: &Type) -> String {
        type_colours::fetch(&type_.name)
    }

    fn build_single_damage_details(&self, type_: &Type, output: &mut String) {
        output.push_str(&formatter::white("Offense\n"));

        self.build_offense_output(type_, output);
        output.push('\n');

        output.push_str(&formatter::white("Defense\n"));
        self.build_defense_output(type_, output);
    }

    fn build_dual_damage_details(&self, type_: &Type, second_type: &Type, output: &mut String) {
        output.push_str(&formatter::white("Offense\n"));

        self.build_single_type_header(type_, output);
        self.build_offense_output(type_, output);
        output.push('\n');

        self.build_single_type_header(second_type, output);
        self.build_offense_output(second_type, output);
        output.push('\n');

        output.push_str(&formatter::white("Defense\n"));
        self.build_dual_defense_output(type_, second_type, output);
    }

    fn build_offense_output(&self, type_: &Type, output: &mut String) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.0),
            &self.to_type_names(&type_relations.no_damage_to),
            output,
        );
        self.build_types_output(
            &formatter::bright_red(TYPE_HEADERS.2),
            &self.to_type_names(&type_relations.half_damage_to),
            output,
        );
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.3),
            &self.to_type_names(&type_relations.double_damage_to),
            output,
        );
    }

    fn build_defense_output(&self, type_: &Type, output: &mut String) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.0),
            &self.to_type_names(&type_relations.no_damage_from),
            output,
        );
        self.build_types_output(
            &formatter::bright_green(TYPE_HEADERS.2),
            &self.to_type_names(&type_relations.half_damage_from),
            output,
        );
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.3),
            &self.to_type_names(&type_relations.double_damage_from),
            output,
        );
    }

    fn build_dual_defense_output(&self, type_: &Type, second_type: &Type, output: &mut String) {
        let (damage_relations, second_damage_relations) =
            (&type_.damage_relations, &second_type.damage_relations);

        let first_no_damage_from = self.to_type_names(&damage_relations.no_damage_from);
        let second_no_damage_from = self.to_type_names(&second_damage_relations.no_damage_from);
        let no_damage_from_types =
            self.build_combined_hash_set(first_no_damage_from, second_no_damage_from);

        self.build_types_output(
            &formatter::green(TYPE_HEADERS.0),
            &no_damage_from_types,
            output,
        );

        let first_half_damage_from = self.to_type_names(&damage_relations.half_damage_from);
        let second_half_damage_from = self.to_type_names(&second_damage_relations.half_damage_from);
        let half_damage_counts =
            self.build_type_counter(first_half_damage_from, second_half_damage_from);

        let first_double_damage_from = self.to_type_names(&damage_relations.double_damage_from);
        let second_double_damage_from =
            self.to_type_names(&second_damage_relations.double_damage_from);
        let double_damage_counts =
            self.build_type_counter(first_double_damage_from, second_double_damage_from);

        let mut quarter_damage_types: HashSet<String> = HashSet::new();
        let mut half_damage_types: HashSet<String> = HashSet::new();
        let mut double_damage_types: HashSet<String> = HashSet::new();
        let mut quad_damage_types: HashSet<String> = HashSet::new();

        type_names::TYPE_NAMES.iter().for_each(|type_name| {
            if no_damage_from_types.contains(type_name) {
                 // no-op
            } else {
                let half_damage_score = -half_damage_counts.get(type_name).unwrap_or(&0);
                let double_damage_score = double_damage_counts.get(type_name).unwrap_or(&0);

                match double_damage_score + half_damage_score {
                    -2 => {
                        quarter_damage_types.insert(type_name.to_owned());
                    }

                    -1 => {
                        half_damage_types.insert(type_name.to_owned());
                    }

                    1 => {
                        double_damage_types.insert(type_name.to_owned());
                    }

                    2 => {
                        quad_damage_types.insert(type_name.to_owned());
                    }

                    _ => {
                         // no-op
                    }
                }
            }
        });

        self.build_types_output(
            &formatter::green(TYPE_HEADERS.1),
            &quarter_damage_types,
            output,
        );
        self.build_types_output(
            &formatter::bright_green(TYPE_HEADERS.2),
            &half_damage_types,
            output,
        );
        self.build_types_output(
            &formatter::bright_red(TYPE_HEADERS.3),
            &double_damage_types,
            output,
        );
        self.build_types_output(&formatter::red(TYPE_HEADERS.4), &quad_damage_types, output);
    }

    fn build_type_counter(&self, a: Vec<String>, b: Vec<String>) -> HashMap<String, i8> {
        let mut counts: HashMap<String, i8> = HashMap::new();

        let mut update_map = |vec: Vec<String>| {
            vec.iter().for_each(|t| {
                let value = counts.entry(t.to_owned()).or_insert(0);
                *value += 1;
            });
        };

        update_map(a);
        update_map(b);

        counts
    }

    fn build_combined_hash_set(&self, a: Vec<String>, b: Vec<String>) -> HashSet<String> {
        let mut hash_set = HashSet::new();

        a.into_iter().for_each(|e| {
            hash_set.insert(e);
        });

        b.into_iter().for_each(|e| {
            hash_set.insert(e);
        });

        hash_set
    }

    fn to_type_names(&self, resources: &[NamedApiResource<Type>]) -> Vec<String> {
        resources
            .iter()
            .map(|type_resource| type_resource.name.to_owned())
            .collect::<Vec<_>>()
    }

    fn build_types_output<I>(&self, header: &str, type_names: &I, output: &mut String)
    where
        for<'a> &'a I: IntoIterator<Item = &'a String>,
    {
        let mut iter = type_names.into_iter().peekable();

        if iter.peek().is_none() {
            return;
        }

        output.push_str(header);

        let mut new_type_names = iter.collect::<Vec<_>>();
        new_type_names.sort();

        let coloured_types = new_type_names
            .iter()
            .map(|type_name| type_colours::fetch(type_name))
            .collect::<Vec<_>>();

        output.push_str(&format!("  {}\n", coloured_types.join(" | ")));
    }
}
