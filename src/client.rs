use async_trait::async_trait;
use mockall::automock;
use rustemon::{
    client::RustemonClient,
    error::Error,
    model::{
        evolution::EvolutionChain,
        items::Item,
        moves::{Move, MoveLearnMethod},
        pokemon::{Ability, Pokemon, PokemonSpecies, Type},
    },
};

#[automock]
#[async_trait]
pub trait ClientImplementation {
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, Error>;
    async fn fetch_item(&self, item_name: &str) -> Result<Item, Error>;
    async fn fetch_move(&self, move_name: &str) -> Result<Move, Error>;
    async fn fetch_move_learn_method(
        &self,
        move_learn_method_name: &str,
    ) -> Result<MoveLearnMethod, Error>;
    async fn fetch_pokemon(&self, pokemon_name: &str) -> Result<Pokemon, Error>;
    async fn fetch_pokemon_species(&self, species_name: &str) -> Result<PokemonSpecies, Error>;
    async fn fetch_type(&self, type_name: &str) -> Result<Type, Error>;

    async fn fetch_evolution_chain_from_url(
        &self,
        evolution_chain: &str,
    ) -> Result<EvolutionChain, Error>;
}

#[derive(Default)]
pub struct Client(RustemonClient);

impl Client {
    fn extract_id_from_url(&self, url: &str) -> Result<i64, Error> {
        let id_str = url
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .ok_or_else(|| Error::UrlParse(url.to_owned()))?;

        id_str.parse::<i64>().map_err(|e| {
            let error_message = format!("{:?}, url: \"{}\"", e, url.to_owned());
            Error::UrlParse(error_message)
        })
    }
}

#[async_trait]
impl ClientImplementation for Client {
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, Error> {
        rustemon::pokemon::ability::get_by_name(ability_name, &self.0).await
    }

    async fn fetch_item(&self, item_name: &str) -> Result<Item, Error> {
        rustemon::items::item::get_by_name(item_name, &self.0).await
    }

    async fn fetch_move(&self, move_name: &str) -> Result<Move, Error> {
        rustemon::moves::move_::get_by_name(move_name, &self.0).await
    }

    async fn fetch_move_learn_method(
        &self,
        move_learn_method_name: &str,
    ) -> Result<MoveLearnMethod, Error> {
        rustemon::moves::move_learn_method::get_by_name(move_learn_method_name, &self.0).await
    }

    async fn fetch_pokemon(&self, pokemon_name: &str) -> Result<Pokemon, Error> {
        rustemon::pokemon::pokemon::get_by_name(pokemon_name, &self.0).await
    }

    async fn fetch_pokemon_species(&self, species_name: &str) -> Result<PokemonSpecies, Error> {
        rustemon::pokemon::pokemon_species::get_by_name(species_name, &self.0).await
    }

    async fn fetch_type(&self, type_name: &str) -> Result<Type, Error> {
        rustemon::pokemon::type_::get_by_name(type_name, &self.0).await
    }

    async fn fetch_evolution_chain_from_url(
        &self,
        evolution_chain_url: &str,
    ) -> Result<EvolutionChain, Error> {
        let id = self.extract_id_from_url(evolution_chain_url)?;
        rustemon::evolution::evolution_chain::get_by_id(id, &self.0).await
    }
}
