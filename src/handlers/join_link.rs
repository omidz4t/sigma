use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "join-link",
    arguments: "",
    description: "Print this bot's contact invitation link. Alias for securejoin-id. Run listen while someone joins.",
    options: "",
    example: "join-link",
    summary: "Print this bot's join link (alias: securejoin-id)",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    super::securejoin_id::handle(context, args).await
}
