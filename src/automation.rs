//! Explicitly enabled message hooks: no shell evaluation and bounded execution/output.
use crate::{Result, cli::Args, output, shared};
use deltachat::{chat, contact::ContactId, context::Context, message::Message};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) async fn run_hook(executable: &Path, input: &str, seconds: u64) -> Result<String> {
    run_process(executable, &[], input, seconds).await
}

pub(crate) async fn run_process(
    executable: &Path,
    arguments: &[String],
    input: &str,
    seconds: u64,
) -> Result<String> {
    run_process_env(executable, arguments, input, seconds, &[]).await
}

pub(crate) async fn run_process_env(
    executable: &Path,
    arguments: &[String],
    input: &str,
    seconds: u64,
    environment: &[(String, String)],
) -> Result<String> {
    let executable = std::fs::canonicalize(executable)?;
    let mut child = crate::agent_isolation::command(&executable, environment, None, None)?
        .args(arguments)
        .envs(environment.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let operation = async {
        let mut stdin = child.stdin.take().ok_or("hook stdin unavailable")?;
        stdin.write_all(input.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        drop(stdin);
        let stdout = child.stdout.take().ok_or("hook stdout unavailable")?;
        let mut bytes = Vec::new();
        stdout.take(65537).read_to_end(&mut bytes).await?;
        if bytes.len() > 65536 {
            return Err("hook output exceeds 64 KiB".into());
        }
        let status = child.wait().await?;
        if !status.success() {
            return Err(format!("message hook exited with {status}").into());
        }
        Ok(String::from_utf8(bytes)?)
    };
    tokio::time::timeout(Duration::from_secs(seconds), operation)
        .await
        .map_err(|_| "message hook timed out")?
}

pub(crate) async fn incoming(context: &Context, args: &Args, original: &Message) -> Result<()> {
    if original.get_from_id() == ContactId::SELF || original.is_info() {
        return Ok(());
    }
    let text = if let Some(executable) = args.option("--on-message") {
        let input = output::encode(&shared::message_value(context, original).await?);
        run_hook(
            Path::new(executable),
            &input,
            args.option("--hook-timeout").unwrap_or("30").parse()?,
        )
        .await?
    } else if let Some(text) = args.option("--auto-reply") {
        text.to_owned()
    } else {
        return Ok(());
    };
    if text.trim().is_empty() {
        return Ok(());
    }
    let chat = shared::sendable_chat(context, original.get_chat_id().to_u32()).await?;
    let mut reply = Message::new_text(text);
    reply.set_quote(context, Some(original)).await?;
    let id = chat::send_msg(context, chat, &mut reply).await?;
    output::record(
        "auto_reply_queued",
        vec![
            ("message_id", id.to_u32().into()),
            ("parent_id", original.get_id().to_u32().into()),
            ("chat_id", chat.to_u32().into()),
        ],
        &format!("Auto reply {} queued", id.to_u32()),
    );
    Ok(())
}
