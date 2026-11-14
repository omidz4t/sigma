use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-hook-list",
    arguments: "NAME",
    description: "List all saved hooks, including executable paths, event types, payload filters and timeouts. Works while running or stopped.",
    options: "",
    example: "app-hook-list demo --json",
    summary: "List named app hooks",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
