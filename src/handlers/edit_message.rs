use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "edit-message",
    arguments: "MESSAGE_ID TEXT",
    description: "Edit the text of your own stored message. Core enforces edit restrictions; changes are local and outgoing edit requests may be queued. Run listen afterward.",
    options: "",
    example: "edit-message 42 \"Corrected text\"",
    summary: "Edit the text of your own stored message",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    crate::shared::ensure_configured(context).await?;
    let id = deltachat::message::MsgId::new(args.positional[0].parse()?);
    deltachat::chat::send_edit_request(context, id, args.positional[1].clone()).await?;
    crate::output::record(
        "edited",
        vec![("message_id", id.to_u32().into())],
        "Message edited locally; run listen to send the update",
    );
    Ok(())
}
