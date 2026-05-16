//! Per-agent output-pattern signal detection.
//!
//! Sibling of `status_detection`: where status answers "what is the agent
//! doing right now" with a single value from
//! `{Idle, Running, Waiting, Done}`, signals answer "what noteworthy events
//! just happened" with zero or more typed [`AgentSignal`]s. The two are kept
//! separate so a single rate-limit notice doesn't have to be folded into the
//! status enum (which would also suppress a coincident auth-required event).
//!
//! Dispatched through the same `AgentDef` registry as `detect_status`. Most
//! agents register [`no_signals`] until someone wires patterns for them; Claude
//! ships the initial set.
//!
//! Design notes: docs/plans/2026-05-15-per-agent-signal-detection-design.md.
//! Companion design: docs/plans/2026-05-15-modular-harness-recovery-design.md
//! (the cold-restart leg of the same per-agent contract).

use serde::{Deserialize, Serialize};

use super::utils::strip_ansi;

/// A noteworthy event extracted from pane content. Zero or more per poll.
///
/// Variants are non-exhaustive in practice (new ones will be added as new
/// agent quirks are observed); callers should treat unknown shapes as a
/// no-op rather than relying on a closed set.
///
/// Serialized as part of `session::Instance.last_signals` so the TUI and
/// web dashboard can render badges without re-scanning pane content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentSignal {
    /// Agent reports its usage limit is exhausted and no further requests
    /// will succeed until a reset window passes. `reset_at` carries the
    /// agent-formatted reset string if one was parseable from the message
    /// (e.g. "11pm", "tomorrow at 3am"); always treat it as opaque display
    /// text, not a parsed timestamp.
    LimitsExhausted { reset_at: Option<String> },
    /// Agent requires the user to log in / re-authenticate before it can
    /// continue. Distinct from `LimitsExhausted`: auth blocks all turns,
    /// limits block only until reset.
    AuthRequired,
    /// Agent's context window is full or the in-pane prompt is reporting
    /// "context low / compact required". Lighter-weight than limits;
    /// usually self-recovers via autocompact.
    ContextFull,
    /// Soft rate-limit (throttle), in contrast with the hard
    /// [`LimitsExhausted`] block. `retry_after_seconds` is the agent's
    /// suggested wait if it surfaced one.
    RateLimited { retry_after_seconds: Option<u32> },
}

/// Dispatch by `tool` string to the per-agent detector registered on
/// [`crate::agents::AgentDef`]. Returns an empty vec for unknown tools or
/// agents with no registered detector (the shared [`no_signals`] no-op).
pub fn detect_signals_from_content(content: &str, tool: &str) -> Vec<AgentSignal> {
    let clean = strip_ansi(content);
    crate::agents::get_agent(tool)
        .map(|a| (a.detect_signals)(&clean))
        .unwrap_or_default()
}

/// Shared no-op detector for agents that have not had patterns wired yet.
/// Pointed at by every [`crate::agents::AgentDef`] entry by default so the
/// registry never carries a per-agent stub function with an empty body.
pub fn no_signals(_content: &str) -> Vec<AgentSignal> {
    Vec::new()
}

/// Claude Code signal patterns. Conservative on purpose: false positives
/// here mean the TUI red-blinks at the user mid-task, which is worse than
/// missing a signal on first try.
pub fn detect_claude_signals(content: &str) -> Vec<AgentSignal> {
    let mut out = Vec::new();

    // Limits exhausted. Claude renders this as a single line at the bottom
    // of the pane after the user submits a turn that exceeds their plan
    // budget. Two known shapes:
    //   "You've used all your Sonnet messages until 11pm. ..."
    //   "Claude usage limit reached. Your limit will reset at 11pm (UTC)."
    //
    // Keep the match anchored on phrases that are unique to the limit
    // notice (i.e. avoid generic "limit" matches that would fire on
    // ordinary tool error messages that mention "rate limit").
    let limits_phrases = [
        "you've used all your",
        "claude usage limit reached",
        "5-hour limit reached",
        "weekly usage limit",
    ];
    let lower = content.to_lowercase();
    if limits_phrases.iter().any(|p| lower.contains(p)) {
        out.push(AgentSignal::LimitsExhausted {
            reset_at: parse_claude_reset_time(content),
        });
    }

    // Auth required. Claude prints "Please log in" or "Invalid API key"
    // when its credentials are missing or rejected.
    if lower.contains("please log in")
        || lower.contains("invalid api key")
        || lower.contains("authentication required")
    {
        out.push(AgentSignal::AuthRequired);
    }

    out
}

/// Best-effort extraction of the "reset at <time>" fragment Claude appends
/// to its limit-exhausted line. Returns the substring between "reset at "
/// (or "until ") and the next sentence boundary; opaque display text only,
/// no timestamp parsing.
fn parse_claude_reset_time(content: &str) -> Option<String> {
    for needle in ["reset at ", "until ", "resets at "] {
        if let Some(start) = content.to_lowercase().find(needle) {
            let after = &content[start + needle.len()..];
            let end = after
                .find(['.', '\n', ')'])
                .unwrap_or(after.len().min(40));
            let candidate = after[..end].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_signals_on_empty() {
        assert!(detect_claude_signals("").is_empty());
    }

    #[test]
    fn no_signals_on_routine_running_pane() {
        let pane = "✶ Working… (5s · ↓ 142 tokens · esc to interrupt)\n> ";
        assert!(detect_claude_signals(pane).is_empty());
    }

    #[test]
    fn claude_limits_used_all_messages() {
        let pane = "You've used all your Sonnet messages until 11pm. \
                    Upgrade to Max for higher limits.";
        let signals = detect_claude_signals(pane);
        assert_eq!(signals.len(), 1, "got {:?}", signals);
        match &signals[0] {
            AgentSignal::LimitsExhausted { reset_at } => {
                assert_eq!(reset_at.as_deref(), Some("11pm"));
            }
            other => panic!("expected LimitsExhausted, got {:?}", other),
        }
    }

    #[test]
    fn claude_limits_reset_phrasing() {
        let pane = "Claude usage limit reached. Your limit will reset at 3am (UTC).";
        let signals = detect_claude_signals(pane);
        assert_eq!(signals.len(), 1);
        match &signals[0] {
            AgentSignal::LimitsExhausted { reset_at } => {
                assert_eq!(reset_at.as_deref(), Some("3am (UTC"));
            }
            other => panic!("expected LimitsExhausted, got {:?}", other),
        }
    }

    #[test]
    fn claude_5h_limit_phrasing() {
        let pane = "5-hour limit reached. Continue at 7pm.";
        let signals = detect_claude_signals(pane);
        assert!(matches!(
            signals.as_slice(),
            [AgentSignal::LimitsExhausted { .. }]
        ));
    }

    #[test]
    fn claude_auth_required() {
        let pane = "Please log in to continue.\n";
        let signals = detect_claude_signals(pane);
        assert_eq!(signals, vec![AgentSignal::AuthRequired]);
    }

    #[test]
    fn claude_invalid_api_key_is_auth_required() {
        let pane = "Error: Invalid API key. Please check your credentials.";
        let signals = detect_claude_signals(pane);
        assert!(signals.contains(&AgentSignal::AuthRequired));
    }

    #[test]
    fn claude_limits_and_auth_can_coexist() {
        let pane = "Authentication required\nYou've used all your Sonnet messages until 11pm.";
        let signals = detect_claude_signals(pane);
        assert_eq!(signals.len(), 2);
    }

    #[test]
    fn no_signals_handles_unknown_tool() {
        let out = detect_signals_from_content("anything", "totally-unknown-agent");
        assert!(out.is_empty());
    }

    #[test]
    fn no_signals_default_is_empty() {
        assert!(no_signals("Please log in. Limit reached.").is_empty());
    }

    #[test]
    fn parse_reset_time_until_form() {
        let s = parse_claude_reset_time("You've used all your stuff until 11pm.");
        assert_eq!(s.as_deref(), Some("11pm"));
    }

    #[test]
    fn parse_reset_time_missing() {
        let s = parse_claude_reset_time("nothing here");
        assert_eq!(s, None);
    }
}
