//! Snooze duration picker. Opens when the user presses `w`/`W` on a
//! non-snoozed session — three single-key shortcuts so the choice is
//! one keystroke after the `w` that summoned it.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;

use super::DialogResult;
use crate::tui::styles::Theme;

/// Duration in minutes used for each hotkey.
const THIRTY_MIN: u32 = 30;
const ONE_HOUR: u32 = 60;
const ONE_DAY: u32 = 24 * 60;

/// Small picker. Hotkeys:
///   `3` → 30 min
///   `6` → 1 hr
///   `2` → 24 hr
///   `Esc` / `q` → cancel
///
/// Submitted value is minutes; the caller passes it straight to
/// `Instance::snooze(minutes)`.
pub struct SnoozeDurationDialog {
    title: String,
}

impl SnoozeDurationDialog {
    pub fn new(session_title: &str) -> Self {
        Self {
            title: session_title.to_string(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DialogResult<u32> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => DialogResult::Cancel,
            KeyCode::Char('3') => DialogResult::Submit(THIRTY_MIN),
            KeyCode::Char('6') => DialogResult::Submit(ONE_HOUR),
            KeyCode::Char('2') => DialogResult::Submit(ONE_DAY),
            _ => DialogResult::Continue,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let dialog_area = super::centered_rect(area, 48, 9);
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.waiting))
            .title(" Snooze ")
            .title_style(Style::default().fg(theme.waiting).bold());

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // session title
                Constraint::Length(1), // spacer
                Constraint::Length(1), // 3 ...
                Constraint::Length(1), // 6 ...
                Constraint::Length(1), // 2 ...
            ])
            .split(inner);

        let subject = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{}  ", self.title),
                Style::default().fg(theme.text).bold(),
            ),
            Span::styled("— how long?", Style::default().fg(theme.dimmed)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(subject, chunks[0]);

        let key_style = Style::default().fg(theme.waiting).bold();
        let text_style = Style::default().fg(theme.text);
        let row = |k: &'static str, label: &'static str| {
            Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("[{}]", k), key_style),
                Span::raw("  "),
                Span::styled(label, text_style),
            ]))
        };
        frame.render_widget(row("3", "30 min"), chunks[2]);
        frame.render_widget(row("6", "1 hr"), chunks[3]);
        frame.render_widget(row("2", "24 hr"), chunks[4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn k(c: KeyCode) -> KeyEvent {
        KeyEvent::new(c, KeyModifiers::NONE)
    }

    #[test]
    fn three_submits_thirty() {
        let mut d = SnoozeDurationDialog::new("sess");
        match d.handle_key(k(KeyCode::Char('3'))) {
            DialogResult::Submit(m) => assert_eq!(m, 30),
            _ => panic!("expected Submit(30)"),
        }
    }

    #[test]
    fn six_submits_sixty() {
        let mut d = SnoozeDurationDialog::new("sess");
        match d.handle_key(k(KeyCode::Char('6'))) {
            DialogResult::Submit(m) => assert_eq!(m, 60),
            _ => panic!("expected Submit(60)"),
        }
    }

    #[test]
    fn two_submits_one_day() {
        let mut d = SnoozeDurationDialog::new("sess");
        match d.handle_key(k(KeyCode::Char('2'))) {
            DialogResult::Submit(m) => assert_eq!(m, 24 * 60),
            _ => panic!("expected Submit(1440)"),
        }
    }

    #[test]
    fn esc_cancels() {
        let mut d = SnoozeDurationDialog::new("sess");
        assert!(matches!(
            d.handle_key(k(KeyCode::Esc)),
            DialogResult::Cancel
        ));
    }

    #[test]
    fn q_cancels() {
        let mut d = SnoozeDurationDialog::new("sess");
        assert!(matches!(
            d.handle_key(k(KeyCode::Char('q'))),
            DialogResult::Cancel
        ));
    }

    #[test]
    fn unknown_continues() {
        let mut d = SnoozeDurationDialog::new("sess");
        assert!(matches!(
            d.handle_key(k(KeyCode::Char('x'))),
            DialogResult::Continue
        ));
    }

    #[test]
    fn four_is_not_a_shortcut() {
        // Only 3/6/2 are bound. Typing "4" shouldn't accidentally submit
        // something.
        let mut d = SnoozeDurationDialog::new("sess");
        assert!(matches!(
            d.handle_key(k(KeyCode::Char('4'))),
            DialogResult::Continue
        ));
    }
}
