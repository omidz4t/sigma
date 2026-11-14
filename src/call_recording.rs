//! Streaming Opus recordings kept outside the agent sandbox/outbox.
use crate::{Result, agent_guard::Scope, call_browser::Browser};
use deltachat::{
    chat,
    context::Context,
    message::{MsgId, Viewtype},
};
use std::{io::Write, path::PathBuf};
pub(crate) const NOTICE: &str = "\nThis call is being recorded and the recording will be sent to the bot master. Announce this clearly in your first spoken sentence.";
pub(crate) struct Media {
    pub(crate) browser: Browser,
    recording: Option<Recording>,
}
struct Recording {
    file: std::fs::File,
    path: PathBuf,
    manifest: PathBuf,
    bytes: usize,
}
impl Media {
    pub(crate) async fn new() -> Result<Self> {
        Ok(Self {
            browser: Browser::start(&crate::call_browser::executable()?).await?,
            recording: None,
        })
    }
    pub(crate) async fn start(&mut self, scope: &Scope, id: MsgId) -> Result<()> {
        let dir = scope.dir.join("recordings");
        std::fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        }
        let path = dir.join(format!("call-{}.webm.part", id.to_u32()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        let manifest = dir.join(format!("call-{}.json", id.to_u32()));
        metadata(&manifest, &path, "recording", false, None)?;
        self.recording = Some(Recording {
            file,
            path,
            manifest,
            bytes: 0,
        });
        self.browser.eval("sigmaCall.startRecording()").await?;
        Ok(())
    }
    pub(crate) async fn flush(&mut self) -> Result<()> {
        let Some(recording) = &mut self.recording else {
            return Ok(());
        };
        // Each chunk is <=256 KiB, keeping CDP responses below its 1 MiB cap.
        for _ in 0..64 {
            let chunk = self.browser.eval("sigmaCall.recordChunk()").await?;
            if chunk.payload.is_null() {
                recording.file.flush()?;
                return Ok(());
            }
            let hex = chunk.payload.as_str().ok_or("invalid recording chunk")?;
            if hex.len() > 524288 || !hex.len().is_multiple_of(2) || !hex.is_ascii() {
                return Err("invalid recording size".into());
            }
            let bytes = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            recording.bytes += bytes.len();
            if recording.bytes > 32 * 1024 * 1024 {
                return Err("recording exceeded 32 MiB".into());
            }
            recording.file.write_all(&bytes)?;
        }
        Err("recording chunk backlog exceeded".into())
    }
    pub(crate) async fn finish(
        &mut self,
        context: &Context,
        scope: &Scope,
        id: MsgId,
    ) -> Result<()> {
        if self.recording.is_none() {
            return Ok(());
        }
        let stopped = self.browser.eval("sigmaCall.stopRecording()").await;
        let drained = self.flush().await;
        let recording = self.recording.take().unwrap();
        recording.file.sync_all()?;
        if recording.bytes == 0 {
            std::fs::remove_file(&recording.path)?;
            std::fs::remove_file(&recording.manifest)?;
            return Ok(());
        }
        let path = recording.path.with_extension(""); // remove .part, preserve .webm
        std::fs::rename(&recording.path, &path)?;
        if stopped.is_err() || drained.is_err() {
            crate::output::error("Saved partial call recording after recorder failure");
        }
        let partial = stopped.is_err() || drained.is_err();
        metadata(&recording.manifest, &path, "saved", partial, None)?;
        // Master changes revoke sharing, but the private local copy survives.
        if scope.check(context).await.is_ok() {
            let caption = format!(
                "Recording of call {}{}",
                id.to_u32(),
                if stopped.is_err() || drained.is_err() {
                    " (partial)"
                } else {
                    ""
                }
            );
            let mut message =
                crate::shared::attachment_message(context, &path, Some(&caption), Viewtype::File)?;
            let message_id = chat::send_msg(context, scope.owner_chat, &mut message).await?;
            metadata(
                &recording.manifest,
                &path,
                "queued-to-master",
                partial,
                Some(message_id),
            )?;
        }
        Ok(())
    }
}

fn metadata(
    manifest: &std::path::Path,
    path: &std::path::Path,
    status: &str,
    partial: bool,
    message: Option<MsgId>,
) -> Result<()> {
    let data = crate::output::encode(&crate::output::object(vec![
        ("path", path.to_string_lossy().into_owned().into()),
        ("status", status.into()),
        ("partial", partial.into()),
        (
            "message_id",
            message
                .map(|id| id.to_u32().into())
                .unwrap_or(crate::output::Value::Null),
        ),
    ]));
    let temporary = manifest.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(data.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(temporary, manifest)?;
    Ok(())
}
