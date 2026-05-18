mod list;
mod show;

#[derive(clap::Args)]
#[command(
    about = "commands related to target configuration",
    arg_required_else_help = true
)]
pub struct TargetArgs {
    #[command(subcommand)]
    command: TargetCommand,
}

#[derive(clap::Subcommand)]
enum TargetCommand {
    /// list target name
    List,
    /// show detail of a target
    Show(show::ShowArgs),
}

impl TargetArgs {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            TargetCommand::List => list::run(),
            TargetCommand::Show(args) => show::run(args),
        }
    }
}
