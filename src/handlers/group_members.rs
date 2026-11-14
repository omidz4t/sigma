use crate::shared::{list_members, managed_chat};
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::constants::Chattype;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "group-members",
    arguments: "GROUP_ID",
    description: "List locally stored group members with contact IDs and names. Does not fetch mail.",
    options: "",
    example: "group-members 12",
    summary: "List locally stored group members with contact IDs and names",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    list_members(context, id).await?;
    Ok(())
}
