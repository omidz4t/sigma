use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-hook-remove",
    arguments: "NAME HOOK_ID",
    description: "Remove one named hook while running or stopped. Cancels that hook’s queued/running work; other hooks are unaffected. Saved configuration remains removed after restart.",
    options: "",
    example: "app-hook-remove demo buttons",
    summary: "Remove one named app hook",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
