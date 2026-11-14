use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "account-default",
    arguments: "NAME",
    description: "Persist the account used when --account and --data-path are omitted. Use legacy for the original account. Existing workers retain their original account.",
    options: "",
    example: "account-default codex-bot",
    summary: "Select the default account",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::accounts::manage(args).await
}
