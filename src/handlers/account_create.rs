use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "account-create",
    arguments: "NAME",
    description: "Create and provision a new named account. Existing names are refused; retry partial provisioning with init --account NAME. Does not change the default account.",
    options: "  --name TEXT          Display name (default: Sigma)\n  --profile-photo PATH  Local profile image\n  --account-qr QR       Optional provisioning QR\n",
    example: "account-create codex-bot --name Codex",
    summary: "Create a new named account",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::accounts::manage(args).await
}
