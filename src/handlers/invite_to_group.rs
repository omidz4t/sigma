use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype, contact::ContactId};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "invite-to-group",
    arguments: "GROUP_ID CONTACT_ID",
    description: "Send a group invitation link to an existing contact. The person must join; run listen while they do.",
    options: "",
    example: "invite-to-group 12 10",
    summary: "Send a group invitation link to an existing contact",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    let contact = ContactId::new(args.positional[1].parse()?);
    deltachat::contact::Contact::get_by_id(context, contact).await?;
    let target = chat::ChatId::create_for_contact(context, contact).await?;
    let mut message = deltachat::message::Message::new_text(
        deltachat::securejoin::get_securejoin_qr(context, Some(id)).await?,
    );
    let sent = chat::send_msg_sync(context, target, &mut message).await?;
    crate::output::record(
        "invitation_sent",
        vec![
            ("message_id", sent.to_u32().into()),
            ("chat_id", id.to_u32().into()),
            ("contact_id", contact.to_u32().into()),
        ],
        "Group invitation submitted; recipient must join",
    );
    Ok(())
}
