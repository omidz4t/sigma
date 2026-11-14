use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::chat;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "create-channel",
    arguments: "NAME",
    description: "Create a broadcast channel owned by this bot and print its chat ID. Subscribers receive read-only posts.",
    options: "",
    example: "create-channel \"Bot updates\"",
    summary: "Create a broadcast channel; print its chat ID",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    if !context.is_configured().await? {
        return Err("account is not configured; retry init".into());
    }
    let id = chat::create_broadcast(context, args.positional[0].clone()).await?;
    crate::output::record(
        "created",
        vec![("chat_id", id.to_u32().into())],
        &id.to_u32().to_string(),
    );

    Ok(())
}
