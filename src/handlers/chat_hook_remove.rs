use crate::{Result, account::Account, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "chat-hook-remove",
    arguments: "CHAT_ID HOOK_ID",
    description: "Remove a saved command hook from this chat.",
    options: "",
    example: "chat-hook-remove 10 ping",
    summary: "Remove a saved command hook from this chat",
};
pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    crate::chat_hooks::manage(account, args).await
}
