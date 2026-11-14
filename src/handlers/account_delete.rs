use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "account-delete",
    arguments: "NAME",
    description: "Permanently delete a named account’s local keys, credentials, messages, blobs, aliases and worker data. Refuses the current default, running accounts and symlinked directories. Select another default and stop workers first. Does not delete remote provider mailboxes.",
    options: "",
    example: "account-delete old-bot",
    summary: "Delete a stopped, nondefault local account",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::accounts::manage(args).await
}
