use crate::{Result, cli::Args, help::CommandHelp, shared::ensure_configured};
use deltachat::{chat, context::Context};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "create-group",
    arguments: "NAME",
    description: "Create an encrypted group containing this bot and print its local chat ID. Unlike broadcast channels, group members can participate in the conversation. Requires a configured account.",
    options: "",
    example: "create-group \"Team chat\"",
    summary: "Create an encrypted group; print its chat ID",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let id = chat::create_group(context, &args.positional[0]).await?;
    crate::output::record(
        "created",
        vec![("chat_id", id.to_u32().into())],
        &id.to_u32().to_string(),
    );
    Ok(())
}
