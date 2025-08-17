use clap::Parser;
use poke_search::Cli;
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn parse_args(args: Vec<&str>) -> Cli {
    let mut full_args = vec![PACKAGE_NAME];
    full_args.extend(args);
    Cli::parse_from(full_args)
}
