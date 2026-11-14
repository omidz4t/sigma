use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "restore",
    arguments: "BACKUP_FILE",
    description: "Restore a core backup into a NEW/empty account directory. Existing account databases are refused. On failure inspect the destination and use another empty directory; the source backup is unchanged.",
    options: "  --passphrase-file PATH  Read decryption passphrase from a UTF-8 file\n",
    example: "restore ./backup.tar --data-path ./restored-bot",
    summary: "Restore a core backup into a NEW/empty account directory",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    deltachat::imex::imex(
        context,
        deltachat::imex::ImexMode::ImportBackup,
        std::path::Path::new(&args.positional[0]),
        crate::shared::backup_passphrase(args)?,
    )
    .await?;
    crate::output::record(
        "restored",
        vec![(
            "database",
            context.get_dbfile().to_string_lossy().into_owned().into(),
        )],
        "Backup restored",
    );
    Ok(())
}
