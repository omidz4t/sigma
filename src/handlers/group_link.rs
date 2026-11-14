use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::constants::Chattype;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "group-link",
    arguments: "GROUP_ID",
    description: "Print a secure-join invitation for a group you belong to. Keep listen running while people join.",
    options: "",
    example: "group-link 12",
    summary: "Print a secure-join invitation for a group you belong to",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    let link = deltachat::securejoin::get_securejoin_qr(context, Some(id)).await?;
    crate::output::record(
        "invitation",
        vec![
            ("chat_id", id.to_u32().into()),
            ("link", link.clone().into()),
        ],
        &link,
    );
    Ok(())
}
