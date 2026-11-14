use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{ensure_configured, outgoing_channel};
use deltachat::context::Context;
use deltachat::securejoin;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "channel-link",
    arguments: "CHANNEL_ID",
    description: "Print the subscription invitation for a broadcast channel owned by this account. Run listen while subscribers join.",
    options: "",
    example: "channel-link 12",
    summary: "Print a channel subscription invitation",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let channel = outgoing_channel(context, args.positional[0].parse()?).await?;
    let link = securejoin::get_securejoin_qr(context, Some(channel)).await?;
    crate::output::record(
        "invitation",
        vec![
            ("chat_id", channel.to_u32().into()),
            ("link", link.clone().into()),
        ],
        &link,
    );
    Ok(())
}
