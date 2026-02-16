use clap::Parser;
use clipat::client;
use clipat::server;
use clipat::{Cli, Commands};

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Copy(args) => client::copy(args).unwrap(),
        Commands::Paste(args) => client::paste(args).unwrap(),
        Commands::Server(args) => server::run(args).unwrap(),
    }
}
