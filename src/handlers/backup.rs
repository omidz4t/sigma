use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "backup",
    arguments: "DIRECTORY",
    description: "Export a core backup containing keys, credentials, chats, messages and blobs. Keep it private. A passphrase protects the database only; attachment blobs in the TAR remain unencrypted. Without a passphrase the database is also unencrypted.",
    options: "  --passphrase-file PATH  Read database encryption passphrase from a UTF-8 file\n",
    example: "backup ./backups --passphrase-file ./backup-password",
    summary: "Export a core backup containing keys, credentials, chats, messages and blobs",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let path = std::path::Path::new(&args.positional[0]);
    crate::shared::private_directory(path)?;
    let events = context.get_event_emitter();
    deltachat::imex::imex(
        context,
        deltachat::imex::ImexMode::ExportBackup,
        path,
        crate::shared::backup_passphrase(args)?,
    )
    .await?;
    let mut written = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let deltachat::EventType::ImexFileWritten(path) = event.typ {
            written.push(crate::output::Value::from(
                path.to_string_lossy().into_owned(),
            ));
        }
    }
    crate::output::record(
        "backup",
        vec![("files", crate::output::Value::Array(written))],
        &format!("Backup exported to {}", path.display()),
    );
    Ok(())
}
