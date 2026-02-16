use clap::{Parser, Subcommand};
use client::{CopyCli, PasteCli};
use server::ServerCli;

pub mod client;
mod clipboard;
mod protocol;
pub mod server;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Copy(CopyCli),
    Paste(PasteCli),
    Server(ServerCli),
}
