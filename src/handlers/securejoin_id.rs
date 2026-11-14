use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::ensure_configured;
use deltachat::context::Context;
use deltachat::securejoin;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "securejoin-id",
    arguments: "",
    description: "Print this bot's contact invitation link. Run listen while someone joins.",
    options: "",
    example: "securejoin-id",
    summary: "Print this bot's secure-join invitation",
};

pub(crate) async fn handle(context: &Context, _args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let link = securejoin::get_securejoin_qr(context, None).await?;
    crate::output::record("invitation", vec![("link", link.clone().into())], &link);
    Ok(())
}
