use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self},
    name_matcher::{matcher, type_names},
    type_badge::{self},
};

use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use rustemon::model::{pokemon::Type, resource::NamedApiResource};
use tokio::try_join;

const EXCLUDED_TYPES: &[&str] = &["unknown", "shadow"];

enum DamageType {
    None,
    Quarter,
    Half,
    Normal,
    Double,
    Quadruple,
}

impl DamageType {
    const NONE_MULTIPLIER: &'static str = "0x\n";
    const QUARTER_MULTIPLIER: &'static str = "0.25x\n";
    const HALF_MULTIPLIER: &'static str = "0.5x\n";
    const NORMAL_MULTIPLIER: &'static str = "1x\n";
    const DOUBLE_MULTIPLIER: &'static str = "2x\n";
    const QUADRUPLE_MULTIPLIER: &'static str = "4x\n";

    fn multiplier(&self) -> &'static str {
        match self {
            DamageType::None => Self::NONE_MULTIPLIER,
            DamageType::Quarter => Self::QUARTER_MULTIPLIER,
            DamageType::Half => Self::HALF_MULTIPLIER,
            DamageType::Normal => Self::NORMAL_MULTIPLIER,
            DamageType::Double => Self::DOUBLE_MULTIPLIER,
            DamageType::Quadruple => Self::QUADRUPLE_MULTIPLIER,
        }
    }
}

enum DamageContext {
    Offence,
    Defence,
}

impl DamageContext {
    fn multiplier_header(&self, damage_type: DamageType) -> String {
        let formatter = self.formatter(&damage_type);
        let multiplier = damage_type.multiplier();

        formatter(multiplier)
    }

    fn formatter(&self, damage_type: &DamageType) -> fn(&str) -> String {
        match self {
            DamageContext::Offence => match damage_type {
                DamageType::None => formatter::red,
                DamageType::Half => formatter::bright_red,
                DamageType::Normal => formatter::yellow,
                DamageType::Double => formatter::green,

                // These are currently not possible for offence
                DamageType::Quarter => panic!("Quarter damage is not possible for offence"),
                DamageType::Quadruple => panic!("Quadruple damage is not possible for offence"),
            },
            DamageContext::Defence => match damage_type {
                DamageType::Quarter | DamageType::None => formatter::green,
                DamageType::Half => formatter::bright_green,
                DamageType::Normal => formatter::yellow,
                DamageType::Double => formatter::red,
                DamageType::Quadruple => formatter::bright_red,
            },
        }
    }
}

pub struct TypeCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    type_name: String,
    second_type_name: Option<String>,
    list_pokemon: bool,
}

impl TypeCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        type_name: String,
        second_type_name: Option<String>,
        list_pokemon: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        TypeCommand {
            builder: &mut builder,
            client,
            type_name,
            second_type_name,
            list_pokemon,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let (type_, second_type) = match self.fetch_types().await {
            Ok(types) => types,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        match second_type {
            Some(ref second_type) => self.append_dual_type_damage_details(&type_, second_type),
            None => self.append_single_type_damage_details(&type_),
        };

        if self.list_pokemon {
            self.append_pokemon_list(&type_, second_type.as_ref());
        }
    }

    async fn fetch_types(&self) -> Result<(Type, Option<Type>), String> {
        let types = match &self.second_type_name {
            Some(second_type_name) => {
                let (t1, t2) = try_join!(
                    self.fetch_type(&self.type_name),
                    self.fetch_type(second_type_name)
                )?;

                (t1, Some(t2))
            }
            None => (self.fetch_type(&self.type_name).await?, None),
        };

        Ok(types)
    }

    async fn fetch_type(&self, name: &str) -> Result<Type, String> {
        let successful_match = match matcher::match_type_name(name) {
            Ok(successful_match) => successful_match,
            Err(no_match) => {
                return Err(no_match.0);
            }
        };

        let result = self
            .client
            .fetch_type(&successful_match.suggested_name)
            .await;

        match result {
            Ok(type_) => Ok(type_),
            Err(_) => {
                let output = matcher::build_unknown_name(
                    &successful_match.keyword,
                    &successful_match.suggested_name,
                );
                Err(output)
            }
        }
    }

    fn append_pokemon_list(&mut self, type_: &Type, second_type: Option<&Type>) {
        let mut pokemon_names = {
            let type_pokemon_names = self.pokemon_names_from_type(type_);

            if let Some(second_type) = second_type {
                let second_type_pokemon_names = self
                    .pokemon_names_from_type(second_type)
                    .collect::<HashSet<_>>();

                type_pokemon_names
                    .collect::<HashSet<_>>()
                    .intersection(&second_type_pokemon_names)
                    .cloned()
                    .collect_vec()
            } else {
                type_pokemon_names.collect_vec()
            }
        };

        pokemon_names.sort();

        let formatted_pokemon = pokemon_names
            .iter()
            .map(|pokemon_name| format!("  {}", formatter::split_and_capitalise(pokemon_name)))
            .collect_vec();

        let num_pokemon = pokemon_names.len();
        let header = formatter::white(&format!("Pokemon ({num_pokemon})"));
        self.builder.appendln(header);

        if num_pokemon > 0 {
            self.builder
                .append(formatter::format_columns(&formatted_pokemon, 3));
        } else {
            self.builder
                .append(formatter::red("No pokemon with this type combination."));
        }
    }

    fn pokemon_names_from_type(&self, type_: &Type) -> impl Iterator<Item = String> {
        type_
            .pokemon
            .iter()
            .map(|type_pokemon| type_pokemon.pokemon.name.clone())
    }

    fn build_type_header(&self, type_: &Type, second_type: Option<&Type>) -> String {
        let formatted_type = self.formatted_type(type_);

        if let Some(second_type) = second_type {
            let second_formatted_type = self.formatted_type(second_type);
            format!("{formatted_type} | {second_formatted_type}")
        } else {
            formatted_type
        }
    }

    fn append_type_header(&mut self, type_: &Type, second_type: Option<&Type>) {
        let header = self.build_type_header(type_, second_type);
        self.builder.append(header);
        self.builder.newline();
        self.builder.newline();
    }

    fn append_single_type_header(&mut self, type_: &Type) {
        self.append_type_header(type_, None);
    }

    fn formatted_type(&self, type_: &Type) -> String {
        type_badge::fetch(&type_.name)
    }

    fn append_single_type_damage_details(&mut self, type_: &Type) {
        self.append_single_type_header(type_);

        self.builder.appendln(formatter::white("Offence"));

        self.append_single_damage_output(type_, DamageContext::Offence);
        self.builder.newline();

        self.builder.appendln(formatter::white("Defence"));
        self.append_single_damage_output(type_, DamageContext::Defence);
    }

    fn append_dual_type_damage_details(&mut self, type_: &Type, second_type: &Type) {
        self.append_type_header(type_, Some(second_type));

        self.builder.appendln(formatter::white("Offence"));

        self.append_single_type_header(type_);
        self.append_single_damage_output(type_, DamageContext::Offence);
        self.builder.newline();

        self.append_single_type_header(second_type);
        self.append_single_damage_output(second_type, DamageContext::Offence);
        self.builder.newline();

        self.builder.appendln(formatter::white("Defence"));
        self.append_dual_defence_output(type_, second_type);
    }

    fn append_single_damage_output(&mut self, type_: &Type, context: DamageContext) {
        let type_relations = &type_.damage_relations;

        let (no_damage_names, half_damage_names, double_damage_names) = match context {
            DamageContext::Offence => (
                self.to_type_names(&type_relations.no_damage_to),
                self.to_type_names(&type_relations.half_damage_to),
                self.to_type_names(&type_relations.double_damage_to),
            ),
            DamageContext::Defence => (
                self.to_type_names(&type_relations.no_damage_from),
                self.to_type_names(&type_relations.half_damage_from),
                self.to_type_names(&type_relations.double_damage_from),
            ),
        };

        let normal_damage_names = self.normal_damage_names_from(
            &no_damage_names,
            &half_damage_names,
            &double_damage_names,
        );

        self.append_types_output(&context, DamageType::None, &no_damage_names);
        self.append_types_output(&context, DamageType::Half, &half_damage_names);
        self.append_types_output(&context, DamageType::Normal, &normal_damage_names);
        self.append_types_output(&context, DamageType::Double, &double_damage_names);
    }

    fn normal_damage_names_from(
        &self,
        no_damage_names: &[String],
        half_damage_names: &[String],
        double_damage_names: &[String],
    ) -> Vec<String> {
        type_names::TYPE_NAMES
            .iter()
            .filter(|type_name| {
                !no_damage_names.contains(type_name)
                    && !half_damage_names.contains(type_name)
                    && !double_damage_names.contains(type_name)
                    && !EXCLUDED_TYPES.contains(&type_name.as_str())
            })
            .map(ToOwned::to_owned)
            .collect_vec()
    }

    fn append_dual_defence_output(&mut self, type_: &Type, second_type: &Type) {
        let (damage_relations, second_damage_relations) =
            (&type_.damage_relations, &second_type.damage_relations);

        let first_no_damage_from = self.to_type_names(&damage_relations.no_damage_from);
        let second_no_damage_from = self.to_type_names(&second_damage_relations.no_damage_from);
        let no_damage_from_types =
            self.build_combined_hash_set(first_no_damage_from, second_no_damage_from);

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
        let mut normal_damage_types: HashSet<String> = HashSet::new();

        type_names::TYPE_NAMES
            .iter()
            .filter(|type_name| !no_damage_from_types.contains(type_name.as_str()))
            .for_each(|type_name| {
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
                        if !EXCLUDED_TYPES.contains(&type_name.as_str()) {
                            normal_damage_types.insert(type_name.to_owned());
                        }
                    }
                }
            });

        let context = &DamageContext::Defence;
        self.append_types_output(context, DamageType::None, &no_damage_from_types);
        self.append_types_output(context, DamageType::Quarter, &quarter_damage_types);
        self.append_types_output(context, DamageType::Half, &half_damage_types);
        self.append_types_output(context, DamageType::Normal, &normal_damage_types);
        self.append_types_output(context, DamageType::Double, &double_damage_types);
        self.append_types_output(context, DamageType::Quadruple, &quad_damage_types);
    }

    fn build_type_counter(&self, a: Vec<String>, b: Vec<String>) -> HashMap<String, i8> {
        let mut counts: HashMap<String, i8> = HashMap::new();

        self.increment_counts(&mut counts, &a);
        self.increment_counts(&mut counts, &b);

        counts
    }

    fn increment_counts(&self, counts: &mut HashMap<String, i8>, vec: &[String]) {
        for t in vec {
            let value = counts.entry(t.to_owned()).or_insert(0);
            *value += 1;
        }
    }

    fn build_combined_hash_set(&self, a: Vec<String>, b: Vec<String>) -> HashSet<String> {
        let mut hash_set = HashSet::new();

        for e in a {
            hash_set.insert(e);
        }

        for e in b {
            hash_set.insert(e);
        }

        hash_set
    }

    fn to_type_names(&self, resources: &[NamedApiResource<Type>]) -> Vec<String> {
        resources
            .iter()
            .map(|type_resource| type_resource.name.clone())
            .collect_vec()
    }

    fn append_types_output<'a, I>(
        &mut self,
        damage_context: &DamageContext,
        damage_type: DamageType,
        type_names: I,
    ) where
        I: IntoIterator<Item = &'a String>,
    {
        let mut iter = type_names.into_iter().peekable();

        if iter.peek().is_none() {
            return;
        }

        let header = damage_context.multiplier_header(damage_type);
        self.builder.append(header);

        let mut coloured_types = iter.sorted().map(|type_name| type_badge::fetch(type_name));

        self.builder
            .appendln(format!("  {}", coloured_types.join(" | ")));
    }
}
