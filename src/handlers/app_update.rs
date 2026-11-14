use crate::{Result, cli::Args, help::CommandHelp};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-update",
    arguments: "NAME UPDATE_JSON",
    description: "Queue a durable webxdc status update through the running worker. It transmits it while continuing to receive realtime packets. Include info/notify for visible updates; omit them for silent state changes. This updates app state, not app code.",
    options: "",
    example: "app-update demo '{\"payload\":{\"text\":\"Hello\"}}' --json",
    summary: "Send app state through the app worker",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::apps::client_command(args).await?;
    Ok(())
}
