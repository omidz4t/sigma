use crate::{Result, account::Account, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "chat-hook-add",
    arguments: "CHAT_ID HOOK_ID EXECUTABLE",
    description: "Save a slash-command hook on a chat, independently of webxdc. Run listen to receive commands. Stop the account worker/listener before changing definitions.",
    options: "  --command /NAME     Required exact slash command\n  --hook-timeout SEC  Execution limit (default: 30)\n",
    example: "chat-hook-add 10 ping ./examples/command-hook.py --command /ping",
    summary: "Save a slash-command hook on a chat, independently of webxdc",
};
pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    crate::chat_hooks::manage(account, args).await
}
