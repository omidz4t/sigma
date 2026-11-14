use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::print_message;
use deltachat::context::Context;
use deltachat::{
    chat::{self, Chat, ChatId, ChatItem},
    message::{self, Message},
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "read-messages",
    arguments: "CHAT_ID",
    description: "Print detailed stored history, including sender, text, attachments, and reactions. Does not fetch; run receive or listen to synchronize first.",
    options: "  --mark-read  Mark displayed messages read through core; receipts may be sent later\n",
    example: "read-messages 12 --mark-read",
    summary: "Read stored history; optionally --mark-read",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = ChatId::new(args.positional[0].parse()?);
    Chat::load_from_db(context, id).await?;
    let mut displayed = Vec::new();
    for item in chat::get_chat_msgs(context, id).await? {
        if let ChatItem::Message { msg_id } = item {
            let message = Message::load_from_db(context, msg_id).await?;
            print_message(context, &message).await?;
            displayed.push(msg_id);
        }
    }
    if args.option("--mark-read").is_some() {
        message::markseen_msgs(context, displayed).await?;
    }
    Ok(())
}
