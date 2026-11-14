use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-send",
    arguments: "NAME DATA",
    description: "Send realtime UTF-8 data through the running app worker, without stopping its listener. --file reads binary bytes instead. Up to 128000 bytes. A realtime hook must be registered and a peer must have joined; packet delivery is best effort. Requests and received packets are journaled locally.",
    options: "  --file             Interpret DATA as a binary file path\n",
    example: "app-send demo '{\"type\":\"ping\"}' --json",
    summary: "Send live data through the app worker",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
