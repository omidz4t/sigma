use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-events",
    arguments: "NAME",
    description: "Read the persisted event journal, including captured realtime packets, outgoing operations and durable status updates. Does not require a running worker. Each record has an increasing seq; save it and use --after to resume. Realtime packets sent while the worker was offline cannot be recovered.",
    options: "  --after SEQ        Only show events after this local journal sequence (default: 0)\n",
    example: "app-events demo --after 0 --json",
    summary: "Read saved app events after a cursor",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
