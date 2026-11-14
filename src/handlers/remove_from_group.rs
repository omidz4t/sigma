use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype, contact::ContactId};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "remove-from-group",
    arguments: "GROUP_ID CONTACT_ID",
    description: "Remove a regular contact from the group. Membership updates may be queued; run listen to transmit them.",
    options: "",
    example: "remove-from-group 12 10",
    summary: "Remove a regular contact from the group",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    let contact = ContactId::new(args.positional[1].parse()?);
    if contact.is_special() {
        return Err("use leave-group/leave-channel to remove yourself".into());
    }
    chat::remove_contact_from_chat(context, id, contact).await?;
    if chat::is_contact_in_chat(context, id, contact).await? {
        return Err("core did not remove this member".into());
    }
    crate::output::record(
        "member_removed",
        vec![
            ("chat_id", id.to_u32().into()),
            ("contact_id", contact.to_u32().into()),
        ],
        "Member removed locally; run listen for pending updates",
    );
    Ok(())
}
