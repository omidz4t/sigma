use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::help::{command_help, general_help};

pub(crate) const GENERAL_HEADER: &str = "sigma — stateful chatmail bot CLI\n\nUSAGE: sigma COMMAND [ARGS] [OPTIONS]\n       sigma COMMAND --help\n       sigma help [COMMAND]\n\nAI WEBXDC WORKFLOW:\n  send-webxdc TARGET app.xdc --app NAME -> app-start NAME\n  app-info NAME -> app-send NAME DATA / app-update NAME JSON\n  app-events NAME --after SEQ --json; app-stop NAME\n\nCOMMANDS:\n";

pub(crate) const GENERAL_OPTIONS: &str = "\nOPTIONS:
  --account NAME                Named account (default: saved selection)
  --data-path PATH              Explicit account directory
  --json                        Emit JSON Lines on stdout
  --app NAME                    Save a named app instance (send-webxdc)
  --name NAME                   Display name (init, account-create, agent-connect, add-contact)
  --profile-photo PATH          Local image file (init, account-create, agent-connect)
  --profile-photo-address PATH  Alias for --profile-photo
  --account-qr QR               Optional DCACCOUNT:/DCLOGIN: provisioning QR (init)
  --timeout SECONDS             Secure-join / receive / react wait (default: 120)
  --mark-read                   Mark displayed history as read (read-messages)
  --chat                        Treat a send command TARGET as a chat ID
  --as-text                     Send UTF-8 file contents as a text message (send-file)
  --output PATH                 QR SVG path (give-qr; default: DATA_PATH/join.svg)
  --caption TEXT                Optional caption (send-file, send-photo, send-webxdc)
  --chat-id ID                  Restrict search-messages to a chat
  --for-all                     Request remote deletion (delete-message)
  --on-message PATH             Executable message hook (listen)
  --auto-reply TEXT             Fixed automatic reply (listen)
  --hook-timeout SECONDS        Hook wait limit (listen; default: 30)
  --passphrase-file PATH        Backup/restore encryption password file
  --help, -h                    Show general or command-specific help
  --version, -V                 Show version

Options may appear before or after positional arguments. Use -- before literal
arguments beginning with '-'. Each data path holds one persistent bot account.";

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "help",
    arguments: "[COMMAND]",
    description: "Show general help or detailed help for a command. No account or network access is required.",
    options: "",
    example: "help send-photo",
    summary: "Show general or command-specific help",
};

pub(crate) async fn handle(args: &Args) -> Result<()> {
    if args.positional.len() > 1 {
        return Err("help expects at most one command name".into());
    }
    if let Some(command) = args.positional.first() {
        crate::out!("{}", command_help(command)?);
    } else {
        crate::out!("{}", general_help());
    }
    Ok(())
}
