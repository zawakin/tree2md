use crate::fs_tree::ProgressTracker;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Simple progress animation for terminal output
pub struct ProgressAnimation {
    message: String,
    enabled: bool,
    start_time: Option<Instant>,
    last_update: Option<Instant>,
    #[allow(dead_code)]
    spinner_frames: Vec<&'static str>,
    #[allow(dead_code)]
    current_frame: usize,
}

impl ProgressAnimation {
    /// Create a new progress animation
    pub fn new(message: String, enabled: bool) -> Self {
        Self {
            message,
            enabled,
            start_time: None,
            last_update: None,
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
        }
    }

    /// Create a tree-growing animation
    pub fn tree_growing(enabled: bool) -> Self {
        Self::new("Growing your tree...".to_string(), enabled)
    }

    /// Start the animation
    pub fn start(&mut self) {
        if !self.enabled {
            return;
        }

        self.start_time = Some(Instant::now());
        self.last_update = Some(Instant::now());
        self.draw_initial();
    }

    /// Update the animation (call this periodically during work)
    #[allow(dead_code)]
    pub fn update(&mut self, progress_percent: Option<f32>) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_update {
            // Only update every 100ms to avoid flicker
            if now.duration_since(last) < Duration::from_millis(100) {
                return;
            }
        }

        self.last_update = Some(now);
        self.current_frame = (self.current_frame + 1) % self.spinner_frames.len();

        self.draw_progress(progress_percent);
    }

    /// Complete the animation with a final message
    pub fn complete(&mut self, message: Option<&str>) {
        if !self.enabled {
            return;
        }

        self.clear_line();

        if let Some(msg) = message {
            println!("🎄 {}", msg);
        } else {
            println!("🎄 Tree complete! Happy coding!");
        }

        let _ = io::stdout().flush();
    }

    /// Draw the initial animation frame
    fn draw_initial(&self) {
        if !self.enabled {
            return;
        }

        print!("\r🌳 {} ", self.message);
        let _ = io::stdout().flush();
    }

    /// Draw a progress update
    #[allow(dead_code)]
    fn draw_progress(&self, progress_percent: Option<f32>) {
        if !self.enabled {
            return;
        }

        self.clear_line();

        if let Some(percent) = progress_percent {
            // Draw with progress bar
            let bar_width = 15;
            let filled = ((percent * bar_width as f32).round() as usize).min(bar_width);
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));

            print!("\r🌳 {} [{}] {:.0}%", self.message, bar, percent * 100.0);
        } else {
            // Draw with spinner
            print!(
                "\r{} {} 🌱",
                self.spinner_frames[self.current_frame], self.message
            );
        }

        let _ = io::stdout().flush();
    }

    /// Clear the current line
    fn clear_line(&self) {
        if !self.enabled {
            return;
        }

        // Move to start of line and clear it
        print!("\r{}", " ".repeat(80));
        print!("\r");
        let _ = io::stdout().flush();
    }
}

impl Drop for ProgressAnimation {
    fn drop(&mut self) {
        if self.enabled && self.start_time.is_some() {
            self.clear_line();
        }
    }
}

/// Simple non-blocking animation runner
pub struct AnimationRunner {
    animation: Option<ProgressAnimation>,
    progress_tracker: Option<ProgressTracker>,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl AnimationRunner {
    pub fn new(enabled: bool, progress_tracker: Option<ProgressTracker>) -> Self {
        let mut runner = Self {
            animation: None,
            progress_tracker,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        };

        if enabled {
            runner.start();
        }

        runner
    }

    /// Start the animation in a background thread
    pub fn start(&mut self) {
        if self.animation.is_some() {
            return; // Already running
        }

        let mut animation = ProgressAnimation::tree_growing(true);
        animation.start();

        if let Some(tracker) = &self.progress_tracker {
            let tracker = tracker.clone();
            let running = self.running.clone();
            running.store(true, Ordering::SeqCst);

            // Start a background thread to update the animation
            let handle = thread::spawn(move || {
                let mut last_update = Instant::now();

                while running.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    if now.duration_since(last_update) >= Duration::from_millis(100) {
                        let progress = tracker.get_progress();
                        // Since we can't move animation into thread, we just update progress
                        // In a real implementation, we'd use channels or Arc<Mutex>
                        print!("\r🌳 Growing your tree... [{:>3.0}%]", progress * 100.0);
                        let _ = io::stdout().flush();
                        last_update = now;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            });

            self.thread_handle = Some(handle);
        }

        self.animation = Some(animation);
    }

    #[allow(dead_code)]
    pub fn update(&mut self, progress: f32) {
        if let Some(anim) = &mut self.animation {
            anim.update(Some(progress));
        }
    }

    pub fn complete(&mut self) {
        // Stop the background thread
        self.running.store(false, Ordering::SeqCst);

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Clear line and show completion message
        if let Some(mut anim) = self.animation.take() {
            anim.complete(None);
        }
    }
}

impl Drop for AnimationRunner {
    fn drop(&mut self) {
        self.complete();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_animation_runner_basic() {
        // Test with disabled animation - just ensure it doesn't panic
        let _runner = AnimationRunner::new(false, None);

        // Test with enabled animation but we can't check internals
        // Just ensure it doesn't panic
        let _runner = AnimationRunner::new(true, None);
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn test_progress_animation_basic() {
        // Test ProgressAnimation creation
        let _animation = ProgressAnimation::new("Testing...".to_string(), false);
        // Can't test much without access to internals
    }
}
