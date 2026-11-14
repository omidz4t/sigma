//! Owner model selection and sanitized provider-local catalogs.
use crate::{
    Result,
    output::{self, Value},
};
use deltachat::webxdc::StatusUpdateItem;
use std::path::{Path, PathBuf};
#[derive(Debug, PartialEq)]
pub(crate) struct Model {
    pub(crate) id: String,
    pub(crate) name: String,
}
pub(crate) struct Catalog {
    pub(crate) models: Vec<Model>,
    pub(crate) fetched_at: String,
}
fn valid(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._:-".contains(&b))
}
pub(crate) fn parse_catalog(provider: &str, raw: &str) -> Result<Catalog> {
    let item = StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    let v = &item.payload;
    let mut models = vec![];
    match provider {
        "codex" => {
            for model in v["models"]
                .as_array()
                .ok_or("invalid Codex model catalog")?
            {
                if model["visibility"].as_str() == Some("hide") {
                    continue;
                }
                if let Some(id) = model["slug"].as_str().filter(|id| valid(id)) {
                    models.push(Model {
                        id: id.into(),
                        name: model["display_name"]
                            .as_str()
                            .unwrap_or(id)
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(100)
                            .collect(),
                    });
                }
            }
        }
        "grok" => {
            for (id, model) in v["models"]
                .as_object()
                .ok_or("invalid Grok model catalog")?
            {
                if valid(id) {
                    models.push(Model {
                        id: id.clone(),
                        name: model["info"]["name"]
                            .as_str()
                            .unwrap_or(id)
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(100)
                            .collect(),
                    });
                }
            }
        }
        _ => return Err("model management supports native Codex and Grok agents only".into()),
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models.dedup_by(|a, b| a.id == b.id);
    if models.is_empty() {
        return Err("provider model catalog is empty".into());
    }
    Ok(Catalog {
        models,
        fetched_at: v["fetched_at"]
            .as_str()
            .unwrap_or("unknown")
            .chars()
            .filter(|c| !c.is_control())
            .take(100)
            .collect(),
    })
}
pub(crate) fn catalog(provider: &str) -> Result<Catalog> {
    let (env, folder) = match provider {
        "codex" => ("CODEX_HOME", ".codex"),
        "grok" => ("GROK_HOME", ".grok"),
        _ => return Err("model management supports native Codex and Grok agents only".into()),
    };
    let home = std::env::var_os(env)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(folder)))
        .ok_or("provider home is unavailable")?;
    use std::io::Read;
    let mut raw = Vec::new();
    std::fs::File::open(home.join("models_cache.json")).map_err(|_| "provider model catalog unavailable; run/login to the provider CLI to populate it (Grok: grok models)")?.take(4 * 1024 * 1024 + 1).read_to_end(&mut raw)?;
    if raw.len() > 4 * 1024 * 1024 {
        return Err("provider model catalog exceeds 4 MiB".into());
    }
    parse_catalog(provider, std::str::from_utf8(&raw)?)
}
pub(crate) fn selected(dir: &Path, fallback: Option<&str>) -> Result<Option<String>> {
    let raw = match std::fs::read_to_string(dir.join("model.json")) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(fallback.map(str::to_owned));
        }
        Err(e) => return Err(e.into()),
    };
    let item = StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    match item.payload.get("model") {
        Some(v) if v.is_null() => Ok(None),
        Some(v) => Ok(Some(
            v.as_str()
                .filter(|id| valid(id))
                .ok_or("invalid saved model ID")?
                .into(),
        )),
        None => Err("missing model setting".into()),
    }
}
fn save(dir: &Path, model: Option<&str>) -> Result<()> {
    let raw = output::encode(&output::object(vec![(
        "model",
        model.map(Value::from).unwrap_or(Value::Null),
    )]));
    let temp = dir.join(format!("model.{}.tmp", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&temp)?;
    file.write_all(raw.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(temp, dir.join("model.json"))?;
    Ok(())
}
pub(crate) fn initialize(dir: &Path, model: Option<&str>) -> Result<()> {
    if !dir.join("model.json").try_exists()? {
        save(dir, model)?;
    }
    Ok(())
}
pub(crate) fn set_from_catalog(dir: &Path, model: &str, catalog: &Catalog) -> Result<()> {
    if model == "default" {
        return save(dir, None);
    }
    if !valid(model) || !catalog.models.iter().any(|m| m.id == model) {
        return Err(
            "model is not in the provider's local catalog; use /models or agent-models to list IDs"
                .into(),
        );
    }
    save(dir, Some(model))
}
pub(crate) fn control(dir: &Path, provider: &str, native: bool, args: &[&str]) -> Result<String> {
    if !native || !matches!(provider, "codex" | "grok") {
        return Err("model management supports native Codex and Grok agents only; custom adapters manage their own models".into());
    }
    if args == ["list"] {
        let catalog = catalog(provider)?;
        return Ok(output::encode(&output::object(vec![
            ("ok", true.into()),
            ("provider", provider.into()),
            ("source", "provider_local_cache".into()),
            ("fetched_at", catalog.fetched_at.into()),
            (
                "models",
                Value::Array(
                    catalog
                        .models
                        .into_iter()
                        .map(|model| {
                            output::object(vec![
                                ("id", model.id.into()),
                                ("name", model.name.into()),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])));
    }
    match args {
        ["get"] => {}
        ["set", "default"] => save(dir, None)?,
        ["set", id] => set_from_catalog(dir, id, &catalog(provider)?)?,
        _ => return Err("expected list, get or set MODEL_ID/default".into()),
    }
    Ok(output::encode(&output::object(vec![
        ("ok", true.into()),
        ("provider", provider.into()),
        (
            "model",
            selected(dir, None)?.map(Value::from).unwrap_or(Value::Null),
        ),
        (
            "effective",
            "next provider turn; existing sessions retained".into(),
        ),
    ])))
}
pub(crate) fn chat(dir: &Path, provider: &str, native: bool, text: &str) -> String {
    let parts = text.split_whitespace().collect::<Vec<_>>();
    let operation = match parts.as_slice() {
        ["/models"] => vec!["list"],
        ["/model"] => vec!["get"],
        ["/model", id] => vec!["set", *id],
        _ => return "Use /models, /model, or /model MODEL_ID (or default).".into(),
    };
    let raw = match control(dir, provider, native, &operation) {
        Ok(raw) => raw,
        Err(error) => return format!("Model unchanged: {error}"),
    };
    let item = match raw.parse() {
        Ok(payload) => StatusUpdateItem {
            payload,
            ..Default::default()
        },
        Err(_) => return "Model response unavailable.".into(),
    };
    let v = &item.payload;
    if operation == ["list"] {
        let mut result = format!(
            "{provider} models — local provider catalog, fetched {}. Availability may have changed.\n",
            v["fetched_at"].as_str().unwrap_or("unknown")
        );
        for model in v["models"].as_array().into_iter().flatten() {
            let line = format!(
                "- {}: {}\n",
                model["id"].as_str().unwrap_or(""),
                model["name"].as_str().unwrap_or("")
            );
            if result.len() + line.len() > 24000 {
                result.push_str("More entries available via agent-models CLI.\n");
                break;
            }
            result.push_str(&line);
        }
        result.push_str("Use /model MODEL_ID to change this bot's model, or /model default for the provider CLI default.");
        return result;
    }
    format!(
        "{} model for this bot: {}. Changes apply on the next provider turn across this bot's chats; conversation history is kept.",
        if operation == ["get"] {
            "Selected"
        } else {
            "Saved"
        },
        v["model"]
            .as_str()
            .unwrap_or("provider CLI configured default (exact ID not pinned)")
    )
}
