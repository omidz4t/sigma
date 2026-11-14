macro_rules! out {
    ($($arg:tt)*) => { $crate::output::text(format!($($arg)*)) };
}
pub(crate) use out;
mod account;
mod accounts;
mod agent_actions;
mod agent_calls;
mod agent_guard;
mod agent_isolation;
mod agent_master;
mod agent_models;
mod agent_outbound;
mod agent_registry;
mod agent_sessions;
mod agent_voice;
mod agents;
mod aliases;
mod app_hooks;
mod apps;
mod automation;
mod call_browser;
mod call_history;
mod call_recording;
mod chat_hooks;
mod cli;
mod handlers;
mod help;
mod output;
mod realtime;
mod shared;
#[cfg(test)]
mod tests;

use cli::Args;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        output::error(&error.to_string());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let raw = std::env::args().skip(1).collect::<Vec<_>>();
    let json_requested = raw
        .iter()
        .take_while(|arg| arg.as_str() != "--")
        .any(|arg| arg == "--json");
    output::configure(
        json_requested,
        raw.iter()
            .find(|arg| !arg.starts_with('-'))
            .map(String::as_str)
            .unwrap_or("help"),
    );
    let mut args = Args::parse(raw)?;
    output::configure(json_requested, &args.command);
    match args.command.as_str() {
        "--help" | "-h" | "help" => return handlers::help::handle(&args).await,
        "--version" | "-V" => return handlers::version::handle().await,
        _ => args.validate()?,
    }
    if args.command == "agent-connect" {
        agents::prepare(&mut args).await?;
    }
    if handlers::dispatch_client(&args).await? {
        return Ok(());
    }
    let account = account::open(&args).await?;
    handlers::dispatch(&account, &args).await
}
