use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self},
    name_matcher::{matcher, type_names},
    type_colours::{self},
};

use std::collections::{HashMap, HashSet};

use rustemon::model::{pokemon::Type, resource::NamedApiResource};

const TYPE_HEADERS: (&str, &str, &str, &str, &str) = ("0x\n", "0.25x\n", "0.5x\n", "2x\n", "4x\n");

pub struct TypeCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    type_name: String,
    second_type_name: Option<String>,
}

impl TypeCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        type_name: String,
        second_type_name: Option<String>,
    ) -> Builder {
        let mut builder = Builder::default();

        TypeCommand {
            builder: &mut builder,
            client,
            type_name,
            second_type_name,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let type_name_ref = &self.type_name.clone();
        let type_ = if let Some(type_) = self.fetch_type(type_name_ref).await {
            type_
        } else {
            self.handle_invalid_type(type_name_ref);
            return;
        };

        let second_type_name_ref = &self.second_type_name.clone();
        match second_type_name_ref {
            Some(second_type_name) => match self.fetch_type(second_type_name).await {
                Some(second_type) => self.build_dual_damage_details(&type_, &second_type),
                None => self.handle_invalid_type(second_type_name),
            },

            None => self.build_single_damage_details(&type_),
        };
    }

    async fn fetch_type(&self, type_name: &str) -> Option<Type> {
        self.client.fetch_type(type_name).await.ok()
    }

    fn handle_invalid_type(&mut self, type_name: &str) {
        let suggestion = matcher::try_suggest_name(type_name, matcher::MatcherType::Type);
        self.builder.append(suggestion);
    }

    fn build_type_header(&mut self, type_: &Type, second_type: Option<&Type>) {
        let formatted_type = self.formatted_type(type_);

        let result = match second_type {
            Some(second_type) => {
                let second_formatted_type = self.formatted_type(second_type);
                format!("{} | {}", formatted_type, second_formatted_type)
            }
            None => formatted_type,
        };

        self.builder.append(format!("{}\n\n", result));
    }

    fn build_single_type_header(&mut self, type_: &Type) {
        self.build_type_header(type_, None);
    }

    fn formatted_type(&self, type_: &Type) -> String {
        type_colours::fetch(&type_.name)
    }

    fn build_single_damage_details(&mut self, type_: &Type) {
        self.build_single_type_header(type_);

        self.builder.append(formatter::white("Offense\n"));

        self.build_offense_output(type_);
        self.builder.append_c('\n');

        self.builder.append(formatter::white("Defense\n"));
        self.build_defense_output(type_);
    }

    fn build_dual_damage_details(&mut self, type_: &Type, second_type: &Type) {
        self.build_type_header(type_, Some(second_type));

        self.builder.append(formatter::white("Offense\n"));

        self.build_single_type_header(type_);
        self.build_offense_output(type_);
        self.builder.append_c('\n');

        self.build_single_type_header(second_type);
        self.build_offense_output(second_type);
        self.builder.append_c('\n');

        self.builder.append(formatter::white("Defense\n"));
        self.build_dual_defense_output(type_, second_type);
    }

    fn build_offense_output(&mut self, type_: &Type) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.0),
            &self.to_type_names(&type_relations.no_damage_to),
        );
        self.build_types_output(
            &formatter::bright_red(TYPE_HEADERS.2),
            &self.to_type_names(&type_relations.half_damage_to),
        );
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.3),
            &self.to_type_names(&type_relations.double_damage_to),
        );
    }

    fn build_defense_output(&mut self, type_: &Type) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.0),
            &self.to_type_names(&type_relations.no_damage_from),
        );
        self.build_types_output(
            &formatter::bright_green(TYPE_HEADERS.2),
            &self.to_type_names(&type_relations.half_damage_from),
        );
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.3),
            &self.to_type_names(&type_relations.double_damage_from),
        );
    }

    fn build_dual_defense_output(&mut self, type_: &Type, second_type: &Type) {
        let (damage_relations, second_damage_relations) =
            (&type_.damage_relations, &second_type.damage_relations);

        let first_no_damage_from = self.to_type_names(&damage_relations.no_damage_from);
        let second_no_damage_from = self.to_type_names(&second_damage_relations.no_damage_from);
        let no_damage_from_types =
            self.build_combined_hash_set(first_no_damage_from, second_no_damage_from);

        self.build_types_output(&formatter::green(TYPE_HEADERS.0), &no_damage_from_types);

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

        self.build_types_output(&formatter::green(TYPE_HEADERS.1), &quarter_damage_types);
        self.build_types_output(&formatter::bright_green(TYPE_HEADERS.2), &half_damage_types);
        self.build_types_output(&formatter::bright_red(TYPE_HEADERS.3), &double_damage_types);
        self.build_types_output(&formatter::red(TYPE_HEADERS.4), &quad_damage_types);
    }

    fn build_type_counter(&self, a: Vec<String>, b: Vec<String>) -> HashMap<String, i8> {
        let mut counts: HashMap<String, i8> = HashMap::new();

        self.increment_counts(&mut counts, &a);
        self.increment_counts(&mut counts, &b);

        counts
    }

    fn increment_counts(&self, counts: &mut HashMap<String, i8>, vec: &[String]) {
        vec.iter().for_each(|t| {
            let value = counts.entry(t.to_owned()).or_insert(0);
            *value += 1;
        });
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

    fn build_types_output<I>(&mut self, header: &str, type_names: &I)
    where
        for<'a> &'a I: IntoIterator<Item = &'a String>,
    {
        let mut iter = type_names.into_iter().peekable();

        if iter.peek().is_none() {
            return;
        }

        self.builder.append(header);

        let mut new_type_names = iter.collect::<Vec<_>>();
        new_type_names.sort();

        let coloured_types = new_type_names
            .iter()
            .map(|type_name| type_colours::fetch(type_name))
            .collect::<Vec<_>>();

        self.builder
            .append(format!("  {}\n", coloured_types.join(" | ")));
    }
}
