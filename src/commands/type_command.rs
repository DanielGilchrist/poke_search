use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self},
    name_matcher::{matcher, type_names},
    type_colours::{self},
};

use std::collections::{HashMap, HashSet};

use rustemon::model::{pokemon::Type, resource::NamedApiResource};

const EXCLUDED_TYPES: &[&str] = &["unknown", "shadow"];

enum DamageType {
    None,
    Quarter,
    Half,
    Normal,
    Double,
    Quadruple,
}

enum DamageContext {
    Offence,
    Defence,
}

impl DamageContext {
    fn coloured_header(&self, damage_type: DamageType) -> String {
        let multiplier = match damage_type {
            DamageType::None => "0x\n",
            DamageType::Quarter => "0.25x\n",
            DamageType::Half => "0.5x\n",
            DamageType::Normal => "1x\n",
            DamageType::Double => "2x\n",
            DamageType::Quadruple => "4x\n",
        };

        let formatter = match self {
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
                DamageType::None => formatter::green,
                DamageType::Quarter => formatter::green,
                DamageType::Half => formatter::bright_green,
                DamageType::Normal => formatter::yellow,
                DamageType::Double => formatter::red,
                DamageType::Quadruple => formatter::bright_red,
            },
        };

        formatter(multiplier)
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
        let type_name_ref = &self.type_name;
        let Ok(type_) = self.fetch_type(type_name_ref).await else {
            self.handle_invalid_type(&type_name_ref.clone());
            return;
        };

        let second_type_name_ref = &self.second_type_name;
        let second_type = match second_type_name_ref {
            Some(second_type_name) => {
                let Ok(second_type) = self.fetch_type(second_type_name).await else {
                    self.handle_invalid_type(&second_type_name.clone());
                    return;
                };

                Some(second_type)
            }
            None => None,
        };

        match second_type {
            Some(ref second_type) => self.append_dual_type_damage_details(&type_, second_type),
            None => self.append_single_type_damage_details(&type_),
        };

        if self.list_pokemon {
            self.append_pokemon_list(&type_, second_type.as_ref());
        }
    }

    async fn fetch_type(&self, type_name: &str) -> Result<Type, rustemon::error::Error> {
        self.client.fetch_type(type_name).await
    }

    fn append_pokemon_list(&mut self, type_: &Type, second_type: Option<&Type>) {
        let mut pokemon_names = if let Some(second_type) = second_type {
            let type_pokemon_names = self.pokemon_names_from_type(type_);
            let second_type_pokemon_names = self.pokemon_names_from_type(second_type);

            type_pokemon_names
                .intersection(&second_type_pokemon_names)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.pokemon_names_from_type(type_)
                .into_iter()
                .collect::<Vec<_>>()
        };

        pokemon_names.sort();

        let formatted_pokemon = pokemon_names
            .iter()
            .map(|pokemon_name| format!("  {}", formatter::split_and_capitalise(pokemon_name)))
            .collect::<Vec<_>>()
            .join("\n");

        let num_pokemon = pokemon_names.len();
        let header = formatter::white(&format!("\nPokemon ({num_pokemon})\n"));
        self.builder.append(header);

        if num_pokemon > 0 {
            self.builder.append(formatted_pokemon);
        } else {
            self.builder
                .append(formatter::red("No pokemon with this type combination."));
        }
    }

    fn pokemon_names_from_type(&self, type_: &Type) -> HashSet<String> {
        type_
            .pokemon
            .iter()
            .map(|type_pokemon| type_pokemon.pokemon.name.clone())
            .collect::<HashSet<_>>()
    }

    fn handle_invalid_type(&mut self, type_name: &str) {
        let suggestion = matcher::try_suggest_name(type_name, matcher::MatcherType::Type);
        self.builder.append(suggestion);
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
        self.builder.append(format!("{header}\n\n"));
    }

    fn append_single_type_header(&mut self, type_: &Type) {
        self.append_type_header(type_, None);
    }

    fn formatted_type(&self, type_: &Type) -> String {
        type_colours::fetch(&type_.name)
    }

    fn append_single_type_damage_details(&mut self, type_: &Type) {
        self.append_single_type_header(type_);

        self.builder.append(formatter::white("Offence\n"));

        self.append_single_damage_output(type_, DamageContext::Offence);
        self.builder.append_c('\n');

        self.builder.append(formatter::white("Defence\n"));
        self.append_single_damage_output(type_, DamageContext::Defence);
    }

    fn append_dual_type_damage_details(&mut self, type_: &Type, second_type: &Type) {
        self.append_type_header(type_, Some(second_type));

        self.builder.append(formatter::white("Offence\n"));

        self.append_single_type_header(type_);
        self.append_single_damage_output(type_, DamageContext::Offence);
        self.builder.append_c('\n');

        self.append_single_type_header(second_type);
        self.append_single_damage_output(second_type, DamageContext::Offence);
        self.builder.append_c('\n');

        self.builder.append(formatter::white("Defence\n"));
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
            .map(|type_name| type_name.to_owned())
            .collect::<Vec<_>>()
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

    fn append_types_output<I>(
        &mut self,
        damage_context: &DamageContext,
        damage_type: DamageType,
        type_names: &I,
    ) where
        for<'a> &'a I: IntoIterator<Item = &'a String>,
    {
        let mut iter = type_names.into_iter().peekable();

        if iter.peek().is_none() {
            return;
        }

        let header = damage_context.coloured_header(damage_type);
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
