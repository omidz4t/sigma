use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "join-channel",
    arguments: "INVITATION",
    description: "Join a full channel invitation and print the resulting local chat ID after the handshake. Keep the inviter online.",
    options: "  --timeout SECONDS  Handshake wait (default: 120)\n",
    example: "join-channel \"https://i.delta.chat/#FULL_INVITATION\"",
    summary: "Join a full channel invitation and print the resulting local chat ID after the handshake",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let (_, id) = crate::shared::join_invitation(
        context,
        &args.positional[0],
        args.option("--timeout"),
        "channel",
    )
    .await?;
    crate::output::record(
        "joined",
        vec![("chat_id", id.to_u32().into())],
        &id.to_u32().to_string(),
    );
    Ok(())
}
