//! Snooze duration picker. Opens when the user presses `w`/`W` on a
//! non-snoozed session — single-key shortcuts so the choice is
//! one keystroke after the `w` that summoned it.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;

use super::DialogResult;
use crate::tui::styles::Theme;

/// Preset durations in minutes. Ordered shortest → longest so the digit
/// roughly tracks duration ascent.
const FIFTEEN_MIN: u32 = 15;
const THIRTY_MIN: u32 = 30;
const ONE_HOUR: u32 = 60;
const TWO_HOURS: u32 = 2 * 60;
const FOUR_HOURS: u32 = 4 * 60;
const EIGHT_HOURS: u32 = 8 * 60;
const ONE_DAY: u32 = 24 * 60;
const THREE_DAYS: u32 = 3 * 24 * 60;
const ONE_WEEK: u32 = 7 * 24 * 60;

/// Small picker. Hotkeys (digits ascend with duration):
///   `1` → 15 min
///   `2` → 30 min
///   `3` → 1 hr
///   `4` → 2 hr
///   `5` → 4 hr
///   `6` → 8 hr (workday)
///   `7` → 1 day
///   `8` → 3 days
///   `9` → 1 week
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
            KeyCode::Char('1') => DialogResult::Submit(FIFTEEN_MIN),
            KeyCode::Char('2') => DialogResult::Submit(THIRTY_MIN),
            KeyCode::Char('3') => DialogResult::Submit(ONE_HOUR),
            KeyCode::Char('4') => DialogResult::Submit(TWO_HOURS),
            KeyCode::Char('5') => DialogResult::Submit(FOUR_HOURS),
            KeyCode::Char('6') => DialogResult::Submit(EIGHT_HOURS),
            KeyCode::Char('7') => DialogResult::Submit(ONE_DAY),
            KeyCode::Char('8') => DialogResult::Submit(THREE_DAYS),
            KeyCode::Char('9') => DialogResult::Submit(ONE_WEEK),
            _ => DialogResult::Continue,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let dialog_area = super::centered_rect(area, 52, 15);
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
                Constraint::Length(1), // 1 — 15 min
                Constraint::Length(1), // 2 — 30 min
                Constraint::Length(1), // 3 — 1 hr
                Constraint::Length(1), // 4 — 2 hr
                Constraint::Length(1), // 5 — 4 hr
                Constraint::Length(1), // 6 — 8 hr
                Constraint::Length(1), // 7 — 1 day
                Constraint::Length(1), // 8 — 3 days
                Constraint::Length(1), // 9 — 1 week
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
        frame.render_widget(row("1", "15 min"), chunks[2]);
        frame.render_widget(row("2", "30 min"), chunks[3]);
        frame.render_widget(row("3", "1 hr"), chunks[4]);
        frame.render_widget(row("4", "2 hr"), chunks[5]);
        frame.render_widget(row("5", "4 hr"), chunks[6]);
        frame.render_widget(row("6", "8 hr (workday)"), chunks[7]);
        frame.render_widget(row("7", "1 day"), chunks[8]);
        frame.render_widget(row("8", "3 days"), chunks[9]);
        frame.render_widget(row("9", "1 week"), chunks[10]);
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
    fn ascending_digit_presets() {
        // Pin the full 1-9 mapping. Adding/changing a preset must
        // update both the dialog and this test in the same commit.
        let cases: &[(char, u32)] = &[
            ('1', 15),
            ('2', 30),
            ('3', 60),
            ('4', 120),
            ('5', 240),
            ('6', 480),
            ('7', 1440),
            ('8', 3 * 1440),
            ('9', 7 * 1440),
        ];
        for (digit, minutes) in cases {
            let mut d = SnoozeDurationDialog::new("sess");
            match d.handle_key(k(KeyCode::Char(*digit))) {
                DialogResult::Submit(m) => assert_eq!(m, *minutes, "digit {digit}"),
                _ => panic!("expected Submit({minutes}) for digit {digit}"),
            }
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
    fn zero_is_not_a_shortcut() {
        // Digits 1-9 are bound; 0 shouldn't submit anything.
        let mut d = SnoozeDurationDialog::new("sess");
        assert!(matches!(
            d.handle_key(k(KeyCode::Char('0'))),
            DialogResult::Continue
        ));
    }
}
