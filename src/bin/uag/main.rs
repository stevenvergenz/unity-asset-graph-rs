mod broken;
mod build;
mod info;
mod outside;
mod unused;

use crate::{broken::BrokenArgs, build::BuildArgs, info::InfoArgs, outside::OutsideArgs, unused::UnusedArgs};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,

    /// Path to the database file
    #[arg(long, short, default_value = "db.bin")]
    pub db_path: PathBuf,
}

#[derive(Subcommand)]
pub enum CliCommand {
    Build(BuildArgs),
    Info(InfoArgs),
    Unused(UnusedArgs),
    Broken(BrokenArgs),
    Outside(OutsideArgs),
}

impl CliArgs {
    pub fn run(&self) {
        match &self.command {
            CliCommand::Build(args) => args.run(self),
            CliCommand::Info(args) => args.run(self),
            CliCommand::Unused(args) => args.run(self),
            CliCommand::Broken(args) => args.run(self),
            CliCommand::Outside(args) => args.run(self),
        };
    }
}

fn main() {
    CliArgs::parse().run();
}
