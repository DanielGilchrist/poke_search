use async_trait::async_trait;
use mockall::automock;
use rustemon::{
    client::RustemonClient,
    error::Error,
    model::{
        moves::{Move, MoveLearnMethod},
        pokemon::{Ability, Pokemon, Type, PokemonSpecies},
    },
};

#[automock]
#[async_trait]
pub trait ClientImplementation {
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, Error>;
    async fn fetch_move(&self, move_name: &str) -> Result<Move, Error>;
    async fn fetch_move_learn_method(
        &self,
        move_learn_method_name: &str,
    ) -> Result<MoveLearnMethod, Error>;
    async fn fetch_pokemon(&self, pokemon_name: &str) -> Result<Pokemon, Error>;
    async fn fetch_pokemon_species(&self, species_name: &str) -> Result<PokemonSpecies, Error>;
    async fn fetch_type(&self, type_name: &str) -> Result<Type, Error>;
    fn client(&self) -> &RustemonClient;
  }

#[derive(Default)]
pub struct Client(RustemonClient);

#[async_trait]
impl ClientImplementation for Client {
  fn client(&self) -> &RustemonClient {
    &self.0
  }
    async fn fetch_ability(&self, ability_name: &str) -> Result<Ability, Error> {
        rustemon::pokemon::ability::get_by_name(ability_name, &self.0).await
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
}
