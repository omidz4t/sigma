//! Optional Chromium media bridge; CDP uses private inherited pipes, never a port.
use crate::{
    Result,
    output::{self, Value},
};
use deltachat::webxdc::StatusUpdateItem;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

pub(crate) fn executable() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("SIGMA_CALL_BROWSER") {
        let path = PathBuf::from(path);
        if path.is_absolute() && path.is_file() {
            return Ok(path);
        }
        return Err("SIGMA_CALL_BROWSER must name an existing absolute Chromium executable".into());
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        for name in ["chromium", "chromium-browser", "google-chrome"] {
            let path = dir.join(name);
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let root = PathBuf::from(home).join(".cache/ms-playwright");
        if let Ok(entries) = std::fs::read_dir(root) {
            let mut paths = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect::<Vec<_>>();
            paths.sort();
            for dir in paths.into_iter().rev() {
                for suffix in [
                    "chrome-linux/chrome",
                    "chrome-linux64/chrome",
                    "chrome-headless-shell-linux64/chrome-headless-shell",
                ] {
                    let path = dir.join(suffix);
                    if path.is_file() {
                        return Ok(path);
                    }
                }
            }
        }
    }
    Err(
        "outbound calls require installed Chromium; set SIGMA_CALL_BROWSER to its executable"
            .into(),
    )
}
pub(crate) struct Browser {
    child: tokio::process::Child,
    input: tokio::process::ChildStdin,
    output: BufReader<tokio::process::ChildStdout>,
    profile: PathBuf,
    sequence: u32,
    session: Option<String>,
}
impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
        // Give Chromium time to exit before removing its private profile.
        let path = self.profile.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = std::fs::remove_dir_all(path);
        });
    }
}
impl Browser {
    pub(crate) async fn start(exe: &Path) -> Result<Self> {
        let profile =
            std::env::temp_dir().join(format!("sigma-call-browser-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&profile)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&profile, std::fs::Permissions::from_mode(0o700))?;
        }
        let mut child = tokio::process::Command::new("/bin/sh")
            .args(["-c", "exec 3<&0 4>&1; exec \"$@\"", "sigma-call-browser"])
            .arg(exe)
            .args([
                "--headless",
                "--remote-debugging-pipe",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
                "--autoplay-policy=no-user-gesture-required",
            ])
            .arg(format!("--user-data-dir={}", profile.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let input = child.stdin.take().ok_or("browser stdin missing")?;
        let output = BufReader::new(child.stdout.take().ok_or("browser stdout missing")?);
        let mut browser = Self {
            child,
            input,
            output,
            profile,
            sequence: 0,
            session: None,
        };
        let target = browser
            .rpc(
                "Target.createTarget",
                output::object(vec![("url", "about:blank".into())]),
            )
            .await?;
        let id = target.payload["result"]["targetId"]
            .as_str()
            .ok_or("browser target missing")?;
        let attach = browser
            .rpc(
                "Target.attachToTarget",
                output::object(vec![("targetId", id.into()), ("flatten", true.into())]),
            )
            .await?;
        browser.session = Some(
            attach.payload["result"]["sessionId"]
                .as_str()
                .ok_or("browser session missing")?
                .into(),
        );
        Ok(browser)
    }
    async fn rpc(&mut self, method: &str, params: Value) -> Result<StatusUpdateItem> {
        self.sequence += 1;
        let id = self.sequence;
        let mut fields = vec![
            ("id", id.into()),
            ("method", method.into()),
            ("params", params),
        ];
        if let Some(session) = &self.session {
            fields.push(("sessionId", session.as_str().into()));
        }
        let mut data = output::encode(&output::object(fields));
        data.push('\0');
        self.input.write_all(data.as_bytes()).await?;
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                let mut buf = vec![];
                (&mut self.output)
                    .take(1024 * 1024 + 1)
                    .read_until(0, &mut buf)
                    .await?;
                if buf.is_empty() || buf.len() > 1024 * 1024 {
                    return Err("Chromium call transport closed or oversized".into());
                }
                if buf.pop() != Some(0) {
                    return Err("incomplete Chromium response".into());
                }
                let item = StatusUpdateItem {
                    payload: String::from_utf8(buf)?.parse()?,
                    ..Default::default()
                };
                if item.payload["id"].as_u64() != Some(u64::from(id)) {
                    continue;
                }
                if item.payload.get("error").is_some() {
                    return Err("Chromium call command failed".into());
                }
                return Ok(item);
            }
        })
        .await
        .map_err(|_| "Chromium call command timed out")?
    }
    pub(crate) async fn eval(&mut self, expression: &str) -> Result<StatusUpdateItem> {
        let result = self
            .rpc(
                "Runtime.evaluate",
                output::object(vec![
                    ("expression", expression.into()),
                    ("awaitPromise", true.into()),
                    ("returnByValue", true.into()),
                ]),
            )
            .await?;
        if result.payload["result"].get("exceptionDetails").is_some() {
            return Err("Chromium audio bridge failed".into());
        }
        Ok(StatusUpdateItem {
            payload: result.payload["result"]["result"]["value"].clone(),
            ..Default::default()
        })
    }
    pub(crate) async fn offers(&mut self, ice: &str) -> Result<(String, String)> {
        self.negotiate(ice, None).await
    }
    pub(crate) async fn incoming(&mut self, ice: &str, offer: &str) -> Result<(String, String)> {
        self.negotiate(ice, Some(offer)).await
    }
    async fn negotiate(&mut self, ice: &str, incoming: Option<&str>) -> Result<(String, String)> {
        // ICE configuration comes from trusted chatmail core, not the model.
        let value: StatusUpdateItem = StatusUpdateItem {
            payload: ice.parse()?,
            ..Default::default()
        };
        if !value.payload.is_array() {
            return Err("invalid core ICE configuration".into());
        }
        let v = self
            .eval(&format!(
                "({})({}, {})",
                include_str!("call_browser.js"),
                value.payload,
                output::encode(&incoming.map(Value::from).unwrap_or(Value::Null))
            ))
            .await?;
        Ok((
            v.payload["peer"]
                .as_str()
                .ok_or("peer offer missing")?
                .into(),
            v.payload["provider"]
                .as_str()
                .ok_or("provider offer missing")?
                .into(),
        ))
    }
    pub(crate) async fn answer(&mut self, provider: bool, sdp: &str) -> Result<()> {
        crate::agent_calls::validate_offer(sdp, false)?;
        let quoted = output::encode(&Value::from(sdp));
        self.eval(&format!(
            "sigmaCall.{}.setRemoteDescription({{type:'answer',sdp:{quoted}}}).then(()=>true)",
            if provider { "provider" } else { "peer" }
        ))
        .await?;
        Ok(())
    }
}
