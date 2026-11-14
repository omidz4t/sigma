use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-stop",
    arguments: "NAME",
    description: "Ask the app worker to leave realtime and stop networking. Saved app identity, update cursor and captured events remain available for restarting.",
    options: "",
    example: "app-stop demo --json",
    summary: "Stop an app worker without losing state",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
