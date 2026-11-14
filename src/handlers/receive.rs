use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::print_message;
use deltachat::context::Context;
use deltachat::message::Message;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "receive",
    arguments: "",
    description: "Fetch mail once and print fresh/unread messages, including previously stored unread messages. Core excludes muted/blocked/request chats. Does not mark read or wait for outgoing mail.",
    options: "  --timeout SECONDS  Wait limit in seconds (default: 120)\n",
    example: "receive --timeout 60",
    summary: "Fetch mail once and print unread messages",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    if !context.is_configured().await? {
        return Err("account is not configured; retry init".into());
    }
    let seconds = args.option("--timeout").unwrap_or("120").parse::<u64>()?;
    tokio::select! {
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds), context.background_fetch()) => {
            result.map_err(|_| "receive timed out; stored messages remain available through read-messages")??;
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            return Err("receive interrupted; stored messages remain available through read-messages".into());
        }
    }
    let unread = context.get_fresh_msgs().await?;
    for id in &unread {
        let message = Message::load_from_db(context, *id).await?;
        print_message(context, &message).await?;
    }
    eprintln!("{} unread message(s)", unread.len());

    Ok(())
}
