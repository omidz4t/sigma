use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-list",
    arguments: "",
    description: "List saved app names and their original core message/chat IDs. Works while a worker is running.",
    options: "",
    example: "app-list --json",
    summary: "List saved app instances",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
