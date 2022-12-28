use crate::{
    formatter::{self},
    name_matcher::matcher,
    type_colours::{self},
};

use rustemon::{
    client::RustemonClient,
    model::{pokemon::Type, resource::NamedApiResource},
    pokemon::type_,
};

const TYPE_HEADERS: (&str, &str, &str) = ("0x\n", "0.5x\n", "2x\n");

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
        self.build_defense_output(type_, output);
    }

    fn build_offense_output(&self, type_: &Type, output: &mut String) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.0),
            &type_relations.no_damage_to,
            output,
        );
        self.build_types_output(
            &formatter::bright_red(TYPE_HEADERS.1),
            &type_relations.half_damage_to,
            output,
        );
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.2),
            &type_relations.double_damage_to,
            output,
        );
    }

    fn build_defense_output(&self, type_: &Type, output: &mut String) {
        let type_relations = &type_.damage_relations;
        self.build_types_output(
            &formatter::green(TYPE_HEADERS.0),
            &type_relations.no_damage_from,
            output,
        );
        self.build_types_output(
            &formatter::bright_green(TYPE_HEADERS.1),
            &type_relations.half_damage_from,
            output,
        );
        self.build_types_output(
            &formatter::red(TYPE_HEADERS.2),
            &type_relations.double_damage_from,
            output,
        );
    }

    fn build_types_output(
        &self,
        header: &str,
        types: &Vec<NamedApiResource<Type>>,
        output: &mut String,
    ) {
        if types.is_empty() {
            return;
        }

        output.push_str(header);
        let mut type_names = types
            .iter()
            .map(|type_resource| &type_resource.name)
            .collect::<Vec<_>>();

        type_names.sort();

        let coloured_types = type_names
            .iter()
            .map(|type_name| type_colours::fetch(type_name))
            .collect::<Vec<_>>();

        output.push_str(&format!("  {}\n", coloured_types.join(" | ")));
    }
}
