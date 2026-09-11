//! Ratatui-based Live Trace Tree Viewer and TUI Inspector for `tgs trace --live` / `tgs trace --tui`.

use super::journal::TraceJournal;
use crate::error::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

pub fn run_trace_tui(journal: &TraceJournal) -> Result<()> {
    enable_raw_mode().ok();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).ok();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        crate::error::TagisanError::Execution(format!("Failed to initialize TUI terminal: {e}"))
    })?;

    let res = run_tui_loop(&mut terminal, journal);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    res
}

fn run_tui_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    journal: &TraceJournal,
) -> Result<()> {
    let mut events = journal.read_all().unwrap_or_default();
    let mut list_state = ListState::default();
    if !events.is_empty() {
        list_state.select(Some(events.len().saturating_sub(1)));
    }

    loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),      // Header
                        Constraint::Min(10),        // Main body (split left/right)
                        Constraint::Length(3),      // Footer / Help
                    ])
                    .split(size);

                // 1. Header
                let header = Paragraph::new(Line::from(vec![
                    Span::styled(
                        " 🔭 TAGISAN ENTERPRISE TELEMETRY MESH ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("Journal: {:?} ({} spans recorded)", journal.path(), events.len()),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
                f.render_widget(header, chunks[0]);

                // 2. Body split (List on Left, Details on Right)
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[1]);

                // Left: Event list
                let items: Vec<ListItem> = events
                    .iter()
                    .enumerate()
                    .map(|(i, ev)| {
                        let span_color = match ev.span_name.as_str() {
                            "dag.node.execute" => Color::Yellow,
                            "agent.react.turn" => Color::Green,
                            "debate.round" => Color::Magenta,
                            "agentshield.interception" => Color::Red,
                            "cost.usd" => Color::Blue,
                            _ => Color::White,
                        };

                        let dur_str = ev
                            .duration_ms
                            .map(|d| format!("{d}ms"))
                            .unwrap_or_else(|| "—".to_string());

                        let line = Line::from(vec![
                            Span::styled(format!("[{:03}] ", i + 1), Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                format!("{:<24} ", ev.span_name),
                                Style::default().fg(span_color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(format!("{:<8} ", dur_str), Style::default().fg(Color::Cyan)),
                        ]);
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .title(" Trace Spans (Live Hierarchy) ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(Color::Rgb(30, 45, 75))
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("▶ ");
                f.render_stateful_widget(list, body_chunks[0], &mut list_state);

                // Right: Detailed span inspector
                let selected_event = list_state.selected().and_then(|idx| events.get(idx));
                let detail_text = if let Some(ev) = selected_event {
                    let fields_pretty = serde_json::to_string_pretty(&ev.fields)
                        .unwrap_or_else(|_| ev.fields.to_string());

                    let dur_str = ev
                        .duration_ms
                        .map(|d| format!("{d} ms"))
                        .unwrap_or_else(|| "In-flight / Unfinished".to_string());

                    vec![
                        Line::from(vec![
                            Span::styled("Span Name: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            Span::styled(&ev.span_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(vec![
                            Span::styled("Span ID:   ", Style::default().fg(Color::Gray)),
                            Span::styled(&ev.id, Style::default().fg(Color::Cyan)),
                        ]),
                        Line::from(vec![
                            Span::styled("Timestamp: ", Style::default().fg(Color::Gray)),
                            Span::styled(format!("{} (unix ms)", ev.timestamp), Style::default().fg(Color::White)),
                        ]),
                        Line::from(vec![
                            Span::styled("Duration:  ", Style::default().fg(Color::Gray)),
                            Span::styled(dur_str, Style::default().fg(Color::Green)),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled("Span Attributes & Payload:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
                        Line::from(fields_pretty),
                    ]
                } else {
                    vec![Line::from(Span::styled(
                        "No span selected. Use Up/Down arrows to inspect trace events.",
                        Style::default().fg(Color::DarkGray),
                    ))]
                };

                let detail_p = Paragraph::new(detail_text)
                    .block(
                        Block::default()
                            .title(" Span Attribute Inspector (OpenTelemetry Payload) ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Green)),
                    )
                    .wrap(Wrap { trim: false });
                f.render_widget(detail_p, body_chunks[1]);

                // 3. Footer / Help
                let footer = Paragraph::new(Line::from(vec![
                    Span::styled(" [↑/k] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Previous  "),
                    Span::styled(" [↓/j] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Next  "),
                    Span::styled(" [r] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("Reload  "),
                    Span::styled(" [c] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw("Clear Journal  "),
                    Span::styled(" [q / Esc] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::raw("Quit"),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
                f.render_widget(footer, chunks[2]);
            })
            .map_err(|e| {
                crate::error::TagisanError::Execution(format!("TUI draw error: {e}"))
            })?;

        // Poll keyboard event with 250ms timeout for live updates
        if event::poll(Duration::from_millis(250)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    events.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        if !events.is_empty() {
                            list_state.select(Some(i));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i + 1 >= events.len() {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        if !events.is_empty() {
                            list_state.select(Some(i));
                        }
                    }
                    KeyCode::Char('r') => {
                        events = journal.read_all().unwrap_or_default();
                        if !events.is_empty() && list_state.selected().is_none() {
                            list_state.select(Some(events.len() - 1));
                        }
                    }
                    KeyCode::Char('c') => {
                        let _ = journal.clear();
                        events.clear();
                        list_state.select(None);
                    }
                    _ => {}
                }
            }
        } else {
            // Live background polling: re-read journal if size changed
            if let Ok(latest) = journal.read_all() {
                if latest.len() != events.len() {
                    let was_at_bottom = list_state
                        .selected()
                        .map(|s| s + 1 >= events.len())
                        .unwrap_or(true);

                    events = latest;
                    if was_at_bottom && !events.is_empty() {
                        list_state.select(Some(events.len() - 1));
                    }
                }
            }
        }
    }

    Ok(())
}
