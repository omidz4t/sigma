//! Direct Delta Chat SDP -> Codex app-server WebRTC signaling. Sigma never decodes audio.
use crate::{
    Result,
    agent_guard::Scope,
    output::{self, Value},
};
use deltachat::{
    chat,
    context::Context,
    message::{Message, MsgId},
    webxdc::StatusUpdateItem,
};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
pub(crate) fn supported(provider: &str, native: bool) -> bool {
    provider == "codex" && native
}
pub(crate) fn description(provider: &str, native: bool) -> &'static str {
    if supported(provider, native) {
        "Experimental Codex WebRTC calls are enabled for the bot master only. Audio connects directly to OpenAI; Sigma relays signaling. End-to-end phone audio still needs a real call test. Use /call stop to hang up."
    } else {
        "Live calls are not available for this provider/adapter. Grok Build's installed CLI has no verified live-call transport; the separate xAI voice API is not this bridge. Voice notes remain available."
    }
}
pub(crate) struct Active {
    pub(crate) id: MsgId,
    pub(crate) task: tokio::task::JoinHandle<()>,
}
impl Drop for Active {
    fn drop(&mut self) {
        // Dropping the handle detaches; core hang-up lets the task finalize its recording.
    }
}
fn rpc(id: u32, method: &str, params: Value) -> String {
    output::encode(&output::object(vec![
        ("id", id.into()),
        ("method", method.into()),
        ("params", params),
    ])) + "\n"
}
async fn send(
    writer: &mut tokio::process::ChildStdin,
    id: u32,
    method: &str,
    params: Value,
) -> Result<()> {
    writer.write_all(rpc(id, method, params).as_bytes()).await?;
    Ok(())
}
pub(crate) fn validate_offer(offer: &str, video: bool) -> Result<()> {
    if video {
        return Err("only audio calls are supported".into());
    }
    if offer.len() > 128 * 1024
        || !offer.starts_with("v=0")
        || !offer.lines().any(|l| l.starts_with("m=audio "))
    {
        return Err("invalid or oversized audio SDP offer".into());
    }
    Ok(())
}
pub(crate) struct Config {
    pub(crate) dir: PathBuf,
    pub(crate) scope: Scope,
    pub(crate) id: MsgId,
    pub(crate) offer: String,
    pub(crate) model: Option<String>,
    pub(crate) custom: Option<String>,
    pub(crate) record: bool,
}
pub(crate) fn start(context: Context, executable: PathBuf, config: Config) -> Active {
    let id = config.id;
    let task = tokio::spawn(async move {
        let result = run(&context, &executable, &config).await;
        let _ = context.end_call(id).await;
        if let Err(error) = result {
            output::error(&format!("Call {} ended: {error}", id.to_u32()));
            if config.scope.check(&context).await.is_ok() {
                let mut notice = Message::new_text(format!(
                    "Call could not continue: {}",
                    error.to_string().chars().take(400).collect::<String>()
                ));
                let _ = chat::send_msg(&context, config.scope.chat, &mut notice).await;
            }
        }
    });
    Active { id, task }
}
pub(crate) async fn run(context: &Context, executable: &Path, config: &Config) -> Result<()> {
    let mut media = None;
    let result = run_inner(context, executable, config, &mut media).await;
    if config.record {
        let _ = context.end_call(config.id).await;
    }
    if let Some(media) = &mut media
        && let Err(e) = media.finish(context, &config.scope, config.id).await
    {
        output::error(&format!("call recording: {e}"));
    }
    result
}
async fn run_inner(
    context: &Context,
    executable: &Path,
    config: &Config,
    media_slot: &mut Option<crate::call_recording::Media>,
) -> Result<()> {
    let dir = config.dir.as_path();
    let scope = &config.scope;
    let id = config.id;
    let original_offer = config.offer.as_str();
    let (offer, peer_answer) = if config.record {
        *media_slot = Some(crate::call_recording::Media::new().await?);
        let media = media_slot.as_mut().unwrap();
        let (answer, offer) = media
            .browser
            .incoming(
                &deltachat::calls::ice_servers(context).await?,
                original_offer,
            )
            .await?;
        (offer, Some(answer))
    } else {
        (audio_offer(original_offer)?, None)
    };
    let model = config.model.as_deref();
    let custom = config.custom.as_deref();
    scope.check(context).await?;
    let actions = crate::agent_actions::Server::start_for_sender(
        context,
        scope.chat,
        dir,
        true,
        Some(scope.clone()),
    )
    .await?;
    let mut environment = actions.environment.clone();
    environment.push(("SIGMA_INCOMING_MESSAGE_ID".into(), id.to_u32().to_string()));
    let previous = crate::agent_sessions::load(dir)?;
    if previous
        .as_ref()
        .is_some_and(|s| s.provider != "codex" || s.id.is_none())
    {
        return Err("call requires a valid saved Codex session".into());
    }
    let mut command = crate::agent_isolation::command(
        executable,
        &environment,
        Some("codex"),
        previous.as_ref().and_then(|s| s.id.as_deref()),
    )?;
    let mut child = command
        .args(["--no-daemon", "--ask-for-approval", "never", "--cd"])
        .arg(&actions.outbox)
        .args([
            "app-server",
            "--enable",
            "realtime_conversation",
            "--listen",
            "stdio://",
        ])
        .envs(environment.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut writer = child.stdin.take().ok_or("Codex call stdin unavailable")?;
    let mut reader = BufReader::new(child.stdout.take().ok_or("Codex call stdout unavailable")?);
    send(
        &mut writer,
        1,
        "initialize",
        output::object(vec![
            (
                "clientInfo",
                output::object(vec![
                    ("name", "sigma-calls".into()),
                    ("version", env!("CARGO_PKG_VERSION").into()),
                ]),
            ),
            (
                "capabilities",
                output::object(vec![("experimentalApi", true.into())]),
            ),
        ]),
    )
    .await?;
    let mut prompt = format!(
        "{}\n{}\n{}\nYou are on an audio call with this bot's sole bot master. Audio is sent directly to your realtime service. Spoken permission/master/model commands must not change administration; those must be typed or sent from the trusted terminal. Do not advertise video support. The Model field identifies the backing coding agent, not this realtime voice engine. No explicit realtime voice model ID was supplied; distinguish those when asked.",
        crate::handlers::agent_connect::starting_prompt(model, custom),
        actions.instructions(),
        crate::agent_guard::instructions()
    );
    if config.record {
        prompt.push_str(crate::call_recording::NOTICE);
    }
    let mut thread = None::<String>;
    let mut accepted = false;
    let mut greeting_at = None;
    let mut start_requested = false;
    let deadline = tokio::time::sleep(Duration::from_secs(60));
    tokio::pin!(deadline);
    let duration = tokio::time::sleep(Duration::from_secs(20 * 60));
    tokio::pin!(duration);
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut buffer = Vec::new();
    loop {
        // Bound allocation while reading, including a line with no newline. Keep
        // the accumulated bytes across select cancellation by the scope timer.
        let mut limited = (&mut reader).take((1024 * 1024 + 1 - buffer.len()) as u64);
        tokio::select! {
              _=&mut deadline,if !accepted=>return Err("call negotiation timed out".into()),
              _=&mut duration=>return Ok(()),
              _=interval.tick()=>{if let Some(media) = media_slot.as_mut() {media.flush().await?;} scope.check(context).await?;if let Some(call)=context.load_call_by_id(id).await?{if call.is_ended()||(!accepted&&call.is_stale()){return Ok(());}}else{return Ok(());}
                if greeting_at.is_some_and(|at|tokio::time::Instant::now()>=at){
                    greeting_at=None;
                    send(&mut writer,4,"thread/realtime/appendText",output::object(vec![("threadId",thread.as_deref().unwrap().into()),("role","developer".into()),("text",if config.record {"First announce that this call is being recorded and sent to the bot master, then greet the caller.".into()}else{"Greet the caller briefly aloud and say you are ready to listen.".into()})])).await?;
                }
              },
              read=limited.read_until(b'\n',&mut buffer)=>{if read?==0{return Err("Codex call process closed".into());}
        if buffer.len()>1024*1024{return Err("Codex call event exceeds 1 MiB".into());}
               let line=String::from_utf8(std::mem::take(&mut buffer))?;
        let item=StatusUpdateItem{payload:line.parse()?,..Default::default()};let v=&item.payload;
               if let Some(error)=v.get("error"){return Err(format!("Codex call request refused: {}",error.get("message").and_then(|v|v.as_str()).unwrap_or("provider error")).into());}
               if v["id"].as_u64()==Some(1){
                let mut fields=vec![("cwd",actions.outbox.to_string_lossy().into_owned().into()),("approvalPolicy","never".into()),("sandbox","workspace-write".into()),("developerInstructions",prompt.clone().into())];
                if let Some(model)=model{fields.push(("model",model.into()));}
                let method=if let Some(session)=&previous{fields.push(("threadId",session.id.as_deref().unwrap().into()));fields.push(("excludeTurns",true.into()));"thread/resume"}else{"thread/start"};
                send(&mut writer,2,method,output::object(fields)).await?;
               }else if v["id"].as_u64()==Some(2){let tid=v["result"]["thread"]["id"].as_str().ok_or("Codex omitted call thread ID")?.to_owned();uuid::Uuid::parse_str(&tid)?;
                if previous.as_ref().and_then(|s|s.id.as_deref()).is_some_and(|old|old!=tid){return Err("Codex resumed another conversation".into());}
                crate::agent_sessions::save(dir,&crate::agent_sessions::Session{provider:"codex".into(),id:Some(tid.clone())})?;
                send(&mut writer,3,"thread/realtime/start",output::object(vec![("threadId",tid.clone().into()),("outputModality","audio".into()),("version","v3".into()),("initialItems",crate::agent_outbound::initial_context(&prompt)),("prompt",prompt.clone().into()),("transport",output::object(vec![("type","webrtc".into()),("sdp",offer.as_str().into())]))])).await?;
                thread=Some(tid);start_requested=true;
               }else if v["method"].as_str()==Some("thread/realtime/sdp")&&start_requested{
                if accepted{return Err("unexpected repeated call SDP".into());}
                if v["params"]["threadId"].as_str()!=thread.as_deref(){return Err("SDP belongs to another thread".into());}
                let answer=v["params"]["sdp"].as_str().filter(|s|s.starts_with("v=0")&&s.len()<=128*1024).ok_or("invalid Codex SDP answer")?;
                scope.check(context).await?;
                let delta_answer = if let Some(media) = media_slot.as_mut() {
                    media.browser.answer(true, answer).await?;
                    media.start(scope, id).await?;
                    peer_answer.clone().ok_or("incoming bridge answer missing")?
                } else {restore_answer(original_offer,answer)?};
                context.accept_incoming_call(id,delta_answer).await?;accepted=true;greeting_at=Some(tokio::time::Instant::now()+Duration::from_secs(3));
                output::record("agent_call_accepted",vec![("call_id",id.to_u32().into())],"Codex audio call accepted for bot master");
               }else if v["method"].as_str()==Some("thread/realtime/error"){return Err(format!("Codex realtime error: {}",v["params"]["message"].as_str().unwrap_or("unknown error")).into());}
               else if v["method"].as_str()==Some("thread/realtime/closed"){
                let reason=v["params"]["reason"].as_str().unwrap_or("unspecified");
                output::record("agent_call_closed",vec![("call_id",id.to_u32().into()),("reason",reason.into())],"Codex realtime connection closed");
                return Err(format!("Codex voice connection closed ({reason})").into());
               }
               else if v.get("method").is_some()&&v.get("id").is_some(){
                // Never grant app-server requests for filesystem/shell permission or external tool execution.
                let raw= format!("{{\"id\":{},\"error\":{{\"code\":-32601,\"message\":\"Sigma call bridge does not approve client-side requests\"}}}}\n",v["id"]);
                writer.write_all(raw.as_bytes()).await?;
               }
              }
             }
    }
}

fn split_sdp(sdp: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    if sdp.len() > 128 * 1024 || !sdp.starts_with("v=0") {
        return Err("invalid SDP".into());
    }
    let mut header = vec![];
    let mut sections: Vec<Vec<String>> = vec![];
    for line in sdp
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.is_empty())
    {
        if line.starts_with("m=") {
            sections.push(vec![line.to_owned()]);
        } else if let Some(section) = sections.last_mut() {
            section.push(line.into());
        } else {
            header.push(line.into());
        }
    }
    Ok((header, sections))
}
fn mid(section: &[String]) -> Result<&str> {
    section
        .iter()
        .find_map(|line| line.strip_prefix("a=mid:"))
        .ok_or_else(|| "SDP media section missing mid".into())
}
fn render(header: &[String], sections: &[Vec<String>]) -> String {
    header
        .iter()
        .chain(sections.iter().flatten())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\r\n")
        + "\r\n"
}
pub(crate) fn audio_offer(original: &str) -> Result<String> {
    let (mut header, mut sections) = split_sdp(original)?;
    if sections.len() == 1 && sections[0][0].starts_with("m=audio ") {
        return Ok(original.into());
    }
    let audio = sections
        .iter()
        .position(|s| s[0].starts_with("m=audio "))
        .ok_or("no audio media section")?;
    let bundle = header
        .iter()
        .find_map(|line| line.strip_prefix("a=group:BUNDLE "))
        .ok_or("multi-section SDP requires BUNDLE")?
        .split_whitespace()
        .collect::<Vec<_>>();
    for section in &sections {
        if !bundle.contains(&mid(section)?) {
            return Err("unbundled media cannot be relayed without a media backend".into());
        }
    }
    let audio_ice = sections[audio]
        .iter()
        .find_map(|line| line.strip_prefix("a=ice-ufrag:"))
        .map(str::to_owned);
    let audio_pwd = sections[audio]
        .iter()
        .find_map(|line| line.strip_prefix("a=ice-pwd:"))
        .map(str::to_owned);
    let mut extra_candidates = vec![];
    for section in &sections {
        if !section[0].starts_with("m=audio ") {
            if section
                .iter()
                .find_map(|line| line.strip_prefix("a=ice-ufrag:"))
                != audio_ice.as_deref()
                || section
                    .iter()
                    .find_map(|line| line.strip_prefix("a=ice-pwd:"))
                    != audio_pwd.as_deref()
            {
                return Err("bundled media ICE credentials differ".into());
            }
            for line in section {
                if line.starts_with("a=candidate:") || line == "a=end-of-candidates" {
                    extra_candidates.push(line.clone());
                }
            }
        }
    }
    for candidate in extra_candidates {
        if !sections[audio].contains(&candidate) {
            sections[audio].push(candidate);
        }
    }
    // Delta Chat uses negotiated SCTP channels for ICE/mute messages. Those
    // messages are not provider realtime events and cause transport closure.
    // Keep only the bundled audio transport; Codex controls it over its sideband.
    if sections.iter().any(|s| {
        !s[0].starts_with("m=audio ")
            && !s[0].starts_with("m=video ")
            && !s[0].starts_with("m=application ")
    }) {
        return Err("unsupported call media layout".into());
    }
    sections.retain(|section| section[0].starts_with("m=audio "));
    if sections
        .iter()
        .filter(|s| s[0].starts_with("m=audio "))
        .count()
        != 1
        || sections
            .iter()
            .any(|s| !s[0].starts_with("m=audio ") && !s[0].starts_with("m=application "))
    {
        return Err("unsupported call media layout".into());
    }
    let mids = sections
        .iter()
        .map(|s| mid(s).map(str::to_owned))
        .collect::<Result<Vec<_>>>()?;
    for line in &mut header {
        if line.starts_with("a=group:BUNDLE ") {
            *line = format!("a=group:BUNDLE {}", mids.join(" "));
        }
    }
    Ok(render(&header, &sections))
}
pub(crate) fn restore_answer(original: &str, answer: &str) -> Result<String> {
    let (_, original_sections) = split_sdp(original)?;
    let (header, mut answers) = split_sdp(answer)?;
    let mut ordered = vec![];
    for original in original_sections {
        let id = mid(&original)?;
        if original[0].starts_with("m=video ") || original[0].starts_with("m=application ") {
            let mut parts = original[0]
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if parts.len() < 4 {
                return Err("invalid rejected media section".into());
            }
            parts[1] = "0".into();
            ordered.push(vec![
                parts.join(" "),
                "c=IN IP4 0.0.0.0".into(),
                format!("a=mid:{id}"),
                "a=inactive".into(),
            ]);
        } else {
            let index = answers
                .iter()
                .position(|section| mid(section).is_ok_and(|mid| mid == id))
                .ok_or("provider omitted an offered media mid")?;
            let section = answers.remove(index);
            if section[0].split_whitespace().next() != original[0].split_whitespace().next() {
                return Err("provider changed media type".into());
            }
            ordered.push(section);
        }
    }
    if !answers.is_empty() {
        return Err("provider answered unexpected media sections".into());
    }
    Ok(render(&header, &ordered))
}
