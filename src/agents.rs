//! Account-scoped agent workers controlled over private Unix sockets, never saved PIDs.
use crate::{Result, account, cli::Args, output};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::watch;

fn root(args: &Args) -> Result<PathBuf> {
    crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())
}
fn directory(root: &Path, agent: &str) -> Result<PathBuf> {
    let alias = crate::agent_registry::resolve_selector(root, agent)?;
    crate::agent_registry::directory(root, &alias)
}
fn private_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn private_log(path: &Path) -> Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}

pub(crate) struct Runtime {
    pub(crate) stop: watch::Receiver<bool>,
    task: tokio::task::JoinHandle<()>,
    socket: PathBuf,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_file(&self.socket);
    }
}

#[cfg(test)]
pub(crate) async fn register(root: &Path, args: &Args, chat_id: u32) -> Result<Runtime> {
    register_with_context(root, args, chat_id, None).await
}
pub(crate) async fn register_with_context(
    root: &Path,
    args: &Args,
    chat_id: u32,
    context: Option<deltachat::context::Context>,
) -> Result<Runtime> {
    #[cfg(unix)]
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let alias = crate::agent_registry::selected_alias(root, args)?;
        let saved = crate::agent_registry::load(root, &alias)?;
        let dir = directory(root, &alias)?;
        private_directory(&root.join("agents"))?;
        private_directory(&dir)?;
        let socket = dir.join("control.sock");
        // The caller holds the core account lock, so any old socket is stale.
        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }
        let listener = tokio::net::UnixListener::bind(&socket)?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
        let status = output::encode(&output::object(vec![
            ("agent", args.positional[0].as_str().into()),
            ("alias", alias.as_str().into()),
            (
                "id",
                saved
                    .as_ref()
                    .map(|a| a.id.into())
                    .unwrap_or(output::Value::Null),
            ),
            (
                "contact_id",
                saved
                    .as_ref()
                    .map(|a| a.contact_id.into())
                    .unwrap_or(output::Value::Null),
            ),
            ("running", true.into()),
            (
                "record_calls",
                args.option("--record-calls").is_some().into(),
            ),
            (
                "outbound_calls",
                args.option("--allow-outbound-calls").is_some().into(),
            ),
            ("data_path", root.to_string_lossy().into_owned().into()),
            ("pid", std::process::id().into()),
            (
                "target",
                saved
                    .as_ref()
                    .map(|a| a.target.as_str())
                    .or_else(|| args.option("--to"))
                    .unwrap_or("")
                    .into(),
            ),
            ("chat_id", chat_id.into()),
            (
                "model",
                args.option("--model")
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
            (
                "system_prompt",
                (args.option("--system-prompt").is_some()
                    || args.option("--system-prompt-file").is_some())
                .into(),
            ),
        ]));
        let (tx, stop) = watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let operation = async {
                    let mut request = Vec::new();
                    (&mut stream).take(512).read_to_end(&mut request).await?;
                    match request.as_slice() {
                        b"status" => stream.write_all(status.as_bytes()).await?,
                        b"stop" => {
                            stream.write_all(b"stopping").await?;
                            let _ = tx.send(true);
                        }
                        _ if request.starts_with(b"calls ") => {
                            let result: Result<String> = async {
                                let context =
                                    context.as_ref().ok_or("call management unavailable")?;
                                let text = std::str::from_utf8(&request)?;
                                crate::call_history::control(
                                    context,
                                    &dir,
                                    &text[6..].split_whitespace().collect::<Vec<_>>(),
                                )
                                .await
                            }
                            .await;
                            let raw = result.unwrap_or_else(|e| {
                                output::encode(&output::object(vec![
                                    ("ok", false.into()),
                                    ("error", e.to_string().into()),
                                ]))
                            });
                            stream.write_all(raw.as_bytes()).await?;
                        }
                        _ if request.starts_with(b"master ") => {
                            let result: Result<String> = async {
                                let context =
                                    context.as_ref().ok_or("master management unavailable")?;
                                let saved = saved.as_ref().ok_or("agent missing")?;
                                let text = std::str::from_utf8(&request)?;
                                crate::agent_master::control(
                                    context,
                                    &dir,
                                    saved,
                                    &text[7..].split_whitespace().collect::<Vec<_>>(),
                                )
                                .await
                            }
                            .await;
                            let raw = result.unwrap_or_else(|e| {
                                output::encode(&output::object(vec![
                                    ("ok", false.into()),
                                    ("error", e.to_string().into()),
                                ]))
                            });
                            stream.write_all(raw.as_bytes()).await?;
                        }
                        _ if request.starts_with(b"model ") => {
                            let result: Result<String> = (|| {
                                let saved = saved.as_ref().ok_or("agent identity missing")?;
                                let text = std::str::from_utf8(&request)?;
                                crate::agent_models::control(
                                    &dir,
                                    &saved.provider,
                                    !dir.join("custom-adapter").exists(),
                                    &text[6..].split_whitespace().collect::<Vec<_>>(),
                                )
                            })();
                            let response = result.unwrap_or_else(|e| {
                                output::encode(&output::object(vec![
                                    ("ok", false.into()),
                                    ("error", e.to_string().into()),
                                ]))
                            });
                            stream.write_all(response.as_bytes()).await?;
                        }
                        _ if request.starts_with(b"permissions ") => {
                            let result: Result<String> = async {
                                let context = context
                                    .as_ref()
                                    .ok_or("permission management unavailable")?;
                                let saved = saved.as_ref().ok_or("agent identity missing")?;
                                let master =
                                    crate::agent_master::current(context, &dir, saved).await?;
                                let owner = master.contact;
                                let chat = master.chat;
                                let text = std::str::from_utf8(&request)?;
                                crate::agent_guard::control(
                                    context,
                                    &dir,
                                    owner,
                                    chat,
                                    &text[12..].split_whitespace().collect::<Vec<_>>(),
                                )
                                .await
                            }
                            .await;
                            let response = result.unwrap_or_else(|e| {
                                output::encode(&output::object(vec![
                                    ("ok", false.into()),
                                    ("error", e.to_string().into()),
                                ]))
                            });
                            stream.write_all(response.as_bytes()).await?;
                        }
                        _ => stream.write_all(b"invalid command").await?,
                    }
                    std::io::Result::Ok(())
                };
                let _ = tokio::time::timeout(Duration::from_secs(30), operation).await;
            }
        });
        Ok(Runtime { stop, task, socket })
    }
    #[cfg(not(unix))]
    {
        let _ = (root, args, chat_id);
        Err("agent workers currently require Unix (Linux/macOS)".into())
    }
}

pub(crate) async fn request(root: &Path, agent: &str, command: &str) -> Result<String> {
    #[cfg(unix)]
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let socket = directory(root, agent)?.join("control.sock");
        tokio::time::timeout(
            Duration::from_secs(if command.starts_with("calls ") { 30 } else { 3 }),
            async {
                let mut stream = tokio::net::UnixStream::connect(socket).await?;
                stream.write_all(command.as_bytes()).await?;
                stream.shutdown().await?;
                let mut response = String::new();
                stream.take(262145).read_to_string(&mut response).await?;
                if response.len() > 262144 {
                    return Err("agent control response exceeds 256 KiB".into());
                }
                Ok(response)
            },
        )
        .await
        .map_err(|_| "agent control request timed out")?
    }
    #[cfg(not(unix))]
    {
        let _ = (root, agent, command);
        Err("agent workers currently require Unix (Linux/macOS)".into())
    }
}

fn print_status(raw: &str) -> Result<()> {
    let value = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    let v = &value.payload;
    let agent = v
        .get("agent")
        .and_then(|v| v.as_str())
        .ok_or("invalid agent status")?;
    let target = v
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or("invalid agent target")?;
    let pid = v.get("pid").and_then(|v| v.as_i64());
    let id = v.get("id").and_then(|v| v.as_i64());
    let alias = v.get("alias").and_then(|v| v.as_str()).unwrap_or(agent);
    let running = v.get("running").and_then(|v| v.as_bool()).unwrap_or(true);
    let initial_model = v.get("model").and_then(|v| v.as_str());
    let saved_model = if let Some(path) = v.get("data_path").and_then(|v| v.as_str()) {
        crate::agent_models::selected(
            &crate::agent_registry::directory(Path::new(path), alias)?,
            initial_model,
        )?
    } else {
        initial_model.map(str::to_owned)
    };
    let model = saved_model.as_deref();
    let session = if let Some(path) = v.get("data_path").and_then(|v| v.as_str()) {
        crate::agent_sessions::load(&crate::agent_registry::directory(Path::new(path), alias)?)?
    } else {
        None
    };
    let session_id = session.as_ref().and_then(|s| s.id.as_deref());
    let master = if let Some(path) = v.get("data_path").and_then(|v| v.as_str()) {
        crate::agent_master::stored(&crate::agent_registry::directory(Path::new(path), alias)?)?
    } else {
        None
    };

    output::record(
        "agent",
        vec![
            (
                "bot_master_contact_id",
                master
                    .as_ref()
                    .map(|m| m.contact.to_u32().into())
                    .or_else(|| {
                        v.get("contact_id")
                            .and_then(|v| v.as_i64())
                            .map(output::Value::from)
                    })
                    .unwrap_or(output::Value::Null),
            ),
            ("agent", agent.into()),
            ("alias", alias.into()),
            (
                "session_id",
                session_id
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
            (
                "id",
                id.map(output::Value::from).unwrap_or(output::Value::Null),
            ),
            (
                "contact_id",
                v.get("contact_id")
                    .and_then(|v| v.as_i64())
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
            (
                "data_path",
                v.get("data_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
            ),
            ("target", target.into()),
            (
                "pid",
                pid.map(output::Value::from).unwrap_or(output::Value::Null),
            ),
            ("running", running.into()),
            (
                "record_calls",
                (running
                    && v.get("record_calls")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false))
                .into(),
            ),
            (
                "outbound_calls",
                (running
                    && v.get("outbound_calls")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false))
                .into(),
            ),
            (
                "model",
                model
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
            (
                "system_prompt",
                if running {
                    v.get("system_prompt")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        .into()
                } else {
                    output::Value::Null
                },
            ),
            (
                "chat_id",
                v.get("chat_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("invalid agent chat")?
                    .into(),
            ),
        ],
        &format!(
            "{}  {alias} ({agent})  {}  to {target}{}",
            id.map(|v| v.to_string())
                .unwrap_or_else(|| "(legacy)".into()),
            if running {
                format!("running PID {}", pid.unwrap_or(0))
            } else {
                "stopped".into()
            },
            if running {
                format!("  model {}", model.unwrap_or("(default)"))
            } else {
                String::new()
            }
        ),
    );
    Ok(())
}

pub(crate) async fn list(args: &Args) -> Result<()> {
    let root = root(args)?;
    let dir = root.join("agents");
    let mut count = 0;
    if dir.is_dir() {
        let mut entries = std::fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(alias) = entry.file_name().to_str()
                && crate::aliases::validate(alias).is_ok()
            {
                if let Ok(status) = request(&root, alias, "status").await {
                    print_status(&status)?;
                    count += 1;
                } else if let Some(saved) = crate::agent_registry::load(&root, alias)? {
                    print_status(&saved.stopped_status(&root))?;
                    count += 1;
                }
            }
        }
    }
    if count == 0 {
        output::record(
            "agent_list",
            vec![("count", 0_i64.into())],
            "No saved or running agents for this account.",
        );
    }
    Ok(())
}

pub(crate) async fn stop(args: &Args) -> Result<()> {
    let root = root(args)?;
    let agent = &args.positional[0];
    let response = request(&root, agent, "stop")
        .await
        .map_err(|error| format!("agent {agent} is not reachable: {error}"))?;
    if response != "stopping" {
        return Err("invalid agent stop response".into());
    }
    let socket = directory(&root, agent)?.join("control.sock");
    for _ in 0..100 {
        if !socket.exists() && account::AccountLock::acquire(root.join("sigma.lock")).is_ok() {
            output::record(
                "agent_stopped",
                vec![("agent", agent.as_str().into())],
                &format!("Stopped {agent}"),
            );
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err("stop requested, but cleanup has not completed within 10 seconds".into())
}

/// Provision a dedicated account before dispatching either a foreground or background worker.
pub(crate) async fn prepare(args: &mut Args) -> Result<()> {
    if let Some(name) = args.option("--new-account").map(str::to_owned) {
        crate::handlers::agent_connect::executable(&args.positional[0], args.option("--exec"))?;
        let path = crate::accounts::create(args, &name).await?;
        args.options.remove("--new-account");
        args.options.remove("--account-qr");
        args.options
            .insert("--data-path".into(), path.to_string_lossy().into_owned());
    }
    Ok(())
}

pub(crate) async fn start(args: &Args) -> Result<()> {
    let root = std::fs::canonicalize(root(args)?)?;
    if !root.join("account.db").is_file() {
        return Err("account does not exist; run sigma init first".into());
    }
    let alias = crate::agent_registry::selected_alias(&root, args)?;
    let saved = crate::agent_registry::selected(&root, args)?;
    if saved.is_none() && args.option("--to").is_none() {
        return Err("a new agent requires --to TARGET".into());
    }
    let agent = &alias;
    if request(&root, agent, "status").await.is_ok() {
        return Err(
            format!("agent {agent} is already running; use agent-stop {agent} first").into(),
        );
    }
    private_directory(&root.join("agents"))?;
    let dir = directory(&root, agent)?;
    private_directory(&dir)?;
    let log_path = dir.join("worker.log");
    let log = private_log(&log_path)?;
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command.arg("agent-connect").arg(&args.positional[0]);
    for (key, value) in &args.options {
        if matches!(key.as_str(), "--data-path" | "--account" | "--json") {
            continue;
        }
        command.arg(key);
        if !matches!(
            key.as_str(),
            "--allow-outbound-calls" | "--record-calls" | "--foreground"
        ) {
            command.arg(value);
        }
    }
    command
        .arg("--data-path")
        .arg(&root)
        .arg("--foreground")
        .arg("--json")
        .stdin(std::process::Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let startup_seconds = if args.option("--invite").is_some()
        || args
            .option("--to")
            .is_some_and(crate::shared::is_securejoin)
    {
        130
    } else {
        10
    };
    for _ in 0..startup_seconds * 10 {
        if let Some(status) = child.try_wait()? {
            return Err(format!(
                "agent failed to start ({status}); inspect {}",
                log_path.display()
            )
            .into());
        }
        if let Ok(status) = request(&root, agent, "status").await {
            return print_status(&status);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // This handle identifies the exact child created above, rather than a saved PID.
    let _ = child.kill();
    let _ = child.wait();
    Err(format!(
        "agent did not become ready within {startup_seconds} seconds; inspect {}",
        log_path.display()
    )
    .into())
}
