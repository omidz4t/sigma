use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::context::Context;
use deltachat::{
    chat::{self, Chat, ChatId, ChatItem},
    message::Message,
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "messages",
    arguments: "CHAT_ID",
    description: "Print compact stored history without fetching or marking it read. Use read-messages for detailed text, attachments, and reactions.",
    options: "",
    example: "messages 12",
    summary: "Read locally stored messages",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = ChatId::new(args.positional[0].parse()?);
    Chat::load_from_db(context, id).await?;
    for item in chat::get_chat_msgs(context, id).await? {
        if let ChatItem::Message { msg_id } = item {
            let message = Message::load_from_db(context, msg_id).await?;
            if crate::output::is_json() {
                crate::shared::print_message(context, &message).await?;
            } else {
                crate::out!(
                    "{}\t{}\t{:?}\t{:?}",
                    msg_id.to_u32(),
                    message.get_from_id().to_u32(),
                    message.get_state(),
                    message.get_text()
                );
            }
        }
    }

    Ok(())
}
