use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-info",
    arguments: "NAME",
    description: "Show the saved app identity and current worker status. realtime_joined means a peer joined this worker session; it is not a delivery guarantee. Works without opening the account database.",
    options: "",
    example: "app-info demo --json",
    summary: "Check a saved app and its connection",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
