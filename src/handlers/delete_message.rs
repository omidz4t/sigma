use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "delete-message",
    arguments: "MESSAGE_ID",
    description: "Delete a stored message through core, including synchronized deletion on your devices/server. Use --for-all to request deletion for recipients where core allows it. Updates can be queued.",
    options: "  --for-all  Request deletion for all chat members (own encrypted messages only)\n",
    example: "delete-message 42",
    summary: "Delete a stored message through core, including synchronized deletion on your devices/server",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = deltachat::message::MsgId::new(args.positional[0].parse()?);
    deltachat::message::Message::load_from_db(context, id).await?;
    deltachat::message::delete_msgs_ext(context, &[id], args.option("--for-all").is_some()).await?;
    crate::output::record(
        "deleted",
        vec![("message_id", id.to_u32().into())],
        "Message deleted locally; run listen to transmit pending deletion requests",
    );
    Ok(())
}
