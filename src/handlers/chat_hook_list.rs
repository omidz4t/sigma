use crate::{Result, account::Account, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "chat-hook-list",
    arguments: "CHAT_ID",
    description: "List saved command hooks for this chat.",
    options: "",
    example: "chat-hook-list 10",
    summary: "List saved command hooks for this chat",
};
pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    crate::chat_hooks::manage(account, args).await
}
