use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "account-list",
    arguments: "",
    description: "List managed named accounts and the original legacy account. The selected default has a star. Custom --data-path directories are not automatically registered.",
    options: "",
    example: "account-list --json",
    summary: "List accounts and identify the default",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::accounts::manage(args).await
}
