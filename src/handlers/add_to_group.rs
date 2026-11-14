use crate::{Result, cli::Args, help::CommandHelp, shared::ensure_configured};
use deltachat::{
    chat::{self, Chat, ChatId},
    constants::Chattype,
    contact::{Contact, ContactId},
    context::Context,
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "add-to-group",
    arguments: "GROUP_ID CONTACT_ID",
    description: "Add an existing contact to a group containing this bot. Encrypted groups require key contacts; use create-contact with the person's secure-join invitation first. Broadcast channels and direct chats are rejected. Membership updates may be queued; run listen to transmit them.",
    options: "",
    example: "add-to-group 12 10",
    summary: "Add an existing contact to a group",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let group = ChatId::new(args.positional[0].parse()?);
    let contact = ContactId::new(args.positional[1].parse()?);
    add_member(context, group, contact).await?;
    crate::output::record(
        "member_added",
        vec![
            ("chat_id", group.to_u32().into()),
            ("contact_id", contact.to_u32().into()),
        ],
        &format!(
            "Contact {} is a member of group {}; run listen to send pending membership updates",
            contact.to_u32(),
            group.to_u32()
        ),
    );
    Ok(())
}

pub(crate) async fn add_member(context: &Context, group: ChatId, contact: ContactId) -> Result<()> {
    if contact.is_special() {
        return Err("CONTACT_ID must identify a regular contact".into());
    }
    let chat = Chat::load_from_db(context, group).await?;
    if chat.get_type() != Chattype::Group {
        return Err("GROUP_ID must identify a group, not a channel or direct chat".into());
    }
    if !chat.is_self_in_chat(context).await? {
        return Err("this account is not a member of the group".into());
    }
    let person = Contact::get_by_id(context, contact).await?;
    if chat.is_encrypted(context).await? && !person.is_key_contact() {
        return Err("encrypted groups require a key contact; use create-contact with their secure-join invitation first".into());
    }
    chat::add_contact_to_chat(context, group, contact).await?;
    if !chat::is_contact_in_chat(context, group, contact).await? {
        return Err("core did not add the contact to the group".into());
    }
    Ok(())
}
