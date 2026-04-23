//! `agent-of-empires session` subcommands implementation

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::session::{GroupTree, Instance, Storage};

/// Refuse to act on archived sessions. Used by `restart` to honor the
/// "archived = sunk; do not touch without explicit unarchive" invariant.
/// Data-proven failure mode: without this guard, cx account-swap teardown
/// and aoe-restart-all.sh both iterate every live tmux session and call
/// `aoe session restart $title` for each — archived sessions were being
/// restarted (and sent a wake-up prompt) despite the user sinking them.
fn ensure_not_archived(inst: &Instance, identifier: &str) -> Result<()> {
    if inst.archived_at.is_some() {
        bail!(
            "Session '{}' is archived — refusing to restart. Run `aoe session unarchive {}` first.",
            inst.title,
            identifier
        );
    }
    Ok(())
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// Start a session's tmux process
    Start(SessionIdArgs),

    /// Stop session process
    Stop(SessionIdArgs),

    /// Restart session
    Restart(SessionIdArgs),

    /// Attach to session interactively
    Attach(SessionIdArgs),

    /// Show session details
    Show(ShowArgs),

    /// Rename a session
    Rename(RenameArgs),

    /// Capture tmux pane output
    Capture(CaptureArgs),

    /// Auto-detect current session
    Current(CurrentArgs),

    /// Archive a session (sinks it to the bottom of the Attention sort,
    /// rendered in italic+dim; remains visible).
    Archive(SessionIdArgs),

    /// Unarchive a session (clears archived_at).
    Unarchive(SessionIdArgs),

    /// Favorite a session. While favorited AND in a "needs help" status
    /// (Waiting, Error, Idle, Unknown), it pins to the top of the Attention
    /// sort above all non-favorited peers. Rendered bold + underlined with
    /// a "* " prefix (ASCII, no emoji — avoids wide-width rendering
    /// artifacts on narrow iOS terminals). Opposite of archive.
    Favorite(SessionIdArgs),

    /// Unfavorite a session (clears favorited_at).
    Unfavorite(SessionIdArgs),

    /// Snooze a session (temporary archive). Sinks it to the bottom of
    /// the Attention sort and renders it italic+dim with a `z ` prefix
    /// and a remaining-time readout. Wakes automatically when the timer
    /// expires. Default duration is the profile's
    /// `session.snooze_duration_minutes` (default 30); override per-call
    /// with `--minutes`.
    Snooze(SnoozeArgs),

    /// Unsnooze a session (clears snoozed_until — wakes it immediately).
    Unsnooze(SessionIdArgs),
}

#[derive(Args)]
pub struct SessionIdArgs {
    /// Session ID or title
    identifier: String,
}

#[derive(Args)]
pub struct SnoozeArgs {
    /// Session ID or title
    identifier: String,

    /// Override snooze duration in minutes (1-1440). If omitted, uses the
    /// profile's configured `session.snooze_duration_minutes` (default 30).
    #[arg(short = 'm', long)]
    minutes: Option<u64>,
}

#[derive(Args)]
pub struct RenameArgs {
    /// Session ID or title (optional, auto-detects in tmux)
    identifier: Option<String>,

    /// New title for the session
    #[arg(short, long)]
    title: Option<String>,

    /// New group for the session (empty string to ungroup)
    #[arg(short, long)]
    group: Option<String>,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Session ID or title (optional, auto-detects in tmux)
    identifier: Option<String>,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct CaptureArgs {
    /// Session ID or title (auto-detects in tmux if omitted)
    identifier: Option<String>,

    /// Number of lines to capture
    #[arg(short = 'n', long, default_value = "50")]
    lines: usize,

    /// Strip ANSI escape codes
    #[arg(long)]
    strip_ansi: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct CurrentArgs {
    /// Just session name (for scripting)
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct CaptureOutput {
    id: String,
    title: String,
    status: String,
    tool: String,
    content: String,
    lines: usize,
}

#[derive(Serialize)]
struct SessionDetails {
    id: String,
    title: String,
    path: String,
    group: String,
    tool: String,
    command: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_session_id: Option<String>,
    profile: String,
}

pub async fn run(profile: &str, command: SessionCommands) -> Result<()> {
    match command {
        SessionCommands::Start(args) => start_session(profile, args).await,
        SessionCommands::Stop(args) => stop_session(profile, args).await,
        SessionCommands::Restart(args) => restart_session(profile, args).await,
        SessionCommands::Attach(args) => attach_session(profile, args).await,
        SessionCommands::Show(args) => show_session(profile, args).await,
        SessionCommands::Capture(args) => capture_session(profile, args).await,
        SessionCommands::Rename(args) => rename_session(profile, args).await,
        SessionCommands::Current(args) => current_session(args).await,
        SessionCommands::Archive(args) => set_session_archived(profile, args, true).await,
        SessionCommands::Unarchive(args) => set_session_archived(profile, args, false).await,
        SessionCommands::Favorite(args) => set_session_favorited(profile, args, true).await,
        SessionCommands::Unfavorite(args) => set_session_favorited(profile, args, false).await,
        SessionCommands::Snooze(args) => snooze_session(profile, args).await,
        SessionCommands::Unsnooze(args) => unsnooze_session(profile, args).await,
    }
}

async fn snooze_session(profile: &str, args: SnoozeArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let config = crate::session::profile_config::resolve_config(profile)?;

    // `--minutes` overrides the profile default; otherwise use the
    // configured `snooze_duration_minutes`. Validate either way so the
    // on-disk config can't sneak in an out-of-range value.
    let raw_minutes = args
        .minutes
        .unwrap_or(config.session.snooze_duration_minutes as u64);
    crate::session::validate_snooze_duration(raw_minutes).map_err(|e| anyhow::anyhow!("{}", e))?;
    let minutes = raw_minutes as u32;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    instances[idx].snooze(minutes);
    let title = instances[idx].title.clone();

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    println!("✓ Snoozed for {}m: {}", minutes, title);
    Ok(())
}

async fn unsnooze_session(profile: &str, args: SessionIdArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    instances[idx].unsnooze();
    let title = instances[idx].title.clone();

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    println!("✓ Woke: {}", title);
    Ok(())
}

async fn set_session_favorited(profile: &str, args: SessionIdArgs, favorited: bool) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    if favorited {
        instances[idx].favorite();
    } else {
        instances[idx].unfavorite();
    }
    let title = instances[idx].title.clone();

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    let verb = if favorited {
        "Favorited"
    } else {
        "Unfavorited"
    };
    println!("✓ {} session: {}", verb, title);
    Ok(())
}

async fn set_session_archived(profile: &str, args: SessionIdArgs, archived: bool) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    if archived {
        instances[idx].archive();
    } else {
        instances[idx].unarchive();
    }
    let title = instances[idx].title.clone();

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    let verb = if archived { "Archived" } else { "Unarchived" };
    println!("✓ {} session: {}", verb, title);
    Ok(())
}

async fn start_session(profile: &str, args: SessionIdArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    instances[idx].start_with_size(crate::terminal::get_size())?;
    let title = instances[idx].title.clone();

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    println!("✓ Started session: {}", title);
    Ok(())
}

async fn stop_session(profile: &str, args: SessionIdArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let inst = super::resolve_session(&args.identifier, &instances)?;
    let session_id = inst.id.clone();
    let title = inst.title.clone();
    let tmux_session = crate::tmux::Session::new(&inst.id, &inst.title)?;
    let was_running = tmux_session.exists();
    let had_container = inst.is_sandboxed()
        && crate::containers::DockerContainer::from_session_id(&inst.id)
            .is_running()
            .unwrap_or(false);

    if !was_running && !had_container {
        println!("Session is not running: {}", title);
        return Ok(());
    }

    inst.stop()?;

    // Persist Stopped status to disk so it survives TUI restarts
    if let Some(stored) = instances.iter_mut().find(|i| i.id == session_id) {
        stored.status = crate::session::Status::Stopped;
    }
    let group_tree = crate::session::GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    if had_container {
        println!("✓ Stopped session and container: {}", title);
    } else {
        println!("✓ Stopped session: {}", title);
    }

    Ok(())
}

async fn restart_session(profile: &str, args: SessionIdArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let idx = instances
        .iter()
        .position(|i| {
            i.id == args.identifier
                || i.id.starts_with(&args.identifier)
                || i.title == args.identifier
        })
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", args.identifier))?;

    ensure_not_archived(&instances[idx], &args.identifier)?;

    instances[idx].restart_with_size(crate::terminal::get_size())?;
    let title = instances[idx].title.clone();
    let session_id = instances[idx].id.clone();
    let tool = instances[idx].tool.clone();

    // Wait for the agent CLI to render its prompt before injecting input.
    // Without this, keystrokes land in the shell before claude/opencode
    // takes over the TTY and get lost.
    std::thread::sleep(std::time::Duration::from_millis(2000));

    let tmux_session = crate::tmux::Session::new(&session_id, &title)?;
    if tmux_session.exists() {
        let delay = crate::agents::send_keys_enter_delay(&tool);
        let wake_msg = "wake up — pick up what you were doing";
        match tmux_session.send_keys_with_delay(wake_msg, delay) {
            Ok(()) => {
                if let Some(inst) = instances.iter_mut().find(|i| i.id == session_id) {
                    inst.touch_last_accessed();
                }
            }
            Err(e) => {
                eprintln!("Warning: failed to send wake-up message: {}", e);
            }
        }
    }

    let group_tree = GroupTree::new_with_groups(&instances, &groups);
    storage.save_with_groups(&instances, &group_tree)?;

    println!("✓ Restarted session: {}", title);
    Ok(())
}

async fn attach_session(profile: &str, args: SessionIdArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (instances, _) = storage.load_with_groups()?;

    let inst = super::resolve_session(&args.identifier, &instances)?;
    let tmux_session = crate::tmux::Session::new(&inst.id, &inst.title)?;

    if !tmux_session.exists() {
        bail!(
            "Session is not running. Start it first with: agent-of-empires session start {}",
            args.identifier
        );
    }

    tmux_session.attach()?;
    Ok(())
}

async fn show_session(profile: &str, args: ShowArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (instances, _) = storage.load_with_groups()?;

    let mut inst = if let Some(id) = &args.identifier {
        super::resolve_session(id, &instances)?.clone()
    } else {
        // Auto-detect from tmux
        let current_session = std::env::var("TMUX_PANE")
            .ok()
            .and_then(|_| crate::tmux::get_current_session_name());

        if let Some(session_name) = current_session {
            instances
                .iter()
                .find(|i| {
                    let tmux_name = crate::tmux::Session::generate_name(&i.id, &i.title);
                    tmux_name == session_name
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("Current tmux session is not an Agent of Empires session")
                })?
                .clone()
        } else {
            bail!("Not in a tmux session. Specify a session ID or run inside tmux.");
        }
    };

    // Refresh status from tmux so the output reflects current state
    // rather than the stale persisted value.
    crate::tmux::refresh_session_cache();
    inst.update_status();

    if args.json {
        let details = SessionDetails {
            id: inst.id.clone(),
            title: inst.title.clone(),
            path: inst.project_path.clone(),
            group: inst.group_path.clone(),
            tool: inst.tool.clone(),
            command: inst.command.clone(),
            status: format!("{:?}", inst.status).to_lowercase(),
            parent_session_id: inst.parent_session_id.clone(),
            profile: storage.profile().to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&details)?);
    } else {
        println!("Session: {}", inst.title);
        println!("  ID:      {}", inst.id);
        println!("  Path:    {}", inst.project_path);
        println!("  Group:   {}", inst.group_path);
        println!("  Tool:    {}", inst.tool);
        println!("  Command: {}", inst.command);
        println!("  Status:  {:?}", inst.status);
        println!("  Profile: {}", storage.profile());
        if let Some(parent_id) = &inst.parent_session_id {
            println!("  Parent:  {}", parent_id);
        }
    }

    Ok(())
}

async fn capture_session(profile: &str, args: CaptureArgs) -> Result<()> {
    let storage = Storage::new(profile)?;
    let (instances, _) = storage.load_with_groups()?;

    let inst = if let Some(id) = &args.identifier {
        super::resolve_session(id, &instances)?
    } else {
        let current_session = std::env::var("TMUX_PANE")
            .ok()
            .and_then(|_| crate::tmux::get_current_session_name());

        if let Some(session_name) = current_session {
            instances
                .iter()
                .find(|i| {
                    let tmux_name = crate::tmux::Session::generate_name(&i.id, &i.title);
                    tmux_name == session_name
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("Current tmux session is not an Agent of Empires session")
                })?
        } else {
            bail!("Not in a tmux session. Specify a session ID or run inside tmux.");
        }
    };

    let tmux_session = crate::tmux::Session::new(&inst.id, &inst.title)?;

    let (content, status) = if !tmux_session.exists() {
        (String::new(), "stopped".to_string())
    } else {
        let raw = tmux_session.capture_pane(args.lines)?;
        let content = if args.strip_ansi {
            crate::tmux::utils::strip_ansi(&raw)
        } else {
            raw
        };
        let status = crate::hooks::read_hook_status(&inst.id)
            .unwrap_or_else(|| tmux_session.detect_status(&inst.tool).unwrap_or_default());
        (content, format!("{:?}", status).to_lowercase())
    };

    if args.json {
        let output = CaptureOutput {
            id: inst.id.clone(),
            title: inst.title.clone(),
            status,
            tool: inst.tool.clone(),
            content,
            lines: args.lines,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print!("{}", content);
    }

    Ok(())
}

async fn rename_session(profile: &str, args: RenameArgs) -> Result<()> {
    if args.title.is_none() && args.group.is_none() {
        bail!("At least one of --title or --group must be specified");
    }

    let storage = Storage::new(profile)?;
    let (mut instances, groups) = storage.load_with_groups()?;

    let inst = if let Some(id) = &args.identifier {
        super::resolve_session(id, &instances)?
    } else {
        // Auto-detect from tmux
        let current_session = std::env::var("TMUX_PANE")
            .ok()
            .and_then(|_| crate::tmux::get_current_session_name());

        if let Some(session_name) = current_session {
            instances
                .iter()
                .find(|i| {
                    let tmux_name = crate::tmux::Session::generate_name(&i.id, &i.title);
                    tmux_name == session_name
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("Current tmux session is not an Agent of Empires session")
                })?
        } else {
            bail!("Not in a tmux session. Specify a session ID or run inside tmux.");
        }
    };

    let id = inst.id.clone();
    let old_title = inst.title.clone();

    let effective_title = args.title.unwrap_or(old_title.clone());
    let effective_title = effective_title.trim().to_string();

    let idx = instances
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    // Rename tmux session if title changed
    if instances[idx].title != effective_title {
        let tmux_session = crate::tmux::Session::new(&id, &instances[idx].title)?;
        if tmux_session.exists() {
            let new_tmux_name = crate::tmux::Session::generate_name(&id, &effective_title);
            if let Err(e) = tmux_session.rename(&new_tmux_name) {
                eprintln!("Warning: failed to rename tmux session: {}", e);
            } else {
                crate::tmux::refresh_session_cache();
            }
        }
    }

    instances[idx].title = effective_title.clone();

    if let Some(group) = args.group {
        instances[idx].group_path = group.trim().to_string();
    }

    let mut group_tree = GroupTree::new_with_groups(&instances, &groups);
    if !instances[idx].group_path.is_empty() {
        group_tree.create_group(&instances[idx].group_path);
    }
    storage.save_with_groups(&instances, &group_tree)?;

    if old_title != effective_title {
        println!("✓ Renamed session: {} → {}", old_title, effective_title);
    } else {
        println!("✓ Updated session: {}", effective_title);
    }

    Ok(())
}

async fn current_session(args: CurrentArgs) -> Result<()> {
    // Auto-detect profile and session from tmux
    let current_session = std::env::var("TMUX_PANE")
        .ok()
        .and_then(|_| crate::tmux::get_current_session_name());

    let session_name = current_session.ok_or_else(|| anyhow::anyhow!("Not in a tmux session"))?;

    // Search all profiles for this session
    let profiles = crate::session::list_profiles()?;

    for profile_name in &profiles {
        if let Ok(storage) = Storage::new(profile_name) {
            if let Ok((instances, _)) = storage.load_with_groups() {
                if let Some(inst) = instances.iter().find(|i| {
                    let tmux_name = crate::tmux::Session::generate_name(&i.id, &i.title);
                    tmux_name == session_name
                }) {
                    if args.json {
                        #[derive(Serialize)]
                        struct CurrentInfo {
                            session: String,
                            profile: String,
                            id: String,
                        }
                        let info = CurrentInfo {
                            session: inst.title.clone(),
                            profile: profile_name.clone(),
                            id: inst.id.clone(),
                        };
                        println!("{}", serde_json::to_string_pretty(&info)?);
                    } else if args.quiet {
                        println!("{}", inst.title);
                    } else {
                        println!("Session: {}", inst.title);
                        println!("Profile: {}", profile_name);
                        println!("ID:      {}", inst.id);
                    }
                    return Ok(());
                }
            }
        }
    }

    bail!("Current tmux session is not an Agent of Empires session")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_not_archived_allows_live_session() {
        let inst = Instance::new("forit-Avatics", "/tmp/x");
        assert!(ensure_not_archived(&inst, "forit-Avatics").is_ok());
    }

    #[test]
    fn ensure_not_archived_refuses_archived_session() {
        let mut inst = Instance::new("forit-Avatics", "/tmp/x");
        inst.archive();
        let err = ensure_not_archived(&inst, "forit-Avatics").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("archived"), "expected 'archived' in: {msg}");
        assert!(msg.contains("forit-Avatics"), "expected title in: {msg}");
        assert!(
            msg.contains("unarchive"),
            "expected recovery hint in: {msg}"
        );
    }

    #[test]
    fn ensure_not_archived_allows_after_unarchive() {
        let mut inst = Instance::new("forit-Avatics", "/tmp/x");
        inst.archive();
        inst.unarchive();
        assert!(ensure_not_archived(&inst, "forit-Avatics").is_ok());
    }
}
