use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "export-file",
    arguments: "MESSAGE_ID PATH",
    description: "Export an already downloaded attachment to a new local path. Refuses to overwrite existing files. Use download-file for partial messages.",
    options: "",
    example: "export-file 42 ./attachment.pdf",
    summary: "Export an already downloaded attachment to a new local path",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let message = deltachat::message::Message::load_from_db(
        context,
        deltachat::message::MsgId::new(args.positional[0].parse()?),
    )
    .await?;
    message
        .save_file(context, std::path::Path::new(&args.positional[1]))
        .await?;
    crate::output::record(
        "file_exported",
        vec![("path", args.positional[1].clone().into())],
        &args.positional[1],
    );
    Ok(())
}
