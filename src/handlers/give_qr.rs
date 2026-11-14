use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{ensure_configured, save_join_qr};
use deltachat::context::Context;
use deltachat::securejoin;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "give-qr",
    arguments: "",
    description: "Save the bot's contact QR as SVG and print its join link. Output parent directory must exist; existing files are refused.",
    options: "  --output PATH  SVG destination (default: DATA_PATH/join.svg)\n",
    example: "give-qr --output ./bot-invite.svg",
    summary: "Save a QR SVG and print its join link",
};

pub(crate) async fn handle(context: &Context, args: &Args, path: &std::path::Path) -> Result<()> {
    ensure_configured(context).await?;
    let link = securejoin::get_securejoin_qr(context, None).await?;
    let output = args
        .option("--output")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| path.join("join.svg"));
    save_join_qr(&link, &output)?;
    eprintln!("QR code saved to {}", output.display());
    crate::output::record(
        "invitation",
        vec![
            ("link", link.clone().into()),
            ("qr_path", output.to_string_lossy().into_owned().into()),
        ],
        &link,
    );
    Ok(())
}
