use crate::{
    Result,
    cli::Args,
    help::CommandHelp,
    shared::{attachment_message, ensure_configured, send_to_target},
};
use deltachat::{
    context::Context,
    message::{Message, Viewtype},
};
use std::path::Path;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-voice",
    arguments: "TARGET AUDIO_FILE",
    description: "Send a local recording as a voice message through core. TARGET is a contact ID, saved alias, or full contact invitation; use --chat for an existing chat/group/channel ID. The file is sent as supplied: no microphone recording, audio conversion or duration detection. Recipient playback depends on its supported codecs. Failed submissions may remain queued; use listen before retrying.",
    options: "  --chat             TARGET is an existing numeric chat/group/channel ID\n  --caption TEXT     Optional message caption\n  --duration MS      Recording duration in milliseconds (default: unknown)\n  --timeout SECONDS  Secure-join wait limit (default: 120)\n",
    example: "send-voice 10 ./recording.ogg --duration 3500",
    summary: "Send a recording as a voice message",
};

pub(crate) fn prepare(
    context: &Context,
    path: &Path,
    caption: Option<&str>,
    duration: Option<&str>,
) -> Result<Message> {
    if std::fs::metadata(path)?.len() == 0 {
        return Err("voice recording must not be empty".into());
    }
    let mut message = attachment_message(context, path, caption, Viewtype::Voice)?;
    if let Some(duration) = duration {
        let duration: i32 = duration
            .parse()
            .map_err(|_| "--duration must be milliseconds between 0 and 2147483647")?;
        if duration < 0 {
            return Err("--duration must not be negative".into());
        }
        message.set_duration(duration);
    }
    Ok(message)
}

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let message = prepare(
        context,
        Path::new(&args.positional[1]),
        args.option("--caption"),
        args.option("--duration"),
    )?;
    send_to_target(context, args, message).await
}
