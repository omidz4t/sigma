use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-hook-add",
    arguments: "NAME HOOK_ID EXECUTABLE",
    description: "Register an additional named executable hook. Works while the app worker is running or stopped. Every matching hook runs independently; duplicate IDs are refused. Realtime and update hooks select app payloads; message/command hooks apply to the app chat. Only a realtime hook (or legacy all hook) enables joining realtime. Existing events are not retroactively replayed.",
    options: "  --type TYPE         Required: realtime, update, message or command\n  --event EVENT       Legacy alternative: canonical event name or all\n  --command /NAME     Only match this slash command (command hooks)\n  --match JSON        Recursive JSON object subset to match against payloads\n  --hook-timeout SEC  Per-hook timeout (default: 30)\n",
    example: "app-hook-add demo buttons ./hook.py --type realtime --match '{\"type\":\"button_clicked\"}'",
    summary: "Add a named hook with event/payload filters",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
