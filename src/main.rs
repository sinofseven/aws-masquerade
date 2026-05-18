pub mod base;
pub mod cmd;
pub mod fs;
pub mod io;
pub mod models;
pub mod path;
pub mod totp;
pub mod variables;

use cmd::{AssumeArgs, ConfigureArgs, SourceArgs, TargetArgs};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Configure(ConfigureArgs),
    Source(SourceArgs),
    Target(TargetArgs),
    Assume(AssumeArgs),
}

impl Commands {
    fn run(self) -> Result<(), String> {
        match self {
            Commands::Configure(args) => args.run(),
            Commands::Source(args) => args.run(),
            Commands::Target(args) => args.run(),
            Commands::Assume(args) => args.run(),
        }
    }
}

fn main() -> Result<(), String> {
    Cli::parse().command.run()
}
