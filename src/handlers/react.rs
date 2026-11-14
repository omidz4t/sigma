use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::wait_for_reaction;
use deltachat::context::Context;
use deltachat::{message::MsgId, reaction};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "react",
    arguments: "MESSAGE_ID [EMOJI]",
    description: "Set or replace this bot's reaction and wait for submission. Omit emoji to remove it. Use one token shorter than 30 UTF-8 bytes; core applies channel restrictions. Timeout may leave a queued reaction.",
    options: "  --timeout SECONDS  Wait limit in seconds (default: 120)\n",
    example: "react 42 \"👍\"",
    summary: "Set/replace a reaction; omit emoji to remove it",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    if !context.is_configured().await? {
        return Err("account is not configured; retry init".into());
    }
    let target = MsgId::new(args.positional[0].parse()?);
    let emoji = args.positional.get(1).map(String::as_str).unwrap_or("");
    let id = reaction::send_reaction(context, target, emoji).await?;
    context.start_io().await;
    let seconds = args.option("--timeout").unwrap_or("120").parse::<u64>()?;
    let result = wait_for_reaction(context, id, seconds).await;
    context.stop_io().await;
    result?;
    crate::output::record(
        "reaction_submitted",
        vec![
            ("message_id", target.to_u32().into()),
            ("reaction_message_id", id.to_u32().into()),
            ("reaction", emoji.into()),
        ],
        &format!("Reaction submitted for message {}", target.to_u32()),
    );

    Ok(())
}
