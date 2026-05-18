mod list;
mod show;

#[derive(clap::Args)]
#[command(
    about = "Commands related to source configuration",
    arg_required_else_help = true
)]
pub struct SourceArgs {
    #[command(subcommand)]
    command: SourceCommand,
}

#[derive(clap::Subcommand)]
enum SourceCommand {
    /// list source name
    List,
    /// show detail of a source
    Show(show::ShowArgs),
}

impl SourceArgs {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            SourceCommand::List => list::run(),
            SourceCommand::Show(args) => show::run(args),
        }
    }
}
