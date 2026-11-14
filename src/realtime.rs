//! Webxdc ephemeral peer channels, entirely through chatmail core.
use crate::{Result, cli::Args, output};
use deltachat::{EventType, context::Context, message::MsgId, peer_channels};
use std::{io::Read, time::Duration};

pub(crate) const MAX_BYTES: usize = 128_000;

pub(crate) fn payload(value: &str, file: bool) -> Result<Vec<u8>> {
    let bytes = if file {
        let mut bytes = Vec::new();
        std::fs::File::open(value)?
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        bytes
    } else {
        value.as_bytes().to_vec()
    };
    if bytes.len() > MAX_BYTES {
        return Err("realtime data must not exceed 128000 bytes".into());
    }
    Ok(bytes)
}

pub(crate) fn data_fields(id: MsgId, data: &[u8]) -> Vec<(&'static str, output::Value)> {
    vec![
        ("message_id", id.to_u32().into()),
        (
            "data",
            output::Value::Array(data.iter().map(|b| u32::from(*b).into()).collect()),
        ),
        (
            "text",
            std::str::from_utf8(data)
                .map(|text| text.into())
                .unwrap_or(output::Value::Null),
        ),
        ("bytes", (data.len() as u32).into()),
    ]
}

pub(crate) fn print_data(id: MsgId, data: &[u8]) {
    output::record(
        "webxdc_realtime_data",
        data_fields(id, data),
        &format!(
            "App message {} realtime: {}",
            id.to_u32(),
            std::str::from_utf8(data)
                .map(str::to_owned)
                .unwrap_or_else(|_| format!("{data:?}"))
        ),
    );
}

pub(crate) async fn session(
    context: &Context,
    args: &Args,
    id: MsgId,
    initial: Option<Vec<u8>>,
    keep_listening: bool,
) -> Result<()> {
    crate::shared::ensure_configured(context).await?;
    let chat = deltachat::message::Message::load_from_db(context, id)
        .await?
        .get_chat_id();
    crate::shared::sendable_chat(context, chat.to_u32()).await?;
    let events = context.get_event_emitter();
    context.start_io().await;
    eprintln!("Joining realtime channel; keep the same app open on another peer. Ctrl-C stops.");
    let operation = async {
        let connect = async {
            if let Some(joined) =
                peer_channels::send_webxdc_realtime_advertisement(context, id).await?
            {
                joined
                    .await
                    .map_err(|_| "realtime channel closed before a peer connected")?;
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        };
        let timeout = args.option("--timeout").unwrap_or("120").parse()?;
        tokio::time::timeout(Duration::from_secs(timeout), connect).await
            .map_err(|_| "realtime connection timed out; keep the same app open on another peer and retry (no data sent)")??;
        output::record(
            "webxdc_realtime_joined",
            vec![("message_id", id.to_u32().into())],
            &format!("Joined realtime channel for app message {}", id.to_u32()),
        );
        if let Some(data) = initial {
            let len = data.len() as u32;
            peer_channels::send_webxdc_realtime_data(context, id, data).await?;
            output::record(
                "webxdc_realtime_sent",
                vec![("message_id", id.to_u32().into()), ("bytes", len.into())],
                &format!("Submitted {len} realtime bytes (delivery is not acknowledged)"),
            );
        }
        if !keep_listening {
            // Give gossip's outgoing queue time to run; this is not a delivery receipt.
            tokio::time::sleep(Duration::from_secs(2)).await;
            return Ok(());
        }
        while let Some(event) = events.recv().await {
            match event.typ {
                EventType::WebxdcRealtimeData { msg_id, data } if msg_id == id => {
                    print_data(id, &data)
                }
                EventType::WebxdcInstanceDeleted { msg_id } if msg_id == id => {
                    return Err("webxdc app message was deleted".into());
                }
                _ => {}
            }
        }
        Ok(())
    };
    let result: Result<()> = tokio::select! {
        result = operation => result,
        signal = tokio::signal::ctrl_c() => signal.map_err(Into::into),
    };
    let leave = peer_channels::leave_webxdc_realtime(context, id).await;
    context.stop_io().await;
    result?;
    leave?;
    Ok(())
}
