use crate::engine::EngineContext;
use crate::error::Result;
use crate::types::{StreamChunkDelta, TokenUsage};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    Proponent,
    Adversary,
    Lakandiwa,
    Telemetry,
}

impl FocusedPane {
    pub fn next(self) -> Self {
        match self {
            FocusedPane::Proponent => FocusedPane::Adversary,
            FocusedPane::Adversary => FocusedPane::Lakandiwa,
            FocusedPane::Lakandiwa => FocusedPane::Telemetry,
            FocusedPane::Telemetry => FocusedPane::Proponent,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FocusedPane::Proponent => FocusedPane::Telemetry,
            FocusedPane::Adversary => FocusedPane::Proponent,
            FocusedPane::Lakandiwa => FocusedPane::Adversary,
            FocusedPane::Telemetry => FocusedPane::Lakandiwa,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DebateEvent {
    Round1Chunk(String),
    Round1Thinking(String),
    Round1Complete {
        provider: String,
        model: String,
        content: String,
        latency: Duration,
    },
    Round2Chunk(String),
    Round2Thinking(String),
    Round2Complete {
        provider: String,
        model: String,
        content: String,
        latency: Duration,
    },
    Round3Chunk(String),
    Round3Thinking(String),
    Round3Complete {
        provider: String,
        model: String,
        content: String,
        latency: Duration,
    },
    StatusUpdate(String),
    DebateFinished {
        total_cost: f64,
        total_tokens: u32,
        total_time: Duration,
    },
    DebateError(String),
}

/// State container for the TUI application
pub struct TuiState {
    pub topic: String,
    pub proponent_text: String,
    pub proponent_meta: String,
    pub adversary_text: String,
    pub adversary_meta: String,
    pub lakandiwa_text: String,
    pub lakandiwa_meta: String,
    pub status: String,
    pub is_running: bool,
    pub focused_pane: FocusedPane,
    pub scroll_proponent: u16,
    pub scroll_adversary: u16,
    pub scroll_lakandiwa: u16,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub total_time: Duration,
    pub start_instant: Option<Instant>,
}

impl TuiState {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            proponent_text: String::new(),
            proponent_meta: "Waiting for round 1...".to_string(),
            adversary_text: String::new(),
            adversary_meta: "Waiting for round 2...".to_string(),
            lakandiwa_text: String::new(),
            lakandiwa_meta: "Waiting for round 3...".to_string(),
            status: "Press [Enter] to start debate, [Tab] to cycle panes, [q] to exit.".to_string(),
            is_running: false,
            focused_pane: FocusedPane::Proponent,
            scroll_proponent: 0,
            scroll_adversary: 0,
            scroll_lakandiwa: 0,
            total_cost: 0.0,
            total_tokens: 0,
            total_time: Duration::ZERO,
            start_instant: None,
        }
    }

    pub fn scroll_up(&mut self) {
        match self.focused_pane {
            FocusedPane::Proponent => self.scroll_proponent = self.scroll_proponent.saturating_sub(2),
            FocusedPane::Adversary => self.scroll_adversary = self.scroll_adversary.saturating_sub(2),
            FocusedPane::Lakandiwa => self.scroll_lakandiwa = self.scroll_lakandiwa.saturating_sub(2),
            FocusedPane::Telemetry => {}
        }
    }

    pub fn scroll_down(&mut self) {
        match self.focused_pane {
            FocusedPane::Proponent => self.scroll_proponent = self.scroll_proponent.saturating_add(2),
            FocusedPane::Adversary => self.scroll_adversary = self.scroll_adversary.saturating_add(2),
            FocusedPane::Lakandiwa => self.scroll_lakandiwa = self.scroll_lakandiwa.saturating_add(2),
            FocusedPane::Telemetry => {}
        }
    }

    pub fn apply_event(&mut self, event: DebateEvent) {
        match event {
            DebateEvent::Round1Chunk(chunk) => {
                self.proponent_text.push_str(&chunk);
            }
            DebateEvent::Round1Thinking(thought) => {
                if !self.proponent_text.starts_with("--- Thinking ---\n") {
                    self.proponent_text = format!("--- Thinking ---\n{}\n--- Thesis ---\n", thought);
                } else {
                    let insert_pos = self.proponent_text.find("\n--- Thesis ---").unwrap_or(self.proponent_text.len());
                    self.proponent_text.insert_str(insert_pos, &thought);
                }
            }
            DebateEvent::Round1Complete { provider, model, content, latency } => {
                self.proponent_text = content;
                self.proponent_meta = format!("{provider} / {model} ({:.2}s)", latency.as_secs_f32());
            }
            DebateEvent::Round2Chunk(chunk) => {
                self.adversary_text.push_str(&chunk);
            }
            DebateEvent::Round2Thinking(thought) => {
                if !self.adversary_text.starts_with("--- Thinking ---\n") {
                    self.adversary_text = format!("--- Thinking ---\n{}\n--- Critique ---\n", thought);
                } else {
                    let insert_pos = self.adversary_text.find("\n--- Critique ---").unwrap_or(self.adversary_text.len());
                    self.adversary_text.insert_str(insert_pos, &thought);
                }
            }
            DebateEvent::Round2Complete { provider, model, content, latency } => {
                self.adversary_text = content;
                self.adversary_meta = format!("{provider} / {model} ({:.2}s)", latency.as_secs_f32());
            }
            DebateEvent::Round3Chunk(chunk) => {
                self.lakandiwa_text.push_str(&chunk);
            }
            DebateEvent::Round3Thinking(thought) => {
                if !self.lakandiwa_text.starts_with("--- Thinking ---\n") {
                    self.lakandiwa_text = format!("--- Thinking ---\n{}\n--- Verdict ---\n", thought);
                } else {
                    let insert_pos = self.lakandiwa_text.find("\n--- Verdict ---").unwrap_or(self.lakandiwa_text.len());
                    self.lakandiwa_text.insert_str(insert_pos, &thought);
                }
            }
            DebateEvent::Round3Complete { provider, model, content, latency } => {
                self.lakandiwa_text = content;
                self.lakandiwa_meta = format!("{provider} / {model} ({:.2}s)", latency.as_secs_f32());
            }
            DebateEvent::StatusUpdate(status) => {
                self.status = status;
            }
            DebateEvent::DebateFinished { total_cost, total_tokens, total_time } => {
                self.is_running = false;
                self.total_cost = total_cost;
                self.total_tokens = total_tokens;
                self.total_time = total_time;
                self.status = format!(
                    "✔ Debate Completed in {:.2}s! Total Spent: ${:.4} USD ({} tokens). Press [Tab] to inspect panes or [q] to exit.",
                    total_time.as_secs_f32(), total_cost, total_tokens
                );
            }
            DebateEvent::DebateError(err) => {
                self.is_running = false;
                self.status = format!("❌ Error: {err}");
            }
        }
    }
}

/// Launch the interactive multi-pane TUI for Dialectical Debate
pub async fn run_debate_tui(topic: String, ctx: &EngineContext) -> Result<()> {
    enable_raw_mode().map_err(|e| crate::error::TagisanError::Execution(e.to_string()))?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen)
        .map_err(|e| crate::error::TagisanError::Execution(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::TagisanError::Execution(e.to_string()))?;

    let mut state = TuiState::new(topic.clone());
    let (tx, mut rx) = mpsc::unbounded_channel::<DebateEvent>();

    // Start debate automatically on launch
    start_debate_task(topic.clone(), ctx, tx.clone());
    state.is_running = true;
    state.start_instant = Some(Instant::now());
    state.status = "Debate in progress: Streaming Round 1 (Thesis)...".to_string();

    let res = run_app_loop(&mut terminal, &mut state, &mut rx, tx, ctx).await;

    // Cleanup terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    res
}

fn start_debate_task(
    topic: String,
    ctx: &EngineContext,
    tx: mpsc::UnboundedSender<DebateEvent>,
) {
    let ctx_budget = ctx.budget_tracker.clone();
    let ctx_cancel = ctx.cancellation_token.clone();

    // Determine configured providers for 3 roles
    let proponent = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
    } else if std::env::var("OPENAI_API_KEY").is_ok() {
        ("openai".to_string(), "gpt-4o".to_string())
    } else {
        ("ollama".to_string(), "llama3.2".to_string())
    };

    let adversary = if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        ("deepseek".to_string(), "deepseek-reasoner".to_string())
    } else if std::env::var("XAI_API_KEY").is_ok() {
        ("xai".to_string(), "grok-2-latest".to_string())
    } else if std::env::var("GEMINI_API_KEY").is_ok() {
        ("gemini".to_string(), "gemini-2.0-flash".to_string())
    } else {
        ("ollama".to_string(), "llama3.2".to_string())
    };

    let adjudicator = if std::env::var("GEMINI_API_KEY").is_ok() {
        ("gemini".to_string(), "gemini-1.5-pro".to_string())
    } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
    } else {
        ("openai".to_string(), "gpt-4o".to_string())
    };

    let p_prov_id = proponent.0.clone();
    let p_model = proponent.1.clone();
    let a_prov_id = adversary.0.clone();
    let a_model = adversary.1.clone();
    let adj_prov_id = adjudicator.0.clone();
    let adj_model = adjudicator.1.clone();

    let p_prov = ctx.get_provider(&p_prov_id);
    let a_prov = ctx.get_provider(&a_prov_id);
    let adj_prov = ctx.get_provider(&adj_prov_id);

    tokio::spawn(async move {
        let (p_prov, a_prov, adj_prov) = match (p_prov, a_prov, adj_prov) {
            (Ok(p), Ok(a), Ok(adj)) => (p, a, adj),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                tx.send(DebateEvent::DebateError(format!("Provider init error: {e}"))).ok();
                return;
            }
        };

        let start_time = Instant::now();

        // ----------------------------------------------------
        // Round 1: Thesis (Real-Time Live SSE Stream)
        // ----------------------------------------------------
        tx.send(DebateEvent::StatusUpdate("⚔️ Round 1: Proponent streaming thesis in real-time...".to_string())).ok();
        let thesis_prompt = format!(
            "You are the Proponent in a high-rigor peer debate.\nUser Prompt:\n\"{}\"\n\nTASK: Provide a comprehensive, thoroughly reasoned initial solution.",
            topic
        );
        let req1 = crate::types::CompletionRequest::new(p_model.clone(), thesis_prompt)
            .with_temperature(0.7)
            .with_stream(true)
            .with_cancellation(ctx_cancel.clone());

        let round1_start = Instant::now();
        let mut stream1 = match p_prov.stream(req1).await {
            Ok(s) => s,
            Err(e) => {
                tx.send(DebateEvent::DebateError(format!("Thesis stream initialization failed: {e}"))).ok();
                return;
            }
        };

        let mut thesis_text = String::new();
        let mut usage1 = TokenUsage::default();

        while let Some(chunk_res) = stream1.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if let Some(u) = chunk.usage {
                        usage1 = u;
                    }
                    match chunk.delta {
                        StreamChunkDelta::Text(t) => {
                            thesis_text.push_str(&t);
                            tx.send(DebateEvent::Round1Chunk(t)).ok();
                        }
                        StreamChunkDelta::Thinking(th) => {
                            tx.send(DebateEvent::Round1Thinking(th)).ok();
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tx.send(DebateEvent::DebateError(format!("Thesis stream error: {e}"))).ok();
                    return;
                }
            }
        }

        ctx_budget.record(&p_model, usage1.prompt_tokens, usage1.completion_tokens).ok();
        tx.send(DebateEvent::Round1Complete {
            provider: p_prov_id.clone(),
            model: p_model.clone(),
            content: thesis_text.clone(),
            latency: round1_start.elapsed(),
        }).ok();

        // ----------------------------------------------------
        // Round 2: Antithesis (Real-Time Live SSE Stream)
        // ----------------------------------------------------
        tx.send(DebateEvent::StatusUpdate("🛡️ Round 2: Adversary streaming critique in real-time...".to_string())).ok();
        let antithesis_prompt = format!(
            "You are the Adversarial Critic in a high-rigor peer debate.\nOriginal Topic:\n\"{}\"\n\nProponent Solution:\n{}\n\nTASK: Ruthlessly scrutinize the solution for bugs, flaws, and edge cases.",
            topic, thesis_text
        );
        let req2 = crate::types::CompletionRequest::new(a_model.clone(), antithesis_prompt)
            .with_temperature(0.4)
            .with_stream(true)
            .with_cancellation(ctx_cancel.clone());

        let round2_start = Instant::now();
        let mut stream2 = match a_prov.stream(req2).await {
            Ok(s) => s,
            Err(e) => {
                tx.send(DebateEvent::DebateError(format!("Antithesis stream initialization failed: {e}"))).ok();
                return;
            }
        };

        let mut antithesis_text = String::new();
        let mut usage2 = TokenUsage::default();

        while let Some(chunk_res) = stream2.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if let Some(u) = chunk.usage {
                        usage2 = u;
                    }
                    match chunk.delta {
                        StreamChunkDelta::Text(t) => {
                            antithesis_text.push_str(&t);
                            tx.send(DebateEvent::Round2Chunk(t)).ok();
                        }
                        StreamChunkDelta::Thinking(th) => {
                            tx.send(DebateEvent::Round2Thinking(th)).ok();
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tx.send(DebateEvent::DebateError(format!("Antithesis stream error: {e}"))).ok();
                    return;
                }
            }
        }

        ctx_budget.record(&a_model, usage2.prompt_tokens, usage2.completion_tokens).ok();
        tx.send(DebateEvent::Round2Complete {
            provider: a_prov_id.clone(),
            model: a_model.clone(),
            content: antithesis_text.clone(),
            latency: round2_start.elapsed(),
        }).ok();

        // ----------------------------------------------------
        // Round 3: Synthesis (Real-Time Live SSE Stream)
        // ----------------------------------------------------
        tx.send(DebateEvent::StatusUpdate("⚖️ Round 3: Lakandiwa streaming final verdict in real-time...".to_string())).ok();
        let synthesis_prompt = format!(
            "You are the Lakandiwa in this Tagisan debate.\nTopic:\n\"{}\"\n\n--- Thesis ---\n{}\n\n--- Antithesis ---\n{}\n\nTASK: Evaluate arguments and produce the definitive verified synthesis.",
            topic, thesis_text, antithesis_text
        );
        let req3 = crate::types::CompletionRequest::new(adj_model.clone(), synthesis_prompt)
            .with_temperature(0.2)
            .with_stream(true)
            .with_cancellation(ctx_cancel.clone());

        let round3_start = Instant::now();
        let mut stream3 = match adj_prov.stream(req3).await {
            Ok(s) => s,
            Err(e) => {
                tx.send(DebateEvent::DebateError(format!("Synthesis stream initialization failed: {e}"))).ok();
                return;
            }
        };

        let mut synthesis_text = String::new();
        let mut usage3 = TokenUsage::default();

        while let Some(chunk_res) = stream3.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if let Some(u) = chunk.usage {
                        usage3 = u;
                    }
                    match chunk.delta {
                        StreamChunkDelta::Text(t) => {
                            synthesis_text.push_str(&t);
                            tx.send(DebateEvent::Round3Chunk(t)).ok();
                        }
                        StreamChunkDelta::Thinking(th) => {
                            tx.send(DebateEvent::Round3Thinking(th)).ok();
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tx.send(DebateEvent::DebateError(format!("Synthesis stream error: {e}"))).ok();
                    return;
                }
            }
        }

        ctx_budget.record(&adj_model, usage3.prompt_tokens, usage3.completion_tokens).ok();
        tx.send(DebateEvent::Round3Complete {
            provider: adj_prov_id.clone(),
            model: adj_model.clone(),
            content: synthesis_text,
            latency: round3_start.elapsed(),
        }).ok();

        let total_tokens = usage1.prompt_tokens + usage1.completion_tokens
            + usage2.prompt_tokens + usage2.completion_tokens
            + usage3.prompt_tokens + usage3.completion_tokens;

        tx.send(DebateEvent::DebateFinished {
            total_cost: ctx_budget.current_spent_usd(),
            total_tokens,
            total_time: start_time.elapsed(),
        }).ok();
    });
}

async fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TuiState,
    rx: &mut mpsc::UnboundedReceiver<DebateEvent>,
    tx: mpsc::UnboundedSender<DebateEvent>,
    ctx: &EngineContext,
) -> Result<()> {
    loop {
        // Drain all pending debate stream events
        while let Ok(event) = rx.try_recv() {
            state.apply_event(event);
        }

        // Draw UI
        terminal
            .draw(|f| draw_ui(f, state))
            .map_err(|e| crate::error::TagisanError::Execution(e.to_string()))?;

        // Poll keyboard input with short timeout for smooth 60fps rendering
        if event::poll(Duration::from_millis(16))
            .map_err(|e| crate::error::TagisanError::Execution(e.to_string()))?
        {
            if let Event::Key(key) = event::read().map_err(|e| crate::error::TagisanError::Execution(e.to_string()))? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        ctx.cancellation_token.cancel();
                        break;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        ctx.cancellation_token.cancel();
                        break;
                    }
                    KeyCode::Tab => {
                        state.focused_pane = state.focused_pane.next();
                    }
                    KeyCode::BackTab => {
                        state.focused_pane = state.focused_pane.prev();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.scroll_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.scroll_down();
                    }
                    KeyCode::Enter | KeyCode::Char('r') => {
                        if !state.is_running {
                            state.is_running = true;
                            state.proponent_text.clear();
                            state.adversary_text.clear();
                            state.lakandiwa_text.clear();
                            state.scroll_proponent = 0;
                            state.scroll_adversary = 0;
                            state.scroll_lakandiwa = 0;
                            start_debate_task(state.topic.clone(), ctx, tx.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw_ui(f: &mut ratatui::Frame, state: &TuiState) {
    let size = f.area();

    // Main vertical layout: Header, Content (Split Panes), Telemetry Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // 3 Main Panes
            Constraint::Length(6), // Status & Telemetry
        ])
        .split(size);

    // Header Block
    let header_text = vec![
        Line::from(vec![
            Span::styled("🇵🇭 TAGISAN NG TALINO ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("— Real-Time Multi-LLM Live Streaming Debate", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Topic: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("\"{}\"", state.topic), Style::default().fg(Color::LightCyan).add_modifier(Modifier::ITALIC)),
        ]),
    ];
    let header_widget = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(header_widget, chunks[0]);

    // Content Panes: Split into 3 columns (Proponent, Adversary, Lakandiwa)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    // 1. Proponent (Thesis) Pane
    let prop_border_style = if state.focused_pane == FocusedPane::Proponent {
        Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let prop_title = format!(" [1] Proponent (Thesis) — {} ", state.proponent_meta);
    let prop_widget = Paragraph::new(state.proponent_text.as_str())
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_proponent, 0))
        .block(
            Block::default()
                .title(prop_title)
                .borders(Borders::ALL)
                .border_type(if state.focused_pane == FocusedPane::Proponent { BorderType::Thick } else { BorderType::Rounded })
                .border_style(prop_border_style),
        );
    f.render_widget(prop_widget, content_chunks[0]);

    // 2. Adversary (Antithesis) Pane
    let adv_border_style = if state.focused_pane == FocusedPane::Adversary {
        Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let adv_title = format!(" [2] Adversary (Antithesis) — {} ", state.adversary_meta);
    let adv_widget = Paragraph::new(state.adversary_text.as_str())
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_adversary, 0))
        .block(
            Block::default()
                .title(adv_title)
                .borders(Borders::ALL)
                .border_type(if state.focused_pane == FocusedPane::Adversary { BorderType::Thick } else { BorderType::Rounded })
                .border_style(adv_border_style),
        );
    f.render_widget(adv_widget, content_chunks[1]);

    // 3. Lakandiwa (Master Synthesis) Pane
    let lk_border_style = if state.focused_pane == FocusedPane::Lakandiwa {
        Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let lk_title = format!(" [3] Lakandiwa (Synthesis) — {} ", state.lakandiwa_meta);
    let lk_widget = Paragraph::new(state.lakandiwa_text.as_str())
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_lakandiwa, 0))
        .block(
            Block::default()
                .title(lk_title)
                .borders(Borders::ALL)
                .border_type(if state.focused_pane == FocusedPane::Lakandiwa { BorderType::Thick } else { BorderType::Rounded })
                .border_style(lk_border_style),
        );
    f.render_widget(lk_widget, content_chunks[2]);

    // Footer Block (Status & Telemetry)
    let telem_border = if state.focused_pane == FocusedPane::Telemetry {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let footer_lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(&state.status, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Telemetry: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Spent: ${:.4} USD", state.total_cost), Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled(format!("Tokens: {}", state.total_tokens), Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::styled(format!("Time: {:.2}s", state.total_time.as_secs_f32()), Style::default().fg(Color::LightMagenta)),
        ]),
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Tab] Cycle Pane  [↑/↓] Scroll Pane  [Enter/r] Restart Debate  [q/Esc] Exit", Style::default().fg(Color::Gray)),
        ]),
    ];

    let footer_widget = Paragraph::new(footer_lines).block(
        Block::default()
            .title(" Telemetry & Controls ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(telem_border),
    );
    f.render_widget(footer_widget, chunks[2]);
}
