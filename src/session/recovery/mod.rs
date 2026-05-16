//! Per-agent restart recovery dispatch.
//!
//! Each coding agent has its own restart quirks: where its transcript lives,
//! what "oversized" means for its parser, how to fall back when the transcript
//! is missing. Rather than baking Claude-specific assumptions into the session
//! restart path, this module exposes a [`HarnessRecovery`] trait and dispatches
//! by `tool: &str` (matching the existing per-agent contract used by
//! [`crate::tmux::status_detection`]).
//!
//! Recovery aggressiveness is user-tunable via [`RecoveryMode`]: `Strict`
//! (default) never mutates transcript bytes and falls back to a fresh launch
//! when `--resume` would otherwise fail; `Cascade` preserves more conversation
//! history by trimming oversized transcripts and restoring from archive;
//! `Off` skips recovery entirely.
//!
//! The first implementation is [`claude::ClaudeRecovery`]. Other agents (codex,
//! opencode, gemini) can ship their own implementations without touching the
//! restart codepath in `instance.rs`.

use serde::{Deserialize, Serialize};

pub mod claude;

pub use claude::RecoveryOutcome;

/// User-facing recovery aggressiveness. Read from `[session].recovery_mode` and
/// honored by every [`HarnessRecovery`] implementation.
///
/// Defaults to [`RecoveryMode::Strict`] following upstream review feedback that
/// transcript-byte mutation at restart should be opt-in, not default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecoveryMode {
    /// Never mutate transcript bytes. If the transcript is oversized or
    /// missing, return [`RecoveryOutcome::StrictSkipped`] so the caller can
    /// fall back to a fresh launch (clears `agent_session_id`).
    #[default]
    Strict,
    /// Trim oversized transcripts in place and restore from thrash archives
    /// when the live transcript is missing. Preserves more conversation
    /// history at the cost of mutating on-disk state at restart time.
    Cascade,
    /// Skip recovery entirely. Always clear the session id at restart so the
    /// agent launches fresh without `--resume`.
    Off,
}

/// Per-agent transcript/state recovery applied at restart time.
///
/// Implementations are zero-sized dispatch types; state lives in the agent's
/// on-disk artifacts (transcript files, archives) and is rediscovered each
/// call. Implementations should be conservative: prefer
/// [`RecoveryOutcome::NotApplicable`] over panicking when inputs are malformed
/// or unsupported. Filesystem errors during an active restoration are the only
/// case that warrants returning `Err`.
pub trait HarnessRecovery: Send + Sync {
    /// Attempt to recover the transcript/state for `sid` rooted at
    /// `project_path`, honoring `mode`. Returns the cascade outcome so the
    /// caller can log it and decide on follow-up behavior (e.g. fresh-launch
    /// fallback when [`RecoveryOutcome::NoArchiveFreshLaunch`] or
    /// [`RecoveryOutcome::StrictSkipped`]).
    fn recover(
        &self,
        sid: &str,
        project_path: &str,
        mode: RecoveryMode,
    ) -> anyhow::Result<RecoveryOutcome>;
}

/// Look up the recovery implementation for a given agent tool string. Returns
/// `None` when the agent does not (yet) ship its own recovery; callers should
/// treat that as a no-op and let the existing restart path run unchanged.
pub fn for_tool(tool: &str) -> Option<&'static dyn HarnessRecovery> {
    match tool {
        "claude" => Some(&claude::ClaudeRecovery),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_strict() {
        assert_eq!(RecoveryMode::default(), RecoveryMode::Strict);
    }

    #[test]
    fn serde_round_trip_lowercase() {
        for mode in [
            RecoveryMode::Strict,
            RecoveryMode::Cascade,
            RecoveryMode::Off,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let expected = match mode {
                RecoveryMode::Strict => "\"strict\"",
                RecoveryMode::Cascade => "\"cascade\"",
                RecoveryMode::Off => "\"off\"",
            };
            assert_eq!(json, expected);
            let back: RecoveryMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, mode);
        }
    }
}
