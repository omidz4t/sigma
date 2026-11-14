use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-models",
    arguments: "ID_OR_ALIAS",
    description: "List model IDs and names from this native agent provider's local catalog, including its fetch timestamp. No hardcoded IDs or provider credentials are returned. The cached menu may be stale; initialize/refresh it using the provider CLI (Grok: grok models). Works online or stopped. Owner chat equivalent: /models.",
    options: "",
    example: "agent-models grok --account grok-new",
    summary: "List a native agent provider's cached models",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    super::agent_model::execute(args, true).await
}
