use colored::Colorize;
use std::io::{stderr, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Standard braille spinner frames for smooth non-jittering animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// An animated terminal spinner for long-running TGS operations.
/// Shows a live rotating icon, operation description, and real-time elapsed timer.
pub struct Spinner {
    message: Arc<Mutex<String>>,
    active: Arc<AtomicBool>,
    handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    is_tty: bool,
}

impl Spinner {
    /// Start a new animated spinner with the given initial message.
    pub fn start(message: impl Into<String>) -> Self {
        let msg = message.into();
        let is_tty = stderr().is_terminal();
        let active = Arc::new(AtomicBool::new(true));
        let message_lock = Arc::new(Mutex::new(msg.clone()));
        let handle_holder = Arc::new(Mutex::new(None));

        if !is_tty {
            return Self {
                message: message_lock,
                active: Arc::new(AtomicBool::new(false)),
                handle: handle_holder,
                is_tty: false,
            };
        }

        let active_clone = active.clone();
        let msg_clone = message_lock.clone();

        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let mut frame_idx = 0;
            // Hide cursor
            eprint!("\x1b[?25l");
            let _ = stderr().flush();

            while active_clone.load(Ordering::Relaxed) {
                let frame = SPINNER_FRAMES[frame_idx % SPINNER_FRAMES.len()];
                let current_msg = {
                    let guard = msg_clone.lock().unwrap();
                    guard.clone()
                };
                let elapsed_secs = start.elapsed().as_secs();
                let mins = elapsed_secs / 60;
                let secs = elapsed_secs % 60;
                let timer = format!("[{:02}:{:02}]", mins, secs);

                // Alternating vibrant colors: Cyan -> Magenta -> Yellow
                let colored_frame = match (frame_idx / 2) % 3 {
                    0 => frame.cyan().bold(),
                    1 => frame.magenta().bold(),
                    _ => frame.yellow().bold(),
                };

                // Clear line, print spinner, message, and elapsed timer
                eprint!(
                    "\r\x1b[2K  {} {} {}",
                    colored_frame,
                    current_msg.bold(),
                    timer.dimmed()
                );
                let _ = stderr().flush();

                frame_idx += 1;
                tokio::time::sleep(Duration::from_millis(80)).await;
            }

            // Restore cursor
            eprint!("\x1b[?25h");
            let _ = stderr().flush();
        });

        if let Ok(mut h) = handle_holder.lock() {
            *h = Some(handle);
        }

        Self {
            message: message_lock,
            active,
            handle: handle_holder,
            is_tty: true,
        }
    }

    /// Update the message displayed beside the spinner.
    pub fn set_message(&self, new_message: impl Into<String>) {
        if let Ok(mut guard) = self.message.lock() {
            *guard = new_message.into();
        }
    }

    /// Stop the spinner and print a success message with a checkmark.
    pub fn success(&self, message: impl Into<String>) {
        self.stop_with_symbol("✔".green().bold(), message.into());
    }

    /// Stop the spinner and print a failure message with an error mark.
    pub fn failure(&self, message: impl Into<String>) {
        self.stop_with_symbol("✖".red().bold(), message.into());
    }

    /// Stop the spinner and clear the line without leaving a trace.
    pub fn stop(&self) {
        if self.active.swap(false, Ordering::SeqCst) {
            if let Ok(mut h) = self.handle.lock() {
                if let Some(handle) = h.take() {
                    handle.abort();
                }
            }
            if self.is_tty {
                eprint!("\r\x1b[2K\x1b[?25h");
                let _ = stderr().flush();
            }
        }
    }

    fn stop_with_symbol(&self, symbol: colored::ColoredString, message: String) {
        if self.active.swap(false, Ordering::SeqCst) {
            if let Ok(mut h) = self.handle.lock() {
                if let Some(handle) = h.take() {
                    handle.abort();
                }
            }
            if self.is_tty {
                eprintln!("\r\x1b[2K  {} {}", symbol, message);
                eprint!("\x1b[?25h");
                let _ = stderr().flush();
            }
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spinner_lifecycle() {
        let spinner = Spinner::start("Test running task...");
        spinner.set_message("Test updated task...");
        tokio::time::sleep(Duration::from_millis(50)).await;
        spinner.success("Test succeeded");
    }

    #[tokio::test]
    async fn test_spinner_failure() {
        let spinner = Spinner::start("Test failing task...");
        tokio::time::sleep(Duration::from_millis(20)).await;
        spinner.failure("Test failed");
    }

    #[tokio::test]
    async fn test_spinner_drop() {
        let spinner = Spinner::start("Test drop task...");
        drop(spinner);
    }
}
