use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "download-file",
    arguments: "MESSAGE_ID PATH",
    description: "Download a partial message if necessary, then export its attachment to a new path. Refuses to overwrite existing files.",
    options: "  --timeout SECONDS  Download wait (default: 120)\n",
    example: "download-file 42 ./attachment.pdf",
    summary: "Download a partial message if necessary, then export its attachment to a new path",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = deltachat::message::MsgId::new(args.positional[0].parse()?);
    let path = std::path::Path::new(&args.positional[1]);
    if path.try_exists()? {
        return Err("output already exists".into());
    }
    let message = crate::shared::download_message(context, id, args.option("--timeout")).await?;
    message.save_file(context, path).await?;
    crate::output::record(
        "file_downloaded",
        vec![
            ("message_id", id.to_u32().into()),
            ("path", args.positional[1].clone().into()),
        ],
        &args.positional[1],
    );
    Ok(())
}
