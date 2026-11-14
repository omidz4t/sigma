//! Kernel-enforced Linux process isolation using the system bubblewrap executable.
use crate::Result;
use std::path::{Path, PathBuf};
fn private(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    if source.is_symlink() || target.is_symlink() {
        return Err("provider session migration refuses symlinks".into());
    }
    if source.is_dir() {
        private(target)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &target.join(entry.file_name()))?;
        }
    } else {
        std::fs::copy(source, target)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o600))?;
        }
    }
    Ok(())
}
fn find_session(source: &Path, target: &Path, id: &str) -> Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            continue;
        }
        let destination = target.join(entry.file_name());
        if entry.file_name().to_string_lossy().contains(id) {
            if !destination.exists() {
                if let Some(parent) = destination.parent() {
                    private(parent)?;
                }
                copy_tree(&entry.path(), &destination)?;
            }
        } else if entry.file_type()?.is_dir() {
            find_session(&entry.path(), &destination, id)?;
        }
    }
    Ok(())
}
pub(crate) fn provider_home(
    provider: &str,
    state: &Path,
    session: Option<&str>,
) -> Result<PathBuf> {
    let home = state.join("provider-home");
    if home.is_symlink() {
        return Err("provider home must not be a symlink".into());
    }
    let initialized = state.join("provider-home-initialized");
    if initialized.is_file() {
        return Ok(home);
    }
    private(&home)?;
    let var = match provider {
        "codex" => "CODEX_HOME",
        "grok" => "GROK_HOME",
        _ => return Ok(home),
    };
    let original = std::env::var_os(var)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(format!(".{provider}"))))
        .ok_or("provider home missing")?;
    // Seed once. Never write host snapshots through paths after a provider has owned them.
    for name in [
        "auth.json",
        "config.toml",
        "models_cache.json",
        "settings_cache.json",
        "agent_id",
        ".metadata_version",
    ] {
        let source = original.join(name);
        if source.is_file() && !source.is_symlink() {
            copy_tree(&source, &home.join(name))?;
        }
    }
    if let Some(id) = session {
        find_session(&original.join("sessions"), &home.join("sessions"), id)?;
    }
    std::fs::write(initialized, "provider runtime seeded\n")?;
    Ok(home)
}
pub(crate) fn command(
    executable: &Path,
    environment: &[(String, String)],
    provider: Option<&str>,
    session: Option<&str>,
) -> Result<tokio::process::Command> {
    let Some((_, root)) = environment
        .iter()
        .find(|(k, _)| k == "SIGMA_AGENT_SANDBOX_ROOT")
    else {
        return Ok(tokio::process::Command::new(executable));
    };
    if !cfg!(target_os = "linux") {
        return Err("secure agent execution currently requires Linux bubblewrap; refusing an unisolated process".into());
    }
    let bwrap = Path::new("/usr/bin/bwrap");
    if !bwrap.is_file() {
        return Err("secure agent execution requires system bubblewrap at /usr/bin/bwrap; refusing an unisolated process".into());
    }
    let field = |key: &str| -> Result<PathBuf> {
        Ok(PathBuf::from(
            environment
                .iter()
                .find(|(k, _)| k == key)
                .ok_or("missing agent isolation path")?
                .1
                .clone(),
        ))
    };
    let outbox = std::fs::canonicalize(field("SIGMA_AGENT_OUTBOX")?)?;
    let state = std::fs::canonicalize(field("SIGMA_AGENT_STATE_DIR")?)?;
    let root = std::fs::canonicalize(root)?;
    if !state.starts_with(&root) || !outbox.starts_with(&state) {
        return Err("invalid agent isolation layout".into());
    }
    let mut command = tokio::process::Command::new(bwrap);
    command.args([
        "--die-with-parent",
        "--new-session",
        "--unshare-user",
        "--unshare-pid",
        "--unshare-ipc",
        "--unshare-uts",
        "--cap-drop",
        "ALL",
        "--ro-bind",
        "/",
        "/",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--tmpfs",
        "/run",
    ]);
    // Hide the selected account, the legacy account and the named-account registry.
    let mut hidden = vec![root];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        hidden.extend([
            home.join(".config/sigma"),
            home.join(".config/sigma-accounts"),
        ]);
    }
    hidden.sort_by_key(|p| p.components().count());
    let mut covered: Vec<PathBuf> = vec![];
    for path in hidden {
        if path.is_dir() && !covered.iter().any(|p| path.starts_with(p)) {
            command.arg("--tmpfs").arg(&path);
            covered.push(path);
        }
    }
    command
        .arg("--bind")
        .arg(&outbox)
        .arg(&outbox)
        .arg("--chdir")
        .arg(&outbox);
    if let Some(provider) = provider {
        let home = provider_home(provider, &state, session)?;
        command.arg("--bind").arg(&home).arg(&home);
        command.env(
            if provider == "codex" {
                "CODEX_HOME"
            } else {
                "GROK_HOME"
            },
            &home,
        );
    }
    command.arg("--").arg(executable);
    Ok(command)
}
