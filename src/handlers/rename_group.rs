use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "rename-group",
    arguments: "GROUP_ID NAME",
    description: "Rename a group containing/owned by this account. Core queues any required updates.",
    options: "",
    example: "rename-group 12 \"New name\"",
    summary: "Rename a group containing/owned by this account",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    chat::set_chat_name(context, id, &args.positional[1]).await?;
    crate::output::record(
        "renamed",
        vec![
            ("chat_id", id.to_u32().into()),
            ("name", args.positional[1].clone().into()),
        ],
        "Name updated locally; run listen for pending updates",
    );
    Ok(())
}
