//! Private call-request journal, plus call history read through chatmail core.
use crate::{
    Result,
    output::{self, Value},
};
use deltachat::{
    chat::{self, ChatItem},
    chatlist::Chatlist,
    context::Context,
    message::{Message, MsgId},
    webxdc::StatusUpdateItem,
};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
type Cancellations = BTreeMap<(PathBuf, String), Arc<AtomicBool>>;
static LIVE: OnceLock<Mutex<Cancellations>> = OnceLock::new();
static JOURNAL: Mutex<()> = Mutex::new(());
fn live() -> &'static Mutex<Cancellations> {
    LIVE.get_or_init(Default::default)
}
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn path(dir: &Path, id: &str) -> Result<PathBuf> {
    uuid::Uuid::parse_str(id)?;
    Ok(dir.join("calls").join(format!("{id}.json")))
}
fn read(file: &Path) -> Result<StatusUpdateItem> {
    Ok(StatusUpdateItem {
        payload: std::fs::read_to_string(file)?.parse()?,
        ..Default::default()
    })
}
fn write(file: &Path, data: &str) -> Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(file.parent().ok_or("call journal directory missing")?)?;
    let temporary = file.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut opts = std::fs::OpenOptions::new();
    opts.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(&temporary)?;
    f.write_all(data.as_bytes())?;
    f.sync_all()?;
    std::fs::rename(temporary, file)?;
    Ok(())
}
pub(crate) struct Ticket {
    pub(crate) id: String,
    dir: PathBuf,
    pub(crate) cancelled: Arc<AtomicBool>,
}
impl Ticket {
    pub(crate) fn new(dir: &Path) -> Result<Self> {
        let ticket = Self {
            id: uuid::Uuid::new_v4().to_string(),
            dir: dir.into(),
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let raw = output::encode(&output::object(vec![
            ("request_id", ticket.id.as_str().into()),
            ("created_at", now().into()),
            ("status", "queued".into()),
            ("call_id", Value::Null),
            ("duration_seconds", 0i64.into()),
            ("direction", "outgoing".into()),
        ]));
        write(&path(dir, &ticket.id)?, &raw)?;
        live()
            .lock()
            .map_err(|_| "call cancellation lock failed")?
            .insert((dir.into(), ticket.id.clone()), ticket.cancelled.clone());
        Ok(ticket)
    }
    pub(crate) fn diagnostics(&self, stage: &str, media: Option<&str>) -> Result<()> {
        let _lock = JOURNAL.lock().map_err(|_| "call journal lock failed")?;
        let file = path(&self.dir, &self.id)?;
        let mut item = read(&file)?;
        item.payload["last_stage"] = stage.into();
        item.payload["stage_updated_at"] = now().into();
        if let Some(media) = media {
            item.payload["media"] = media.parse()?;
        }
        write(&file, &item.payload.to_string())
    }
    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    pub(crate) fn check(&self) -> Result<()> {
        if self.cancelled.load(Ordering::SeqCst) {
            Err("call cancelled".into())
        } else {
            Ok(())
        }
    }
    pub(crate) fn update(
        &self,
        status: &str,
        id: Option<MsgId>,
        error: Option<&str>,
    ) -> Result<()> {
        let _lock = JOURNAL.lock().map_err(|_| "call journal lock failed")?;
        let file = path(&self.dir, &self.id)?;
        let mut item = read(&file)?;
        item.payload["status"] = status.into();
        item.payload["updated_at"] = now().into();
        if let Some(id) = id {
            item.payload["call_id"] = id.to_u32().into();
        }
        if let Some(error) = error {
            item.payload["error"] = error.chars().take(400).collect::<String>().into();
        }
        write(&file, &item.payload.to_string())
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        if let Ok(mut m) = live().lock() {
            m.remove(&(self.dir.clone(), self.id.clone()));
        }
    }
}
fn requests(dir: &Path) -> Result<Vec<StatusUpdateItem>> {
    let entries = match std::fs::read_dir(dir.join("calls")) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };
    let mut records = vec![];
    for entry in entries {
        let entry = entry?;
        if entry.path().extension().is_some_and(|e| e == "json") {
            records.push(read(&entry.path())?);
        }
    }
    Ok(records)
}
async fn call_ids(context: &Context) -> Result<Vec<MsgId>> {
    let mut ids = vec![];
    let mut chats = HashSet::new();
    for flags in [0, deltachat::constants::DC_GCL_ARCHIVED_ONLY] {
        let list = Chatlist::try_load(context, flags, None, None).await?;
        for i in 0..list.len() {
            let chat = list.get_chat_id(i)?;
            if chat.is_special() || !chats.insert(chat) {
                continue;
            }
            for item in chat::get_chat_msgs(context, chat).await? {
                if let ChatItem::Message { msg_id } = item {
                    let message = Message::load_from_db(context, msg_id).await?;
                    if message.get_viewtype() == deltachat::message::Viewtype::Call {
                        ids.push(msg_id);
                    }
                }
            }
        }
    }
    Ok(ids)
}
pub(crate) async fn control(context: &Context, dir: &Path, parts: &[&str]) -> Result<String> {
    let result = match parts {
        ["list", count] => {
            let count = count.parse::<usize>()?;
            if !(1..=100).contains(&count) {
                return Err("call count must be between 1 and 100".into());
            }
            let mut records = requests(dir)?;
            let mut known = HashSet::new();
            for record in &mut records {
                if let Some(id) = record.payload["call_id"].as_u64() {
                    known.insert(id as u32);
                }
                let id = record.payload["request_id"].as_str().unwrap_or("");
                let running = live()
                    .lock()
                    .map_err(|_| "call cancellation lock failed")?
                    .contains_key(&(dir.into(), id.into()));
                if !running
                    && matches!(
                        record.payload["status"].as_str(),
                        Some("queued" | "starting" | "ringing" | "active")
                    )
                {
                    record.payload["status"] = "interrupted".into();
                }
            }
            for id in call_ids(context).await? {
                let info = context
                    .load_call_by_id(id)
                    .await?
                    .ok_or("call disappeared")?;
                let index = records
                    .iter()
                    .position(|r| r.payload["call_id"].as_u64() == Some(u64::from(id.to_u32())));
                if !known.contains(&id.to_u32()) {
                    records.push(StatusUpdateItem {
                        payload: "{}".parse()?,
                        ..Default::default()
                    });
                }
                let index = index.unwrap_or(records.len() - 1);
                let v = &mut records[index].payload;
                v["call_id"] = id.to_u32().into();
                let manifest = dir
                    .join("recordings")
                    .join(format!("call-{}.json", id.to_u32()));
                if manifest.is_file() {
                    v["recording"] = read(&manifest)?.payload;
                }

                v["chat_id"] = info.msg.get_chat_id().to_u32().into();
                v["chat_name"] = chat::Chat::load_from_db(context, info.msg.get_chat_id())
                    .await?
                    .get_name()
                    .into();
                v["direction"] = if info.is_incoming() {
                    "incoming"
                } else {
                    "outgoing"
                }
                .into();
                v["timestamp"] = info.msg.get_timestamp().into();
                v["duration_seconds"] = info.duration_seconds().into();
                v["call_status"] = match info.call_state() {
                    deltachat::calls::CallState::Alerting => "ringing",
                    deltachat::calls::CallState::Active => "active",
                    deltachat::calls::CallState::Completed { .. } => "completed",
                    deltachat::calls::CallState::Missed => "missed",
                    deltachat::calls::CallState::Declined => "declined",
                    deltachat::calls::CallState::Canceled => "cancelled",
                }
                .into();
                if v.get("status").is_none() {
                    v["status"] = v["call_status"].clone();
                }
            }
            records.sort_by_key(|r| {
                std::cmp::Reverse(
                    r.payload["created_at"]
                        .as_i64()
                        .or(r.payload["timestamp"].as_i64())
                        .unwrap_or(0),
                )
            });
            let array = records
                .iter()
                .take(count)
                .map(|r| r.payload.to_string())
                .collect::<Vec<_>>()
                .join(",");
            return Ok(format!("{{\"ok\":true,\"calls\":[{array}]}}"));
        }
        ["cancel", selector] => {
            let mut cancelled = 0u32;
            if *selector == "all" || uuid::Uuid::parse_str(selector).is_ok() {
                let map = live().lock().map_err(|_| "call cancellation lock failed")?;
                for ((root, id), flag) in map.iter() {
                    if root == dir
                        && (*selector == "all" || id == selector)
                        && !flag.swap(true, Ordering::SeqCst)
                    {
                        cancelled += 1;
                    }
                }
            }
            if let Ok(call_id) = selector.parse::<u32>() {
                for record in requests(dir)? {
                    if record.payload["call_id"].as_u64() == Some(u64::from(call_id))
                        && let Some(id) = record.payload["request_id"].as_str()
                        && let Some(flag) = live()
                            .lock()
                            .map_err(|_| "call cancellation lock failed")?
                            .get(&(dir.into(), id.into()))
                    {
                        flag.store(true, Ordering::SeqCst);
                    }
                }
            }
            let ids = if *selector == "all" {
                call_ids(context).await?
            } else if let Ok(id) = selector.parse::<u32>() {
                vec![MsgId::new(id)]
            } else {
                let record = read(&path(dir, selector)?)?;
                record.payload["call_id"]
                    .as_u64()
                    .map(|id| vec![MsgId::new(id as u32)])
                    .unwrap_or_default()
            };
            for id in ids {
                let info = context
                    .load_call_by_id(id)
                    .await?
                    .ok_or("call ID not found")?;
                if !info.is_ended() {
                    context.end_call(id).await?;
                    cancelled += 1;
                }
            }
            output::object(vec![("ok", true.into()), ("cancelled", cancelled.into())])
        }
        _ => {
            return Err(
                "expected calls list COUNT or calls cancel CALL_ID|REQUEST_UUID|all".into(),
            );
        }
    };
    Ok(output::encode(&result))
}
