use clap::Parser;
use poke_search::{Cli, Client, run};

#[tokio::main]
async fn main() {
    let client = Client::try_build().unwrap_or_else(|e| {
        eprintln!("Failed to initialise client: {e}");
        std::process::exit(1);
    });

    let cli = Cli::parse();

    run(&client, cli).await.print();
}
