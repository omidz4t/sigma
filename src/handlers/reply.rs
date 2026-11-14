use crate::{Result, cli::Args, help::CommandHelp, shared::ensure_configured};
use deltachat::{
    chat::{self, Chat, ChatId},
    context::Context,
    message::{Message, MsgId},
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "reply",
    arguments: "MESSAGE_ID TEXT",
    description: "Reply to a stored message in the same chat using core's quoted-reply API. Quote the text in your shell. Core enforces chat permissions and encryption. Failed submissions may remain queued; use listen before retrying.",
    options: "",
    example: "reply 42 \"Thanks for your message!\"",
    summary: "Send a quoted reply to a message in its chat",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let (chat_id, mut reply) = prepare(
        context,
        MsgId::new(args.positional[0].parse()?),
        &args.positional[1],
    )
    .await?;
    if !Chat::load_from_db(context, chat_id)
        .await?
        .can_send(context)
        .await?
    {
        return Err("cannot reply in this chat; it may be read-only or require acceptance".into());
    }
    let id = chat::send_msg_sync(context, chat_id, &mut reply).await?;
    crate::output::record(
        "sent",
        vec![
            ("message_id", id.to_u32().into()),
            ("chat_id", chat_id.to_u32().into()),
            ("parent_id", args.positional[0].parse::<u32>()?.into()),
        ],
        &format!(
            "Sent reply {} to message {}",
            id.to_u32(),
            args.positional[0]
        ),
    );
    Ok(())
}

pub(crate) async fn prepare(
    context: &Context,
    target: MsgId,
    text: &str,
) -> Result<(ChatId, Message)> {
    if text.trim().is_empty() {
        return Err("message cannot be empty".into());
    }
    let original = Message::load_from_db(context, target).await?;
    let mut reply = Message::new_text(text.to_owned());
    reply.set_quote(context, Some(&original)).await?;
    Ok((original.get_chat_id(), reply))
}
