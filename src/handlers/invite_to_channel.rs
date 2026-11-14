use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{ensure_configured, outgoing_channel};
use deltachat::context::Context;
use deltachat::{
    chat::{self, ChatId},
    contact::{Contact, ContactId},
    message::Message,
    securejoin,
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "invite-to-channel",
    arguments: "CHANNEL_ID CONTACT_ID",
    description: "Send a channel invitation to an existing numeric contact ID. The person must accept it; this does not directly add a subscriber. Run listen while they join.",
    options: "",
    example: "invite-to-channel 12 10",
    summary: "Send a subscription invitation",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let channel = outgoing_channel(context, args.positional[0].parse()?).await?;
    let contact_id = ContactId::new(args.positional[1].parse()?);
    Contact::get_by_id(context, contact_id).await?;
    let link = securejoin::get_securejoin_qr(context, Some(channel)).await?;
    let direct_chat = ChatId::create_for_contact(context, contact_id).await?;
    let mut message = Message::new_text(link);
    let id = chat::send_msg_sync(context, direct_chat, &mut message).await?;
    crate::output::record(
        "invitation_sent",
        vec![
            ("message_id", id.to_u32().into()),
            ("chat_id", channel.to_u32().into()),
            ("contact_id", contact_id.to_u32().into()),
        ],
        &format!(
            "Sent invitation message {}; recipient must accept to subscribe",
            id.to_u32()
        ),
    );
    Ok(())
}
