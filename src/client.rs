use std::path::PathBuf;

use async_trait::async_trait;
use mockall::automock;
use rustemon::{
    client::{CACacheManager, RustemonClient, RustemonClientBuilder},
    model::{
        evolution::EvolutionChain,
        items::Item,
        moves::{Move, MoveLearnMethod},
        pokemon::{Ability, Pokemon, PokemonSpecies, Type},
    },
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Rustemon(#[from] rustemon::error::Error),
    #[error("Failed to create cache directory: {0}")]
    CacheSetup(#[from] std::io::Error),
    #[error("Missing environment variable: {0}")]
    MissingEnv(#[from] std::env::VarError),
}

#[automock]
#[async_trait]
pub trait ClientImplementation {
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, rustemon::error::Error>;
    async fn fetch_item(&self, item_name: &str) -> Result<Item, rustemon::error::Error>;
    async fn fetch_move(&self, move_name: &str) -> Result<Move, rustemon::error::Error>;
    async fn fetch_move_learn_method(
        &self,
        move_learn_method_name: &str,
    ) -> Result<MoveLearnMethod, rustemon::error::Error>;
    async fn fetch_pokemon(&self, pokemon_name: &str) -> Result<Pokemon, rustemon::error::Error>;
    async fn fetch_pokemon_species(
        &self,
        species_name: &str,
    ) -> Result<PokemonSpecies, rustemon::error::Error>;
    async fn fetch_type(&self, type_name: &str) -> Result<Type, rustemon::error::Error>;

    async fn fetch_evolution_chain_from_url(
        &self,
        evolution_chain: &str,
    ) -> Result<EvolutionChain, rustemon::error::Error>;
}

#[derive(Default)]
pub struct Client(RustemonClient);

impl Client {
    pub fn try_build() -> Result<Self, ClientError> {
        let cache_dir = Self::get_cache_dir()?;
        let cache_manager = CACacheManager::new(cache_dir, false);
        let client = RustemonClientBuilder::default()
            .with_manager(cache_manager)
            .try_build()?;

        Ok(Client(client))
    }

    fn get_cache_dir() -> Result<PathBuf, std::env::VarError> {
        let home_dir = std::env::var("HOME")?;
        Ok(PathBuf::from(home_dir).join(".cache").join("poke_search"))
    }

    fn extract_id_from_url(&self, url: &str) -> Result<i64, rustemon::error::Error> {
        let id_str = url
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .ok_or_else(|| rustemon::error::Error::UrlParse(url.to_owned()))?;

        id_str.parse::<i64>().map_err(|e| {
            let error_message = format!("{:?}, url: \"{}\"", e, url.to_owned());
            rustemon::error::Error::UrlParse(error_message)
        })
    }
}

#[async_trait]
impl ClientImplementation for Client {
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, rustemon::error::Error> {
        rustemon::pokemon::ability::get_by_name(ability_name, &self.0).await
    }

    async fn fetch_item(&self, item_name: &str) -> Result<Item, rustemon::error::Error> {
        rustemon::items::item::get_by_name(item_name, &self.0).await
    }

    async fn fetch_move(&self, move_name: &str) -> Result<Move, rustemon::error::Error> {
        rustemon::moves::move_::get_by_name(move_name, &self.0).await
    }

    async fn fetch_move_learn_method(
        &self,
        move_learn_method_name: &str,
    ) -> Result<MoveLearnMethod, rustemon::error::Error> {
        rustemon::moves::move_learn_method::get_by_name(move_learn_method_name, &self.0).await
    }

    async fn fetch_pokemon(&self, pokemon_name: &str) -> Result<Pokemon, rustemon::error::Error> {
        rustemon::pokemon::pokemon::get_by_name(pokemon_name, &self.0).await
    }

    async fn fetch_pokemon_species(
        &self,
        species_name: &str,
    ) -> Result<PokemonSpecies, rustemon::error::Error> {
        rustemon::pokemon::pokemon_species::get_by_name(species_name, &self.0).await
    }

    async fn fetch_type(&self, type_name: &str) -> Result<Type, rustemon::error::Error> {
        rustemon::pokemon::type_::get_by_name(type_name, &self.0).await
    }

    async fn fetch_evolution_chain_from_url(
        &self,
        evolution_chain_url: &str,
    ) -> Result<EvolutionChain, rustemon::error::Error> {
        let id = self.extract_id_from_url(evolution_chain_url)?;
        rustemon::evolution::evolution_chain::get_by_id(id, &self.0).await
    }
}
