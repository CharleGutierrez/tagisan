//! Unified User Notification & System Abnormality Alerting Subsystem
//!
//! Provides first-class alerting, terminal ANSI banner rendering, cross-platform
//! desktop notifications (Windows Toast, Unix notify-send), and thread-safe history/pub-sub
//! across all Local <-> Non-Local transitions and system abnormalities:
//! - Provider failovers / cascades (CascadeProvider & Swarm pipeline: RateLimited 429, BadResponse 502,
//!   AuthError 401, Network timeout, BudgetExceeded)
//! - Skills subsystem transitions (SkillDispatcher & TokenBudget: CheatSheet <-> Comprehensive,
//!   domain diversity anti-monopoly quotas, context downscaling)
//! - Semantic Guard abnormalities (SemanticInvariantGuard: tool schema corruption, dropped required
//!   parameters, diagnostic degradation)
//! - Model auto-healing / Local-only locks (missing local models auto-healed, offline mode overrides)

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::ecc::SemanticViolation;

/// Severity classification of a notification or abnormality alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Critical,
    SecurityAlert,
    Emergency,
}

impl std::fmt::Display for NotificationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
            Self::SecurityAlert => write!(f, "SECURITY_ALERT"),
            Self::Emergency => write!(f, "EMERGENCY"),
        }
    }
}

/// Category classification of the system abnormality or event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationCategory {
    ProviderFailover,
    SkillTransition,
    DomainDiversityQuota,
    ContextDownscaling,
    SemanticGuard,
    ModelAutoHeal,
    OfflineLockOverride,
    SystemAlert,
    SecurityAlert,
    CyberDefenseAlert,
}

impl std::fmt::Display for NotificationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderFailover => write!(f, "PROVIDER_FAILOVER"),
            Self::SkillTransition => write!(f, "SKILL_TRANSITION"),
            Self::DomainDiversityQuota => write!(f, "DOMAIN_DIVERSITY_QUOTA"),
            Self::ContextDownscaling => write!(f, "CONTEXT_DOWNSCALING"),
            Self::SemanticGuard => write!(f, "SEMANTIC_GUARD"),
            Self::ModelAutoHeal => write!(f, "MODEL_AUTO_HEAL"),
            Self::OfflineLockOverride => write!(f, "OFFLINE_LOCK_OVERRIDE"),
            Self::SystemAlert => write!(f, "SYSTEM_ALERT"),
            Self::SecurityAlert => write!(f, "SECURITY_ALERT"),
            Self::CyberDefenseAlert => write!(f, "CYBER_DEFENSE_ALERT"),
        }
    }
}

/// Structured payload details for Provider Failover alerts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailoverNotificationDetails {
    pub original_provider: String,
    pub original_model: String,
    pub target_provider: String,
    pub target_model: String,
    pub trigger_reason: String,
    pub cost_delta: String,
    pub action_taken: String,
    pub stage_info: Option<String>,
}

/// Structured payload details for Skill Subsystem Transitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillTransitionDetails {
    pub from_provider: String,
    pub to_provider: String,
    pub from_mode: String,
    pub to_mode: String,
    pub token_budget: usize,
    pub context_window: usize,
    pub reason: String,
    pub action_taken: String,
}

/// Structured payload details for Domain Diversity Anti-Monopoly Quotas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainQuotaDetails {
    pub domain: String,
    pub max_allowed: usize,
    pub attempted_skill: String,
    pub suppressed_skills: Vec<String>,
    pub provider: String,
    pub action_taken: String,
}

/// Structured payload details for Context Downscaling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextDownscaleDetails {
    pub original_context: usize,
    pub downscaled_context: usize,
    pub model: String,
    pub reason: String,
    pub action_taken: String,
}

/// Structured payload details for Semantic Invariant Guard violations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticGuardDetails {
    pub violation_type: String,
    pub target_item: String,
    pub details: String,
    pub action_taken: String,
}

/// Structured payload details for Model Auto-Healing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoHealDetails {
    pub requested_model: String,
    pub healed_model: String,
    pub provider: String,
    pub reason: String,
    pub action_taken: String,
}

/// Structured payload details for Offline / Local-Only lock overrides
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineLockDetails {
    pub requested_provider: String,
    pub enforced_provider: String,
    pub lock_reason: String,
    pub action_taken: String,
}

/// Structured payload details for Cyber Defense / APT Interception alerts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CyberDefenseAlertDetails {
    pub threat_actor: String,
    pub attack_vector: String,
    pub target: String,
    pub indicator: String,
    pub threat_level: String,
    pub action_taken: String,
    pub mitigation_remediation: String,
}

/// Type-safe polymorphic payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotificationPayload {
    Failover(FailoverNotificationDetails),
    SkillTransition(SkillTransitionDetails),
    DomainQuota(DomainQuotaDetails),
    ContextDownscale(ContextDownscaleDetails),
    SemanticGuard(SemanticGuardDetails),
    AutoHeal(AutoHealDetails),
    OfflineLock(OfflineLockDetails),
    CyberDefenseAlert(CyberDefenseAlertDetails),
    Generic(HashMap<String, String>),
}

/// Unified, timestamped notification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub id: u64,
    pub timestamp_epoch_ms: u64,
    pub category: NotificationCategory,
    pub severity: NotificationSeverity,
    pub title: String,
    pub message: String,
    pub action_taken: String,
    pub payload: NotificationPayload,
}

impl NotificationEvent {
    pub fn new(
        category: NotificationCategory,
        severity: NotificationSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
        action_taken: impl Into<String>,
        payload: NotificationPayload,
    ) -> Self {
        let timestamp_epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: 0, // Assigned by hub
            timestamp_epoch_ms,
            category,
            severity,
            title: title.into(),
            message: message.into(),
            action_taken: action_taken.into(),
            payload,
        }
    }
}

/// Thread-safe central notification hub and subscriber bus
pub struct NotificationHub {
    history: RwLock<Vec<NotificationEvent>>,
    sender: broadcast::Sender<NotificationEvent>,
    desktop_enabled: AtomicBool,
    banner_enabled: AtomicBool,
    desktop_delivery_count: AtomicUsize,
    terminal_banner_count: AtomicUsize,
    event_counter: AtomicU64,
}

static GLOBAL_HUB: OnceLock<NotificationHub> = OnceLock::new();

impl NotificationHub {
    /// Detect whether desktop notifications should default to enabled or suppressed
    pub fn detect_desktop_enabled_default() -> bool {
        if std::env::var("TAGISAN_TEST_MODE").is_ok()
            || std::env::var("TAGISAN_NO_DESKTOP_NOTIFY").is_ok()
            || std::env::var("TGS_NO_DESKTOP_NOTIFY").is_ok()
            || std::env::var("TAGISAN_NO_NOTIFY").is_ok()
            || std::env::var("TGS_NO_NOTIFY").is_ok()
            || std::env::var("TAGISAN_MUTE").is_ok()
            || std::env::var("TGS_MUTE").is_ok()
        {
            return false;
        }

        if let Ok(val) = std::env::var("TAGISAN_NOTIFY") {
            if val == "0" || val.eq_ignore_ascii_case("false") || val.eq_ignore_ascii_case("off") {
                return false;
            }
        }
        if let Ok(val) = std::env::var("TGS_NOTIFY") {
            if val == "0" || val.eq_ignore_ascii_case("false") || val.eq_ignore_ascii_case("off") {
                return false;
            }
        }

        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let config_path = std::path::Path::new(&home).join(".config").join("tagisan").join("config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(enabled) = val.get("desktop_notifications").and_then(|v| v.as_bool()) {
                            return enabled;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(256);
        let desktop_enabled = Self::detect_desktop_enabled_default();
        Self {
            history: RwLock::new(Vec::new()),
            sender,
            desktop_enabled: AtomicBool::new(desktop_enabled),
            banner_enabled: AtomicBool::new(true),
            desktop_delivery_count: AtomicUsize::new(0),
            terminal_banner_count: AtomicUsize::new(0),
            event_counter: AtomicU64::new(1),
        }
    }

    /// Access the global singleton hub
    pub fn global() -> &'static NotificationHub {
        GLOBAL_HUB.get_or_init(NotificationHub::new)
    }

    /// Emit an event through the hub: stores in history, notifies subscribers,
    /// renders ANSI banner, and emits desktop notification.
    pub fn emit(&self, mut event: NotificationEvent) {
        event.id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        if event.timestamp_epoch_ms == 0 {
            event.timestamp_epoch_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }

        // 1. Store in thread-safe history
        {
            let mut hist = self.history.write().unwrap();
            hist.push(event.clone());
        }

        // 2. Publish to real-time subscribers
        let _ = self.sender.send(event.clone());

        // 3. Render ANSI terminal banner if enabled
        if self.banner_enabled.load(Ordering::Relaxed) {
            self.terminal_banner_count.fetch_add(1, Ordering::Relaxed);
            Self::render_terminal_banner(&event);
        }

        // 4. Emit desktop notification if enabled
        if self.desktop_enabled.load(Ordering::Relaxed) {
            self.desktop_delivery_count.fetch_add(1, Ordering::Relaxed);
            Self::send_desktop_notification(&event);
        }
    }

    /// Retrieve a snapshot copy of the full notification history
    pub fn history(&self) -> Vec<NotificationEvent> {
        self.history.read().unwrap().clone()
    }

    /// Number of events in history
    pub fn history_len(&self) -> usize {
        self.history.read().unwrap().len()
    }

    /// Clear all history records (useful for test resets)
    pub fn clear_history(&self) {
        let mut hist = self.history.write().unwrap();
        hist.clear();
    }

    /// Filter history by category
    pub fn filter_by_category(&self, cat: NotificationCategory) -> Vec<NotificationEvent> {
        let hist = self.history.read().unwrap();
        hist.iter().filter(|e| e.category == cat).cloned().collect()
    }

    /// Filter history by severity
    pub fn filter_by_severity(&self, sev: NotificationSeverity) -> Vec<NotificationEvent> {
        let hist = self.history.read().unwrap();
        hist.iter().filter(|e| e.severity == sev).cloned().collect()
    }

    /// Subscribe to real-time notification stream
    pub fn subscribe(&self) -> broadcast::Receiver<NotificationEvent> {
        self.sender.subscribe()
    }

    /// Configure whether desktop notifications are dispatched
    pub fn set_desktop_enabled(&self, enabled: bool) {
        self.desktop_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Configure whether ANSI banners are printed to stderr
    pub fn set_banner_enabled(&self, enabled: bool) {
        self.banner_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Return count of desktop notifications processed/attempted
    pub fn desktop_delivery_count(&self) -> usize {
        self.desktop_delivery_count.load(Ordering::Relaxed)
    }

    /// Return count of terminal banners rendered
    pub fn terminal_banner_count(&self) -> usize {
        self.terminal_banner_count.load(Ordering::Relaxed)
    }

    /// Helper to render consistent, clean ANSI boxed banners
    fn render_terminal_banner(event: &NotificationEvent) {
        let width = 77;
        let print_row = |content: &str| {
            let char_count = content.chars().count();
            let pad = if char_count < width { width - char_count } else { 0 };
            eprintln!("│ {}{} │", content, " ".repeat(pad));
        };

        match &event.payload {
            NotificationPayload::Failover(f) => {
                eprintln!("\n{}", "┌───────────────────────────── ⚠️  FAILOVER NOTICE ─────────────────────────────┐".yellow().bold());
                print_row(&format!("Cloud Provider : {} ({})", f.original_provider, f.original_model));
                print_row(&format!("Trigger Reason : {}", f.trigger_reason));
                print_row(&format!("Action Taken   : {}", f.action_taken));
                print_row(&format!("Cost Delta     : {}", f.cost_delta));
                if let Some(ref stage) = f.stage_info {
                    print_row(&format!("Current Stage  : {}", stage));
                }
                print_row("Context Retained: 100% (Architecture & Types preserved on Blackboard)");
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".yellow().bold());
            }
            NotificationPayload::SkillTransition(s) => {
                eprintln!("\n{}", "┌─────────────────────── 🧠 SKILL SUBSYSTEM TRANSITION ────────────────────────┐".cyan().bold());
                print_row(&format!("Transition     : {} ➔ {}", s.from_provider, s.to_provider));
                print_row(&format!("Injection Mode : {} ➔ {}", s.from_mode, s.to_mode));
                print_row(&format!("Token Budget   : {} tokens (Context: {} tokens)", s.token_budget, s.context_window));
                print_row(&format!("Trigger Reason : {}", s.reason));
                print_row(&format!("Action Taken   : {}", s.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".cyan().bold());
            }
            NotificationPayload::DomainQuota(q) => {
                eprintln!("\n{}", "┌────────────────────── ⚖️  DOMAIN DIVERSITY QUOTA ALERT ──────────────────────┐".magenta().bold());
                print_row(&format!("Domain Enforced: {} (Max Allowed: {} on {})", q.domain, q.max_allowed, q.provider));
                print_row(&format!("Candidate Skill: {}", q.attempted_skill));
                if !q.suppressed_skills.is_empty() {
                    print_row(&format!("Suppressed     : {}", q.suppressed_skills.join(", ")));
                }
                print_row(&format!("Action Taken   : {}", q.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".magenta().bold());
            }
            NotificationPayload::ContextDownscale(c) => {
                eprintln!("\n{}", "┌──────────────────────── 📉 CONTEXT DOWNSCALE ALERT ──────────────────────────┐".blue().bold());
                print_row(&format!("Target Model   : {}", c.model));
                print_row(&format!("Context Scale  : {} tokens ➔ {} tokens", c.original_context, c.downscaled_context));
                print_row(&format!("Trigger Reason : {}", c.reason));
                print_row(&format!("Action Taken   : {}", c.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".blue().bold());
            }
            NotificationPayload::SemanticGuard(g) => {
                eprintln!("\n{}", "┌───────────────────── 🛡️  SEMANTIC INVARIANT GUARD ALERT ──────────────────────┐".red().bold());
                print_row(&format!("Violation Type : {}", g.violation_type));
                print_row(&format!("Target Item    : {}", g.target_item));
                print_row(&format!("Details        : {}", g.details));
                print_row(&format!("Action Taken   : {}", g.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".red().bold());
            }
            NotificationPayload::AutoHeal(h) => {
                eprintln!("\n{}", "┌──────────────────────── 🩹 MODEL AUTO-HEAL RECOVERY ─────────────────────────┐".yellow().bold());
                print_row(&format!("Provider       : {}", h.provider));
                print_row(&format!("Requested Model: {}", h.requested_model));
                print_row(&format!("Fallback Model : {}", h.healed_model));
                print_row(&format!("Reason         : {}", h.reason));
                print_row(&format!("Action Taken   : {}", h.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".yellow().bold());
            }
            NotificationPayload::OfflineLock(l) => {
                eprintln!("\n{}", "┌──────────────────────── 🔒 OFFLINE / LOCAL-ONLY LOCK ────────────────────────┐".bright_red().bold());
                print_row(&format!("Requested Target: {}", l.requested_provider));
                print_row(&format!("Enforced Target : {}", l.enforced_provider));
                print_row(&format!("Reason          : {}", l.lock_reason));
                print_row(&format!("Action Taken    : {}", l.action_taken));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".bright_red().bold());
            }
            NotificationPayload::CyberDefenseAlert(c) => {
                eprintln!("\n{}", "┌───────────────────── 🚨 CRITICAL CYBER DEFENSE ALERT ─────────────────────┐".bright_red().bold());
                print_row(&format!("Threat Actor   : {}", c.threat_actor));
                print_row(&format!("Attack Vector  : {}", c.attack_vector));
                print_row(&format!("Target / Asset : {}", c.target));
                print_row(&format!("Threat Level   : {}", c.threat_level));
                print_row(&format!("Indicator      : {}", c.indicator));
                print_row(&format!("Action Taken   : {}", c.action_taken));
                print_row(&format!("Remediation    : {}", c.mitigation_remediation));
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────┘".bright_red().bold());
            }
            NotificationPayload::Generic(map) => {
                eprintln!("\n{}", "┌────────────────────────────── 📢 SYSTEM ALERT ──────────────────────────────┐".green().bold());
                print_row(&format!("Title  : {}", event.title));
                print_row(&format!("Message: {}", event.message));
                print_row(&format!("Action : {}", event.action_taken));
                for (k, v) in map {
                    print_row(&format!("{}: {}", k, v));
                }
                eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".green().bold());
            }
        }
    }

    /// Dispatches a platform-native desktop notification (Windows Toast or Unix notify-send)
    fn send_desktop_notification(event: &NotificationEvent) {
        // Respect test / headless / user suppression settings
        if !Self::detect_desktop_enabled_default() {
            return;
        }

        let summary = format!("Tagisan: {}", event.title);
        let body = format!("{}\nAction: {}", event.message, event.action_taken);

        #[cfg(target_family = "unix")]
        {
            let urgency = match event.severity {
                NotificationSeverity::Critical | NotificationSeverity::Emergency => "normal", // Cap at normal to prevent sticky screen popups
                NotificationSeverity::Warning | NotificationSeverity::SecurityAlert => "normal",
                NotificationSeverity::Info => "low",
            };
            let _ = std::process::Command::new("notify-send")
                .arg("-u")
                .arg(urgency)
                .arg("-t")
                .arg("2500") // Auto-dismiss after 2.5 seconds so it never occupies the screen
                .arg("-h")
                .arg("int:transient:1") // Transient hint for KDE Plasma & GNOME
                .arg("-a")
                .arg("Tagisan")
                .arg(&summary)
                .arg(&body)
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            // Escape single quotes for PowerShell literal string
            let safe_summary = summary.replace('\'', "''");
            let safe_body = body.replace('\'', "''");
            let script = format!(
                "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
                 $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
                 $textNodes = $template.GetElementsByTagName('text'); \
                 $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) > $null; \
                 $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) > $null; \
                 $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Tagisan'); \
                 $notification = [Windows.UI.Notifications.ToastNotification]::new($template); \
                 $notifier.Show($notification);",
                safe_summary, safe_body
            );
            let _ = std::process::Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&script)
                .spawn();
        }
    }
}

impl Default for NotificationHub {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// CONVENIENCE GLOBAL API FUNCTIONS
// =========================================================================

/// Access the global NotificationHub
pub fn hub() -> &'static NotificationHub {
    NotificationHub::global()
}

/// Emit any custom NotificationEvent
pub fn notify(event: NotificationEvent) {
    hub().emit(event);
}

/// Emit a Provider Failover notification
pub fn notify_failover(
    original_provider: &str,
    original_model: &str,
    target_provider: &str,
    target_model: &str,
    trigger_reason: &str,
    cost_delta: &str,
    action_taken: &str,
    stage_info: Option<String>,
) {
    let details = FailoverNotificationDetails {
        original_provider: original_provider.to_string(),
        original_model: original_model.to_string(),
        target_provider: target_provider.to_string(),
        target_model: target_model.to_string(),
        trigger_reason: trigger_reason.to_string(),
        cost_delta: cost_delta.to_string(),
        action_taken: action_taken.to_string(),
        stage_info,
    };

    let title = format!("Provider Failover: {} ➔ {}", original_provider, target_provider);
    let message = format!(
        "Cloud model '{}:{}' failed ({}) -> Evacuating to '{}:{}'",
        original_provider, original_model, trigger_reason, target_provider, target_model
    );

    let event = NotificationEvent::new(
        NotificationCategory::ProviderFailover,
        NotificationSeverity::Critical,
        title,
        message,
        action_taken,
        NotificationPayload::Failover(details),
    );

    hub().emit(event);
}

/// Emit a Skill Subsystem Transition notification
pub fn notify_skill_transition(
    from_provider: &str,
    to_provider: &str,
    from_mode: &str,
    to_mode: &str,
    token_budget: usize,
    context_window: usize,
    reason: &str,
    action_taken: &str,
) {
    let details = SkillTransitionDetails {
        from_provider: from_provider.to_string(),
        to_provider: to_provider.to_string(),
        from_mode: from_mode.to_string(),
        to_mode: to_mode.to_string(),
        token_budget,
        context_window,
        reason: reason.to_string(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Skill Subsystem Transition: {} ➔ {}", from_mode, to_mode);
    let message = format!(
        "Skill prompt injection mode mutated from {} to {} (transition '{}' ➔ '{}', Budget: {} tokens)",
        from_mode, to_mode, from_provider, to_provider, token_budget
    );

    let event = NotificationEvent::new(
        NotificationCategory::SkillTransition,
        NotificationSeverity::Info,
        title,
        message,
        action_taken,
        NotificationPayload::SkillTransition(details),
    );

    hub().emit(event);
}

/// Emit a Domain Diversity Anti-Monopoly Quota alert
pub fn notify_domain_quota(
    domain: &str,
    max_allowed: usize,
    attempted_skill: &str,
    suppressed_skills: Vec<String>,
    provider: &str,
    action_taken: &str,
) {
    let details = DomainQuotaDetails {
        domain: domain.to_string(),
        max_allowed,
        attempted_skill: attempted_skill.to_string(),
        suppressed_skills,
        provider: provider.to_string(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Domain Diversity Quota Enforced: '{}'", domain);
    let message = format!(
        "Anti-monopoly quota reached (max {} for domain '{}' on {}). Candidate '{}' suppressed.",
        max_allowed, domain, provider, attempted_skill
    );

    let event = NotificationEvent::new(
        NotificationCategory::DomainDiversityQuota,
        NotificationSeverity::Warning,
        title,
        message,
        action_taken,
        NotificationPayload::DomainQuota(details),
    );

    hub().emit(event);
}

/// Emit a Context Downscaling notification
pub fn notify_context_downscaling(
    original_context: usize,
    downscaled_context: usize,
    model: &str,
    reason: &str,
    action_taken: &str,
) {
    let details = ContextDownscaleDetails {
        original_context,
        downscaled_context,
        model: model.to_string(),
        reason: reason.to_string(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Context Window Downscaled: {} ➔ {}", original_context, downscaled_context);
    let message = format!(
        "Context window scaled down from {} to {} tokens for model '{}': {}",
        original_context, downscaled_context, model, reason
    );

    let event = NotificationEvent::new(
        NotificationCategory::ContextDownscaling,
        NotificationSeverity::Warning,
        title,
        message,
        action_taken,
        NotificationPayload::ContextDownscale(details),
    );

    hub().emit(event);
}

/// Emit a Semantic Invariant Guard violation alert
pub fn notify_semantic_guard(violation: &SemanticViolation, action_taken: &str) {
    let (violation_type, target_item, details) = match violation {
        SemanticViolation::TruncatedToolSchema { tool_name, details } => (
            "TruncatedToolSchema".to_string(),
            tool_name.clone(),
            details.clone(),
        ),
        SemanticViolation::DroppedRequiredParameter { tool_name, param_name } => (
            "DroppedRequiredParameter".to_string(),
            format!("{}.{}", tool_name, param_name),
            format!("Required parameter '{}' omitted from tool '{}'", param_name, tool_name),
        ),
        SemanticViolation::DiagnosticDegradation { error_code, details, min_tokens_needed } => (
            "DiagnosticDegradation".to_string(),
            error_code.clone(),
            format!("{} (minimum {} tokens needed)", details, min_tokens_needed),
        ),
        SemanticViolation::SafetyContractOmitted { missing_rule } => (
            "SafetyContractOmitted".to_string(),
            "safety_contract".to_string(),
            missing_rule.clone(),
        ),
        SemanticViolation::ContextExhaustionWithInvariantRisk { required_tokens, available_tokens } => (
            "ContextExhaustionWithInvariantRisk".to_string(),
            "context_invariants".to_string(),
            format!("Required {} tokens, available {} tokens", required_tokens, available_tokens),
        ),
    };

    let details_struct = SemanticGuardDetails {
        violation_type: violation_type.clone(),
        target_item: target_item.clone(),
        details: details.clone(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Semantic Guard Intercepted: {}", violation_type);
    let message = format!("{}: {}", target_item, details);

    let event = NotificationEvent::new(
        NotificationCategory::SemanticGuard,
        NotificationSeverity::Critical,
        title,
        message,
        action_taken,
        NotificationPayload::SemanticGuard(details_struct),
    );

    hub().emit(event);
}

/// Emit a Model Auto-Healing notification
pub fn notify_model_auto_healed(
    requested_model: &str,
    healed_model: &str,
    provider: &str,
    reason: &str,
    action_taken: &str,
) {
    let details = AutoHealDetails {
        requested_model: requested_model.to_string(),
        healed_model: healed_model.to_string(),
        provider: provider.to_string(),
        reason: reason.to_string(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Model Auto-Healed: '{}' ➔ '{}'", requested_model, healed_model);
    let message = format!(
        "Requested model '{}' unavailable on '{}'. Automatically healed to '{}'.",
        requested_model, provider, healed_model
    );

    let event = NotificationEvent::new(
        NotificationCategory::ModelAutoHeal,
        NotificationSeverity::Warning,
        title,
        message,
        action_taken,
        NotificationPayload::AutoHeal(details),
    );

    hub().emit(event);
}

/// Emit an Offline / Local-Only Lock Override alert
pub fn notify_offline_lock(
    requested_provider: &str,
    enforced_provider: &str,
    lock_reason: &str,
    action_taken: &str,
) {
    let details = OfflineLockDetails {
        requested_provider: requested_provider.to_string(),
        enforced_provider: enforced_provider.to_string(),
        lock_reason: lock_reason.to_string(),
        action_taken: action_taken.to_string(),
    };

    let title = format!("Offline Lock Active: Disallowed '{}'", requested_provider);
    let message = format!(
        "Requested provider '{}' overridden to '{}' under offline / local-only lock: {}",
        requested_provider, enforced_provider, lock_reason
    );

    let event = NotificationEvent::new(
        NotificationCategory::OfflineLockOverride,
        NotificationSeverity::SecurityAlert,
        title,
        message,
        action_taken,
        NotificationPayload::OfflineLock(details),
    );

    hub().emit(event);
}

/// Emit a Cyber Defense / Nation-State Security Alert
pub fn notify_cyber_defense_alert(
    threat_actor: &str,
    attack_vector: &str,
    target: &str,
    indicator: &str,
    threat_level: &str,
    action_taken: &str,
    mitigation_remediation: &str,
) {
    let details = CyberDefenseAlertDetails {
        threat_actor: threat_actor.to_string(),
        attack_vector: attack_vector.to_string(),
        target: target.to_string(),
        indicator: indicator.to_string(),
        threat_level: threat_level.to_string(),
        action_taken: action_taken.to_string(),
        mitigation_remediation: mitigation_remediation.to_string(),
    };

    let title = format!("CYBER DEFENSE INTERCEPTION: [{}] {}", threat_actor, attack_vector);
    let message = format!(
        "Nation-state attack pattern '{}' targeting '{}' intercepted. Indicator: {}",
        attack_vector, target, indicator
    );

    let severity = match threat_level.to_lowercase().as_str() {
        "critical" | "emergency" => NotificationSeverity::Critical,
        "high" | "securityalert" | "security_alert" => NotificationSeverity::SecurityAlert,
        "warning" | "medium" => NotificationSeverity::Warning,
        _ => NotificationSeverity::Info,
    };

    let event = NotificationEvent::new(
        NotificationCategory::SecurityAlert,
        severity,
        title,
        message,
        action_taken,
        NotificationPayload::CyberDefenseAlert(details),
    );

    hub().emit(event);
}

/// Emit a generic system alert notification
pub fn notify_system_alert(title: &str, message: &str) {
    let mut map = std::collections::HashMap::new();
    map.insert("alert".to_string(), title.to_string());
    map.insert("details".to_string(), message.to_string());

    let event = NotificationEvent::new(
        NotificationCategory::SystemAlert,
        NotificationSeverity::Info,
        title,
        message,
        "Displaying user system notification",
        NotificationPayload::Generic(map),
    );

    hub().emit(event);
}

/// Get a snapshot copy of the full notification history
pub fn history() -> Vec<NotificationEvent> {
    hub().history()
}

/// Clear history records
pub fn clear_history() {
    hub().clear_history();
}

/// Get events by category
pub fn get_events_by_category(category: NotificationCategory) -> Vec<NotificationEvent> {
    hub().filter_by_category(category)
}

/// Get events by severity
pub fn get_events_by_severity(severity: NotificationSeverity) -> Vec<NotificationEvent> {
    hub().filter_by_severity(severity)
}

/// Subscribe to live notifications
pub fn subscribe() -> broadcast::Receiver<NotificationEvent> {
    hub().subscribe()
}

/// Enable or disable desktop notifications
pub fn set_desktop_enabled(enabled: bool) {
    hub().set_desktop_enabled(enabled);
}

/// Enable or disable ANSI banners to stderr
pub fn set_banner_enabled(enabled: bool) {
    hub().set_banner_enabled(enabled);
}

/// Return total count of desktop delivery notifications
pub fn desktop_delivery_count() -> usize {
    hub().desktop_delivery_count()
}

/// Return total count of terminal banners rendered
pub fn terminal_banner_count() -> usize {
    hub().terminal_banner_count()
}

/// Check whether desktop notifications are enabled in-memory
pub fn is_desktop_enabled() -> bool {
    hub().desktop_enabled.load(Ordering::Relaxed)
}

/// Helper to determine if desktop notifications are suppressed via env or config
pub fn is_desktop_suppressed() -> bool {
    !NotificationHub::detect_desktop_enabled_default() || !hub().desktop_enabled.load(Ordering::Relaxed)
}

/// CLI action for `tgs notify`
#[derive(clap::Subcommand, Debug, Clone)]
pub enum NotifyAction {
    /// Show notification configuration, delivery statistics, and history
    Status,
    /// Disable and mute desktop screen popups (persists to ~/.config/tagisan/config.json)
    Off,
    /// Re-enable desktop screen popups
    On,
    /// Mute desktop screen popups (alias for off)
    Mute,
    /// Send a non-intrusive 2.5-second test alert
    Test,
    /// Clear in-memory notification history
    Clear,
}

/// Main execution handler for `tgs notify` CLI command
pub fn handle_notify_command(action: NotifyAction) -> crate::error::Result<()> {
    match action {
        NotifyAction::Status => {
            let active = is_desktop_enabled() && !is_desktop_suppressed();
            println!("{}", "═════════════════════════════════════════════════════════════".cyan());
            println!("{}", "  🔔 TAGISAN NOTIFICATION SUBSYSTEM STATUS".bold().yellow());
            println!("{}", "═════════════════════════════════════════════════════════════".cyan());
            println!(
                "  Desktop Screen Popups : {}",
                if active {
                    "ENABLED (Auto-dismiss 2.5s)".green().bold()
                } else {
                    "MUTED / DISABLED (Screen clean)".red().bold()
                }
            );
            println!(
                "  ANSI Terminal Banners : {}",
                if hub().banner_enabled.load(Ordering::Relaxed) {
                    "ENABLED".green()
                } else {
                    "DISABLED".dimmed()
                }
            );
            println!("  Desktop Delivery Count: {}", hub().desktop_delivery_count());
            println!("  Terminal Banner Count : {}", hub().terminal_banner_count());
            println!("  History Events Stored : {}", hub().history_len());
            println!();
            println!("  Config File           : ~/.config/tagisan/config.json");
            println!(
                "  TAGISAN_NO_DESKTOP_NOTIFY: {:?}",
                std::env::var("TAGISAN_NO_DESKTOP_NOTIFY").ok()
            );
        }
        NotifyAction::Off | NotifyAction::Mute => {
            set_desktop_enabled(false);
            if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                let dir = std::path::Path::new(&home).join(".config").join("tagisan");
                let _ = std::fs::create_dir_all(&dir);
                let path = dir.join("config.json");
                let mut map = serde_json::Map::new();
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(obj) = val.as_object() {
                                map = obj.clone();
                            }
                        }
                    }
                }
                map.insert("desktop_notifications".to_string(), serde_json::Value::Bool(false));
                let _ = std::fs::write(&path, serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default());
            }
            println!("{} Desktop notifications muted. Tagisan will not display popups on your screen.", "✔".green().bold());
        }
        NotifyAction::On => {
            set_desktop_enabled(true);
            if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                let dir = std::path::Path::new(&home).join(".config").join("tagisan");
                let _ = std::fs::create_dir_all(&dir);
                let path = dir.join("config.json");
                let mut map = serde_json::Map::new();
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(obj) = val.as_object() {
                                map = obj.clone();
                            }
                        }
                    }
                }
                map.insert("desktop_notifications".to_string(), serde_json::Value::Bool(true));
                let _ = std::fs::write(&path, serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default());
            }
            println!("{} Desktop notifications enabled (with 2.5-second auto-expiration).", "✔".green().bold());
        }
        NotifyAction::Test => {
            if is_desktop_suppressed() {
                println!("{} Desktop notifications are currently muted. Run 'tgs notify on' to enable first.", "ℹ".yellow());
            } else {
                println!("Emitting test notification...");
                crate::notify::notify_system_alert("Test Notification", "This is a non-intrusive 2.5-second test alert from Tagisan.");
            }
        }
        NotifyAction::Clear => {
            hub().clear_history();
            println!("{} Notification history cleared.", "✔".green().bold());
        }
    }
    Ok(())
}
