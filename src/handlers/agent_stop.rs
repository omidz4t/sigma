use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-stop",
    arguments: "ID_OR_ALIAS",
    description: "Stop a running agent bridge by its saved numeric ID or alias through its private control socket, cancel the active invocation and wait for account cleanup. Works for background and foreground bridges. Uses the same --data-path as agent-connect.",
    options: "",
    example: "agent-stop grok",
    summary: "Stop an agent by ID or alias",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::agents::stop(args).await
}
