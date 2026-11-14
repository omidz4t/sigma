use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "search-messages",
    arguments: "QUERY",
    description: "Search stored messages with core. Results are detailed message records; fetching mail is a separate operation.",
    options: "  --chat-id ID  Restrict search to one chat\n",
    example: "search-messages \"meeting\" --chat-id 12",
    summary: "Search stored messages with core",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let chat = args
        .option("--chat-id")
        .map(str::parse::<u32>)
        .transpose()?
        .map(deltachat::chat::ChatId::new);
    if let Some(id) = chat {
        deltachat::chat::Chat::load_from_db(context, id).await?;
    }
    for id in context.search_msgs(chat, &args.positional[0]).await? {
        let message = deltachat::message::Message::load_from_db(context, id).await?;
        crate::shared::print_message(context, &message).await?;
    }
    Ok(())
}
