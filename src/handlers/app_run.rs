use crate::{Result, account::Account, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-run",
    arguments: "NAME",
    description: "Run the named app worker in the foreground. Joins realtime only when a realtime-capable hook exists, keeps core networking running, handles app-send/app-update over private local IPC, and saves events. No peer wait timeout: the worker stays available while waiting. Ctrl-C stops it. Usually use app-start for background operation.",
    options: "  --on-event PATH     Save an executable hook for realtime/status-update events\n  --hook-timeout SEC  Hook time limit (default: 30; requires --on-event)\n  --no-hook           Clear the saved hook configuration\n",
    example: "app-run demo",
    summary: "Run the named app worker in the foreground",
};
pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    crate::apps::run(&account.context, &account.path, &args.positional[0], args).await
}
