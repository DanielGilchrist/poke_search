use clap::Parser;
use poke_search::Cli;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn parse_args(args: Vec<&str>) -> Cli {
    let mut full_args = vec![PACKAGE_NAME];
    full_args.extend(args);
    Cli::parse_from(full_args)
}

#[macro_export]
macro_rules! assert_contains {
    ($output:expr, $substring:expr) => {
        assert!(
            $output.contains($substring),
            "Expected to find:\n  {}\n\nIn output:\n{}",
            $substring,
            $output
        )
    };
    ($output:expr, $substring:expr, $($arg:tt)+) => {
        assert!(
            $output.contains($substring),
            $($arg)+
        )
    };
}
