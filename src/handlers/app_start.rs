use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-start",
    arguments: "NAME",
    description: "Start a background app worker, or return the existing worker status. The worker keeps networking active, joins realtime only when a realtime-capable hook exists, accepts app-send/app-update commands and journals events. State is under the account apps/NAME directory. Only one worker per account can run.",
    options: "  --on-event PATH     Save an executable hook for realtime/status-update events\n  --hook-timeout SEC  Hook time limit (default: 30; requires --on-event)\n  --no-hook           Clear the saved hook configuration\n",
    example: "app-start demo --on-event ./hook.py --json",
    summary: "Start a persistent app worker",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
