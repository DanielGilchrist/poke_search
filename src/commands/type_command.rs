use crate::formatter::{self};
use std::process::exit;

use rustemon::{
    client::RustemonClient,
    model::{
        pokemon::{Type, TypeRelations},
        resource::NamedApiResource,
    },
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

        output.push_str(&format!("{} Type\n\n", formatter::capitalise(&type_.name)));
        self.build_damage_details(&type_relations, &mut output);

        println!("{}", output);
    }

    async fn fetch_type(&self) -> Type {
        match type_::get_by_name(&self.type_name, &self.client).await {
            Ok(type_) => type_,
            Err(_) => {
                println!("Type \"{}\" doesn't exist", self.type_name);
                exit(1);
            }
        }
    }

    fn build_damage_details(&self, type_relations: &TypeRelations, output: &mut String) {
        let headers = ("No Damage\n", "Half Damage\n", "Double Damage\n");

        // offensive type information
        output.push_str("Offense\n");
        self.build_types_output(headers.0, &type_relations.no_damage_to, output);
        self.build_types_output(headers.1, &type_relations.half_damage_to, output);
        self.build_types_output(headers.2, &type_relations.double_damage_to, output);

        output.push('\n');

        // defensive type information
        output.push_str("Defense\n");
        self.build_types_output(headers.0, &type_relations.no_damage_from, output);
        self.build_types_output(headers.1, &type_relations.half_damage_from, output);
        self.build_types_output(headers.2, &type_relations.double_damage_from, output);
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
            .map(|type_resource| type_resource.name.as_str())
            .collect::<Vec<_>>();

        type_names.sort();

        output.push_str(&format!("  {}\n", type_names.join(" | ")));
    }
}
