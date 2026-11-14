use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype, contact::ContactId};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "leave-group",
    arguments: "GROUP_ID",
    description: "Leave this group through core. Pending departure messages require listen; this does not delete local history.",
    options: "",
    example: "leave-group 12",
    summary: "Leave this group through core",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    chat::remove_contact_from_chat(context, id, ContactId::SELF).await?;
    crate::output::record(
        "left_chat",
        vec![("chat_id", id.to_u32().into())],
        "Left chat locally; run listen for pending updates",
    );
    Ok(())
}
