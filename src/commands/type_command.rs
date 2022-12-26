use crate::{
    formatter::{self},
    name_matcher::matcher,
    type_colours::{self},
};

use rustemon::{
    client::RustemonClient,
    model::{
        pokemon::{Type, TypeRelations},
        resource::NamedApiResource,
    },
    pokemon::ability,
    pokemon::type_,
};

pub struct TypeCommand {
    client: RustemonClient,
    type_name: String,
}

impl TypeCommand {
    pub async fn execute(client: RustemonClient, type_name: String) {
        TypeCommand { client, type_name }._execute().await
    }

    async fn _execute(&self) {
        let type_ = self.fetch_type().await;
        let type_relations = type_.damage_relations;
        let mut output = String::new();

        output.push_str(&format!("\n{}\n\n", &type_colours::fetch(&type_.name)));
        self.build_damage_details(&type_relations, &mut output);

        println!("{}", output);
    }

    async fn fetch_type(&self) -> Type {
        match type_::get_by_name(&self.type_name, &self.client).await {
            Ok(type_) => type_,
            Err(_) => matcher::try_suggest_name(&self.type_name, matcher::MatcherType::Type),
        }
    }

    fn build_damage_details(&self, type_relations: &TypeRelations, output: &mut String) {
        let headers = ("0x\n", "0.5x\n", "2x\n");

        // offensive type information
        output.push_str(&formatter::white("Offense\n"));
        self.build_types_output(
            &formatter::red(headers.0),
            &type_relations.no_damage_to,
            output,
        );
        self.build_types_output(
            &formatter::bright_red(headers.1),
            &type_relations.half_damage_to,
            output,
        );
        self.build_types_output(
            &formatter::green(headers.2),
            &type_relations.double_damage_to,
            output,
        );

        output.push('\n');

        // defensive type information
        output.push_str(&formatter::white("Defense\n"));
        self.build_types_output(
            &formatter::green(headers.0),
            &type_relations.no_damage_from,
            output,
        );
        self.build_types_output(
            &formatter::bright_green(headers.1),
            &type_relations.half_damage_from,
            output,
        );
        self.build_types_output(
            &formatter::red(headers.2),
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
