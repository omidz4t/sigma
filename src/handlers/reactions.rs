use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::context::Context;
use deltachat::{
    message::{Message, MsgId},
    reaction,
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "reactions",
    arguments: "MESSAGE_ID",
    description: "Print locally stored reaction counts for a message. A blank line means no reactions. Run receive or listen to synchronize remote changes.",
    options: "",
    example: "reactions 42",
    summary: "Show stored reactions for a message",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = MsgId::new(args.positional[0].parse()?);
    Message::load_from_db(context, id).await?;
    let reactions = reaction::get_msg_reactions(context, id).await?;
    let counts = reactions
        .frequencies
        .iter()
        .map(|entry| {
            crate::output::object(vec![
                ("emoji", entry.reaction.as_str().into()),
                ("count", entry.count.into()),
                ("from_self", entry.is_from_self.into()),
            ])
        })
        .collect();
    crate::output::record(
        "reactions",
        vec![
            ("message_id", id.to_u32().into()),
            ("counts", crate::output::Value::Array(counts)),
        ],
        &reactions.to_string(),
    );

    Ok(())
}
