//! Named app instances and a persistent, locally controlled app worker.
use crate::{
    Result, account,
    cli::Args,
    output::{self, Value},
};
use deltachat::{
    context::Context,
    message::{Message, MsgId},
    webxdc::StatusUpdateItem,
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(crate) struct App {
    pub name: String,
    pub message_id: u32,
    pub chat_id: u32,
    pub last_serial: u32,
    pub hooks: Vec<crate::app_hooks::Hook>,
    pub hook_path: Option<PathBuf>,
    pub hook_timeout: u64,
    pub hook_after_seq: i64,
}

pub(crate) fn validate_name(name: &str) -> Result<()> {
    if name.starts_with('-')
        || name.is_empty()
        || name.len() > 48
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(
            "app name must be 1–48 ASCII letters, digits, '-' or '_', and cannot start with '-'"
                .into(),
        );
    }
    Ok(())
}
fn directory(root: &Path, name: &str) -> Result<PathBuf> {
    validate_name(name)?;
    Ok(root.join("apps").join(name))
}
fn private_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn private_file(path: &Path, append: bool) -> Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
impl App {
    fn value(&self) -> Value {
        output::object(vec![
            ("name", self.name.clone().into()),
            ("message_id", self.message_id.into()),
            ("chat_id", self.chat_id.into()),
            ("last_serial", self.last_serial.into()),
            (
                "hook_path",
                self.hook_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned().into())
                    .unwrap_or(Value::Null),
            ),
            ("hook_timeout", (self.hook_timeout as i64).into()),
            ("hook_after_seq", self.hook_after_seq.into()),
            (
                "hooks",
                Value::Array(self.hooks.iter().map(|h| h.value()).collect()),
            ),
        ])
    }
    pub(crate) fn save(&self, root: &Path) -> Result<()> {
        let dir = directory(root, &self.name)?;
        private_dir(&dir)?;
        let tmp = dir.join("app.tmp");
        let mut file = private_file(&tmp, false)?;
        writeln!(file, "{}", output::encode(&self.value()))?;
        file.sync_all()?;
        std::fs::rename(tmp, dir.join("app.json"))?;
        Ok(())
    }
    pub(crate) fn load(root: &Path, name: &str) -> Result<Self> {
        let raw =
            std::fs::read_to_string(directory(root, name)?.join("app.json")).map_err(|e| {
                format!("cannot load app {name}: {e}; use send-webxdc --app NAME or app-bind first")
            })?;
        let json = StatusUpdateItem {
            payload: raw.parse()?,
            ..Default::default()
        };
        let value = &json.payload;
        let id = |key| -> Result<u32> {
            Ok(value
                .get(key)
                .and_then(|v| v.as_u64())
                .ok_or("invalid app state")?
                .try_into()?)
        };
        if value.get("name").and_then(|v| v.as_str()) != Some(name) {
            return Err("app state name mismatch".into());
        }
        let hooks = if let Some(values) = value.get("hooks").and_then(|v| v.as_array()) {
            values
                .iter()
                .map(|v| crate::app_hooks::Hook::parse(&v.to_string()))
                .collect::<Result<Vec<_>>>()?
        } else if let Some(path) = value.get("hook_path").and_then(|v| v.as_str()) {
            vec![crate::app_hooks::Hook {
                id: "default".into(),
                path: PathBuf::from(path),
                event: "all".into(),
                match_json: None,
                timeout: value
                    .get("hook_timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30),
                after_seq: value
                    .get("hook_after_seq")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            }]
        } else {
            Vec::new()
        };
        Ok(Self {
            hooks,
            name: name.into(),
            message_id: id("message_id")?,
            chat_id: id("chat_id")?,
            last_serial: id("last_serial")?,
            hook_path: value
                .get("hook_path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from),
            hook_timeout: value
                .get("hook_timeout")
                .and_then(|v| v.as_u64())
                .unwrap_or(30),
            hook_after_seq: value
                .get("hook_after_seq")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        })
    }
}
pub(crate) fn preflight(root: &Path, name: &str) -> Result<()> {
    let dir = directory(root, name)?;
    if dir.join("app.json").exists() {
        return Err(
            format!("app {name} already exists; use app-info {name}; no app resent").into(),
        );
    }
    private_dir(&dir)
}
pub(crate) async fn bind(context: &Context, root: &Path, name: &str, id: MsgId) -> Result<App> {
    preflight(root, name)?;
    super::handlers::webxdc_updates::instance(context, &id.to_u32().to_string()).await?;
    let message = Message::load_from_db(context, id).await?;
    let app = App {
        name: name.into(),
        message_id: id.to_u32(),
        chat_id: message.get_chat_id().to_u32(),
        last_serial: 0,
        hooks: Vec::new(),
        hook_path: None,
        hook_timeout: 30,
        hook_after_seq: 0,
    };
    app.save(root)?;
    output::record(
        "app_registered",
        vec![
            ("app", name.into()),
            ("message_id", app.message_id.into()),
            ("chat_id", app.chat_id.into()),
        ],
        &format!(
            "App {name} saved (message {}); use app-start {name}",
            app.message_id
        ),
    );
    Ok(app)
}

/// Journal lines are committed before the worker reports success. Recover a torn final line.
pub(crate) struct Journal {
    pub last_line: String,
    file: std::fs::File,
    seq: i64,
    pub last_serial: u32,
    app: App,
}
impl Journal {
    pub(crate) fn open(root: &Path, app: &App) -> Result<Self> {
        let dir = directory(root, &app.name)?;
        private_dir(&dir)?;
        let path = dir.join("events.jsonl");
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let valid = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        let mut seq = 0;
        let mut last_serial = app.last_serial;
        for line in std::str::from_utf8(&bytes[..valid])?.lines() {
            let json = StatusUpdateItem {
                payload: line.parse()?,
                ..Default::default()
            };
            let next = json
                .payload
                .get("seq")
                .and_then(|v| v.as_i64())
                .ok_or("invalid app event sequence")?;
            if next <= seq {
                return Err("app event sequence is not increasing".into());
            }
            seq = next;
            if json.payload.get("type").and_then(|v| v.as_str()) == Some("webxdc_updates") {
                let cursor = json
                    .payload
                    .get("last_serial")
                    .and_then(|v| v.as_u64())
                    .ok_or("invalid stored update cursor")?;
                last_serial = last_serial.max(cursor.try_into()?);
            }
        }
        let file = private_file(&path, true)?;
        file.set_len(valid as u64)?;
        Ok(Self {
            last_line: String::new(),
            file,
            seq,
            last_serial,
            app: app.clone(),
        })
    }
    pub(crate) fn append(
        &mut self,
        kind: &str,
        fields: Vec<(&str, Value)>,
        updates: Option<&str>,
    ) -> Result<i64> {
        let next = self.seq.checked_add(1).ok_or("event sequence exhausted")?;
        let mut all = vec![
            ("seq", next.into()),
            ("type", kind.into()),
            ("app", self.app.name.clone().into()),
            ("message_id", self.app.message_id.into()),
            (
                "time",
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64)
                    .into(),
            ),
        ];
        all.extend(fields);
        let mut line = output::encode(&output::object(all));
        // Only compact, validated core update JSON is embedded here.
        if let Some(updates) = updates {
            line.pop();
            line.push_str(&format!(",\"updates\":{updates}}}"));
        }
        writeln!(self.file, "{line}")?;
        self.file.sync_data()?;
        self.seq = next;
        self.last_line = line;
        Ok(next)
    }
}

pub(crate) async fn client_command(args: &Args) -> Result<bool> {
    if !matches!(
        args.command.as_str(),
        "app-list"
            | "app-info"
            | "app-start"
            | "app-stop"
            | "app-send"
            | "app-update"
            | "app-events"
            | "app-hook-add"
            | "app-hook-list"
            | "app-hook-remove"
    ) {
        return Ok(false);
    }
    let root = crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())?;
    if args.command == "app-list" {
        let dir = root.join("apps");
        if !dir.exists() {
            return Ok(true);
        }
        let mut names = std::fs::read_dir(dir)?
            .map(|entry| entry.map(|e| e.file_name()))
            .collect::<std::io::Result<Vec<_>>>()?;
        names.sort();
        for name in names {
            let Some(name) = name.to_str() else {
                continue;
            };
            if validate_name(name).is_err() {
                continue;
            }
            if !directory(&root, name)?.join("app.json").exists() {
                continue;
            }
            let app = App::load(&root, name)?;
            output::record(
                "app",
                vec![
                    ("app", app.name.into()),
                    ("message_id", app.message_id.into()),
                    ("chat_id", app.chat_id.into()),
                ],
                name,
            );
        }
        return Ok(true);
    }
    let name = &args.positional[0];
    let app = App::load(&root, name)?;
    if args.command == "app-events" {
        let after = args.option("--after").unwrap_or("0").parse::<i64>()?;
        let path = directory(&root, name)?.join("events.jsonl");
        match std::fs::read(path) {
            Ok(bytes) => {
                let valid = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
                for line in std::str::from_utf8(&bytes[..valid])?.lines() {
                    let json = StatusUpdateItem {
                        payload: line.parse()?,
                        ..Default::default()
                    };
                    if json
                        .payload
                        .get("seq")
                        .and_then(|v| v.as_i64())
                        .ok_or("invalid event sequence")?
                        > after
                    {
                        if output::is_json() {
                            println!("{line}");
                        } else {
                            output::text(line.into());
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        return Ok(true);
    }
    #[cfg(unix)]
    {
        unix::client(&root, &app, args).await?;
    }
    #[cfg(not(unix))]
    {
        return Err("app workers currently require Unix (Linux/macOS)".into());
    }
    Ok(true)
}

pub(crate) async fn run(context: &Context, root: &Path, name: &str, args: &Args) -> Result<()> {
    #[cfg(unix)]
    {
        let mut app = App::load(root, name)?;
        crate::app_hooks::configure(&mut app, root, args)?;
        unix::worker(context, root, app).await
    }
    #[cfg(not(unix))]
    {
        let _ = (context, root, name);
        Err("app workers currently require Unix (Linux/macOS)".into())
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::time::Duration;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{UnixListener, UnixStream},
    };
    const MAX_REQUEST: usize = 1_048_576;
    fn socket(root: &Path, name: &str) -> Result<PathBuf> {
        Ok(directory(root, name)?.join("control.sock"))
    }
    async fn line(stream: &mut UnixStream) -> Result<String> {
        use tokio::io::AsyncReadExt;
        let mut bytes = Vec::new();
        let mut reader = BufReader::new(stream).take((MAX_REQUEST + 1) as u64);
        tokio::time::timeout(
            Duration::from_secs(10),
            reader.read_until(b'\n', &mut bytes),
        )
        .await
        .map_err(|_| "app control request timed out")??;
        if bytes.len() > MAX_REQUEST || !bytes.ends_with(b"\n") {
            return Err("invalid or oversized app control line".into());
        }
        Ok(String::from_utf8(bytes)?)
    }
    async fn request(root: &Path, name: &str, value: Value) -> Result<String> {
        let mut stream = UnixStream::connect(socket(root, name)?)
            .await
            .map_err(|_| format!("app {name} is not running; use app-start {name}"))?;
        stream
            .write_all(format!("{}\n", output::encode(&value)).as_bytes())
            .await?;
        line(&mut stream).await
    }
    fn request_value(op: &str) -> Value {
        output::object(vec![("op", op.into())])
    }
    fn show(raw: &str) -> Result<()> {
        let json = StatusUpdateItem {
            payload: raw.parse()?,
            ..Default::default()
        };
        if json.payload.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            return Err(json
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("app request failed")
                .to_owned()
                .into());
        }
        if output::is_json() {
            println!("{}", json.payload);
        } else {
            output::text(
                json.payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("App command completed")
                    .into(),
            );
        }
        Ok(())
    }
    pub(super) async fn client(root: &Path, app: &App, args: &Args) -> Result<()> {
        let name = &app.name;
        let status = request(root, name, request_value("status")).await;
        if matches!(
            args.command.as_str(),
            "app-hook-add" | "app-hook-list" | "app-hook-remove"
        ) {
            let list = args.command == "app-hook-list";
            let add = args.command == "app-hook-add";
            let fields = if list {
                vec![("op", "hooks".into())]
            } else if add {
                let (event, filter) = crate::app_hooks::selectors(args)?;
                let path = crate::app_hooks::executable(&args.positional[2])?;
                vec![
                    ("op", "hook-add".into()),
                    ("id", args.positional[1].clone().into()),
                    ("path", path.to_string_lossy().into_owned().into()),
                    ("event", event.into()),
                    ("match_json", filter.map(Into::into).unwrap_or(Value::Null)),
                    (
                        "timeout",
                        args.option("--hook-timeout")
                            .unwrap_or("30")
                            .parse::<i64>()?
                            .into(),
                    ),
                ]
            } else {
                vec![
                    ("op", "hook-remove".into()),
                    ("id", args.positional[1].clone().into()),
                ]
            };
            if status.is_ok() {
                return show(&request(root, name, output::object(fields)).await?);
            }
            if list {
                return show(&response(
                    app,
                    "hooks",
                    "Saved app hooks",
                    vec![(
                        "hooks",
                        Value::Array(app.hooks.iter().map(|h| h.value()).collect()),
                    )],
                ));
            }
            let _lock = account::AccountLock::acquire(root.join("sigma.lock"))?;
            let mut saved = App::load(root, name)?;
            if add {
                let (event, filter) = crate::app_hooks::selectors(args)?;
                crate::app_hooks::add(
                    &mut saved,
                    root,
                    &args.positional[1],
                    &args.positional[2],
                    &event,
                    filter.as_deref(),
                    args.option("--hook-timeout").unwrap_or("30").parse()?,
                )?;
            } else {
                crate::app_hooks::remove(&mut saved, root, &args.positional[1])?;
            }
            return show(&response(
                &saved,
                if add { "hook-add" } else { "hook-remove" },
                "Hook configuration saved",
                vec![(
                    "hooks",
                    Value::Array(saved.hooks.iter().map(|h| h.value()).collect()),
                )],
            ));
        }
        if args.command == "app-info" {
            if let Ok(status) = status {
                return show(&status);
            }
            output::record(
                "app",
                vec![
                    ("app", name.clone().into()),
                    ("message_id", app.message_id.into()),
                    ("chat_id", app.chat_id.into()),
                    ("running", false.into()),
                    (
                        "hook_path",
                        app.hook_path
                            .as_ref()
                            .map(|p| p.to_string_lossy().into_owned().into())
                            .unwrap_or(Value::Null),
                    ),
                    ("last_serial", app.last_serial.into()),
                ],
                &format!(
                    "App {name}: stopped; message {}, chat {}",
                    app.message_id, app.chat_id
                ),
            );
            return Ok(());
        }
        if args.command == "app-start" {
            if status.is_ok() && crate::app_hooks::requested(args) {
                return Err("stop the app before changing its hook configuration".into());
            }
            if let Ok(status) = status {
                return show(&status);
            }
            let root = std::fs::canonicalize(root)?;
            if crate::app_hooks::requested(args) {
                let _lock = account::AccountLock::acquire(root.join("sigma.lock"))?;
                let mut saved = App::load(&root, name)?;
                crate::app_hooks::configure(&mut saved, &root, args)?;
            }
            let dir = directory(&root, name)?;
            let mut command = std::process::Command::new(std::env::current_exe()?);
            command
                .args(["app-run", name, "--data-path"])
                .arg(&root)
                .arg("--json")
                .stdin(std::process::Stdio::null())
                .stdout(private_file(&dir.join("worker.log"), true)?)
                .stderr(private_file(&dir.join("worker.stderr"), true)?);
            use std::os::unix::process::CommandExt;
            command.process_group(0);
            let mut child = command.spawn()?;
            for _ in 0..100 {
                if let Some(exit) = child.try_wait()? {
                    return Err(format!(
                        "app worker exited ({exit}); inspect {}",
                        dir.join("worker.stderr").display()
                    )
                    .into());
                }
                if let Ok(status) = request(&root, name, request_value("status")).await {
                    return show(&status);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            // This is the child just spawned, not a saved/reused PID.
            child.kill()?;
            let _ = child.wait();
            return Err("app worker did not become ready within 10 seconds".into());
        }
        let fields = match args.command.as_str() {
            "app-stop" => vec![("op", "stop".into())],
            "app-send" => vec![
                ("op", "send".into()),
                (
                    "data",
                    Value::Array(
                        crate::realtime::payload(
                            &args.positional[1],
                            args.option("--file").is_some(),
                        )?
                        .into_iter()
                        .map(|b| u32::from(b).into())
                        .collect(),
                    ),
                ),
            ],
            "app-update" => vec![
                ("op", "update".into()),
                ("json", args.positional[1].clone().into()),
            ],
            _ => return Err("unknown app control command".into()),
        };
        let raw = request(root, name, output::object(fields)).await?;
        show(&raw)?;
        if args.command == "app-stop" {
            for _ in 0..100 {
                if !socket(root, name)?.exists() {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            return Err("stop requested, but worker cleanup has not finished; check app-info before restarting".into());
        }
        Ok(())
    }
    fn enqueue_hook(
        app: &App,
        pending: &mut crate::app_hooks::Pending,
        journal: &mut Journal,
        seq: i64,
    ) -> Result<()> {
        let line = journal.last_line.clone();
        for hook in &app.hooks {
            if let Some(job) = crate::app_hooks::select(hook, seq, &line)? {
                if pending.len() >= crate::app_hooks::QUEUE_LIMIT {
                    journal.append(
                        "hook_finished",
                        vec![
                            ("hook_id", hook.id.clone().into()),
                            ("event_seq", seq.into()),
                            ("success", false.into()),
                            ("error", "hook queue full".into()),
                        ],
                        None,
                    )?;
                } else {
                    pending.push_back(job);
                }
            }
        }
        Ok(())
    }
    struct SocketGuard(PathBuf);
    impl Drop for SocketGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    fn response(app: &App, op: &str, message: &str, mut fields: Vec<(&str, Value)>) -> String {
        let mut all = vec![
            ("type", "app_control".into()),
            ("ok", true.into()),
            ("app", app.name.clone().into()),
            ("operation", op.into()),
            ("message_id", app.message_id.into()),
            ("chat_id", app.chat_id.into()),
            ("message", message.into()),
        ];
        all.append(&mut fields);
        output::encode(&output::object(all))
    }
    pub(super) async fn worker(context: &Context, root: &Path, mut app: App) -> Result<()> {
        crate::shared::ensure_configured(context).await?;
        let id =
            crate::handlers::webxdc_updates::instance(context, &app.message_id.to_string()).await?;
        let chat = Message::load_from_db(context, id).await?.get_chat_id();
        if chat.to_u32() != app.chat_id {
            return Err("saved app chat does not match core".into());
        }
        crate::shared::sendable_chat(context, chat.to_u32()).await?;
        let path = socket(root, &app.name)?;
        if let Ok(metadata) = std::fs::symlink_metadata(&path) {
            use std::os::unix::fs::FileTypeExt;
            if !metadata.file_type().is_socket() {
                return Err("app control path exists and is not a socket".into());
            }
            std::fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path).map_err(|e| {
            format!(
                "cannot bind app socket {}: {e} (use a shorter --data-path if needed)",
                path.display()
            )
        })?;
        let _guard = SocketGuard(path.clone());
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        let mut journal = Journal::open(root, &app)?;
        if app.last_serial != journal.last_serial {
            app.last_serial = journal.last_serial;
            app.save(root)?;
        }
        let events = context.get_event_emitter();
        let (updates, serial) =
            crate::handlers::webxdc_updates::updates(context, id, app.last_serial).await?;
        if serial > app.last_serial {
            journal.append(
                "webxdc_updates",
                vec![("last_serial", serial.into())],
                Some(&updates),
            )?;
            app.last_serial = serial;
            app.save(root)?;
        }
        journal.append("worker_started", vec![], None)?;
        let mut pending = if !app.hooks.is_empty() {
            crate::app_hooks::recover(
                &directory(root, &app.name)?.join("events.jsonl"),
                &app.hooks,
            )?
        } else {
            std::collections::VecDeque::new()
        };
        let mut hook_job: Option<(
            crate::app_hooks::Job,
            tokio::task::JoinHandle<Result<String>>,
        )> = None;
        context.start_io().await;
        let mut join: Option<tokio::task::JoinHandle<Result<()>>> = None;
        let mut joined = false;
        let mut join_done = false;
        let mut realtime_enabled = false;
        journal.append("realtime_disabled", vec![], None)?;
        let result: Result<()> = async {
            loop {
                let want_realtime = crate::app_hooks::needs_realtime(&app.hooks);
                if want_realtime && !realtime_enabled {
                    let ctx = context.clone();
                    join = Some(tokio::spawn(async move {
                        if let Some(ready) = deltachat::peer_channels::send_webxdc_realtime_advertisement(&ctx, id).await? { ready.await.map_err(|_| "realtime channel closed before joining")?; }
                        Ok(())
                    }));
                    join_done = false;realtime_enabled = true;
                    journal.append("realtime_joining", vec![], None)?;
                } else if !want_realtime && realtime_enabled {
                    if let Some(task) = join.take() && !join_done { task.abort();let _ = task.await; }
                    deltachat::peer_channels::leave_webxdc_realtime(context, id).await?;
                    realtime_enabled = false;joined = false;join_done = false;
                    journal.append("realtime_left", vec![], None)?;
                }
                if hook_job.is_none() && let Some(job) = pending.pop_front() {
                    let task = crate::app_hooks::spawn(job.hook.path.clone(), job.input.clone(), directory(root, &app.name)?, job.hook.timeout);
                    hook_job = Some((job, task));
                }
                tokio::select! {
                    finished = async { let (_, job) = hook_job.as_mut().expect("hook job exists"); job.await }, if hook_job.is_some() => {
                        let (job, _) = hook_job.take().expect("completed hook exists");
                        let seq = job.seq;
                        let hook_output = match &finished { Ok(Ok(raw)) => Some(raw.clone()), _ => None };
                        let outcome: Result<()> = match finished {
                            Ok(Ok(raw)) => crate::app_hooks::actions_for_event(context, id, &raw, joined, &job.input).await,
                            Ok(Err(e)) => Err(e),
                            Err(e) => Err(e.into()),
                        };
                        let error = outcome.as_ref().err().map(ToString::to_string);
                        journal.append("hook_finished", vec![("hook_id", job.hook.id.into()), ("event_seq", seq.into()), ("success", outcome.is_ok().into()), ("error", error.map(Into::into).unwrap_or(Value::Null)), ("output", hook_output.map(Into::into).unwrap_or(Value::Null))], None)?;
                    },
                    signal = tokio::signal::ctrl_c() => { signal?;break; },
                    connected = async { join.as_mut().expect("join exists").await }, if join.is_some() && !join_done => {
                        join_done = true;
                        match connected {
                            Ok(Ok(())) => { joined = true;journal.append("realtime_joined", vec![], None)?; },
                            other => { journal.append("realtime_error", vec![("error", format!("{other:?}").into())], None)?; },
                        }
                    },
                    accepted = listener.accept() => {
                        let (mut stream, _) = accepted?;
                        let request = line(&mut stream).await;
                        let mut stop = false;
                        let reply: Result<String> = async {
                            let raw = request?;
                            let json = StatusUpdateItem { payload: raw.parse()?, ..Default::default() };
                            let op = json.payload.get("op").and_then(|v| v.as_str()).ok_or("missing app operation")?;
                            let message = match op {
                                "status" => return Ok(response(&app, op, if joined { "App worker running; realtime joined" } else if realtime_enabled { "App worker running; waiting for realtime peer" } else { "App worker running; realtime disabled (no realtime hook)" }, vec![("running", true.into()), ("realtime_joined", joined.into()), ("realtime_enabled", realtime_enabled.into()), ("last_serial", app.last_serial.into()), ("last_event", journal.seq.into()), ("hook_path", app.hook_path.as_ref().map(|p| p.to_string_lossy().into_owned().into()).unwrap_or(Value::Null)), ("hooks", Value::Array(app.hooks.iter().map(|h| h.value()).collect())), ("hook_pending", ((pending.len() + usize::from(hook_job.is_some())) as i64).into())])),
                                "hooks" => return Ok(response(&app, op, "App hooks", vec![("hooks", Value::Array(app.hooks.iter().map(|h| h.value()).collect()))])),
                                "hook-add" => {
                                    let text = |key| -> Result<&str> { json.payload.get(key).and_then(|v| v.as_str()).ok_or_else(|| format!("missing hook field {key}").into()) };
                                    crate::app_hooks::add_with_journal(&mut app, root, &mut journal, crate::app_hooks::Hook { id: text("id")?.into(), path: PathBuf::from(text("path")?), event: text("event")?.into(), match_json: json.payload.get("match_json").and_then(|v| v.as_str()).map(str::to_owned), timeout: json.payload.get("timeout").and_then(|v| v.as_u64()).ok_or("invalid hook timeout")?, after_seq: 0 })?;
                                    "Named hook added"
                                },
                                "hook-remove" => {
                                    let hook_id = json.payload.get("id").and_then(|v| v.as_str()).ok_or("missing hook id")?;
                                    crate::app_hooks::remove_with_journal(&mut app, root, &mut journal, hook_id)?;
                                    pending.retain(|job| job.hook.id != hook_id);
                                    if hook_job.as_ref().is_some_and(|(job, _)| job.hook.id == hook_id) {
                                        let (job, task) = hook_job.take().expect("hook exists");task.abort();let _ = task.await;
                                        journal.append("hook_finished", vec![("hook_id", hook_id.into()), ("event_seq", job.seq.into()), ("success", false.into()), ("error", "hook removed while running".into())], None)?;
                                    }
                                    "Named hook removed; other hooks remain active"
                                },
                                "stop" => { stop = true;"App worker stopping" },
                                "send" => {
                                    if !realtime_enabled { return Err("realtime is disabled; register a realtime hook first".into()); }
                                    if !joined { return Err("no realtime peer joined yet; keep the app open, check app-info, then retry".into()); }
                                    let data = json.payload.get("data").and_then(|v| v.as_array()).ok_or("data must be a byte array")?;
                                    if data.len() > crate::realtime::MAX_BYTES { return Err("realtime packet exceeds 128000 bytes".into()); }
                                    let bytes = data.iter().map(|v| v.as_u64().and_then(|b| u8::try_from(b).ok()).ok_or("invalid byte")).collect::<std::result::Result<Vec<_>,_>>()?;
                                    deltachat::peer_channels::send_webxdc_realtime_data(context, id, bytes.clone()).await?;
                                    journal.append("realtime_sent", crate::realtime::data_fields(id, &bytes).into_iter().filter(|(key,_)| *key != "message_id").collect(), None)?;
                                    "Realtime packet submitted (not acknowledged)"
                                },
                                "update" => {
                                    let update = json.payload.get("json").and_then(|v| v.as_str()).ok_or("missing update JSON")?;
                                    context.send_webxdc_status_update(id, update).await?;
                                    journal.append("update_queued", vec![("update_json", update.into())], None)?;
                                    "App state update queued; worker networking will transmit it"
                                },
                                _ => return Err("unknown app operation".into()),
                            };
                            Ok(response(&app, op, message, vec![]))
                        }.await;
                        let raw = match reply { Ok(raw) => raw, Err(e) => output::encode(&output::object(vec![("type", "app_control".into()), ("ok", false.into()), ("message", e.to_string().into())])) };
                        // A disconnected client must not kill the worker or cause a retry/send duplication.
                        let _ = stream.write_all(format!("{raw}\n").as_bytes()).await;
                        if stop { break; }
                    },
                    event = events.recv() => match event {
                        Some(event) => match event.typ {
                            deltachat::EventType::WebxdcRealtimeData { msg_id, data } if msg_id == id && realtime_enabled => {
                                joined = true;
                                let seq = journal.append("realtime_received", crate::realtime::data_fields(id, &data).into_iter().filter(|(key,_)| *key != "message_id").collect(), None)?;
                                enqueue_hook(&app, &mut pending, &mut journal, seq)?;
                            },
                            deltachat::EventType::IncomingMsg { chat_id, msg_id } if chat_id == chat => {
                                let message = Message::load_from_db(context, msg_id).await?;
                                if message.get_from_id() != deltachat::contact::ContactId::SELF && !message.is_info() {
                                    let text = message.get_text();
                                    let fields = vec![("incoming_message_id", msg_id.to_u32().into()), ("text", text.clone().into()), ("sender_id", message.get_from_id().to_u32().into()), ("message", crate::shared::message_value(context, &message).await?)];
                                    let seq = journal.append("message_received", fields.clone(), None)?;enqueue_hook(&app, &mut pending, &mut journal, seq)?;
                                    if let Some((command, args)) = crate::app_hooks::command(&text) {
                                        let mut fields = fields;fields.push(("command", command.into()));fields.push(("args", args.into()));
                                        let seq = journal.append("command_received", fields, None)?;enqueue_hook(&app, &mut pending, &mut journal, seq)?;
                                    }
                                }
                            },
                            deltachat::EventType::WebxdcStatusUpdate { msg_id, .. } if msg_id == id => {
                                let (updates, serial) = crate::handlers::webxdc_updates::updates(context, id, app.last_serial).await?;
                                if serial > app.last_serial { let seq = journal.append("webxdc_updates", vec![("last_serial", serial.into())], Some(&updates))?;enqueue_hook(&app, &mut pending, &mut journal, seq)?;app.last_serial = serial;app.save(root)?; }
                            },
                            deltachat::EventType::IncomingWebxdcNotify { msg_id, text, .. } => {
                                let notification = Message::load_from_db(context, msg_id).await?;
                                if msg_id == id || notification.parent(context).await?.is_some_and(|parent| parent.get_id() == id) { journal.append("notification", vec![("text", text.into())], None)?; }
                            },
                            deltachat::EventType::WebxdcInstanceDeleted { msg_id } if msg_id == id => return Err("named app message was deleted".into()),
                            _ => {},
                        },
                        None => break,
                    },
                }
            }
            Ok(())
        }.await;
        if let Some((_, job)) = hook_job {
            job.abort();
            let _ = job.await;
        }
        if let Some(task) = join
            && !join_done
        {
            task.abort();
            let _ = task.await;
        }
        let leave = deltachat::peer_channels::leave_webxdc_realtime(context, id).await;
        context.stop_io().await;
        let logged = if result.is_ok() {
            journal.append("worker_stopped", vec![("reason", "requested".into())], None)
        } else {
            Ok(journal.seq)
        };
        result?;
        leave?;
        logged?;
        Ok(())
    }
}
