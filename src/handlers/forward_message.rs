use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "forward-message",
    arguments: "MESSAGE_ID CHAT_ID",
    description: "Forward a stored message to an explicit destination chat ID, including groups. Core queues the forwarded message; run listen to deliver it.",
    options: "",
    example: "forward-message 42 12",
    summary: "Forward a stored message to an explicit destination chat ID, including groups",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    crate::shared::ensure_configured(context).await?;
    let source = deltachat::message::MsgId::new(args.positional[0].parse()?);
    deltachat::message::Message::load_from_db(context, source).await?;
    let target = crate::shared::sendable_chat(context, args.positional[1].parse()?).await?;
    deltachat::chat::forward_msgs(context, &[source], target).await?;
    crate::output::record(
        "forwarded",
        vec![
            ("source_message_id", source.to_u32().into()),
            ("chat_id", target.to_u32().into()),
        ],
        "Forward queued; run listen to deliver it",
    );
    Ok(())
}
