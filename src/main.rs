mod commands;
use commands::move_command::MoveCommand;

#[tokio::main]
async fn main() {
  let client = rustemon::client::RustemonClient::default();
  let type_name = Some(String::from("dark"));

  MoveCommand::execute(client, type_name).await;
}

