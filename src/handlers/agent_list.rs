use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-list",
    arguments: "",
    description: "List saved agents for the selected account, including stable ID, alias, provider, bound target and running/stopped state. Running agents also show PID and model. Works while a bridge holds the account lock. Stopped identities and bindings remain listed.",
    options: "",
    example: "agent-list --json",
    summary: "List saved agents, aliases, IDs and running state",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::agents::list(args).await
}
