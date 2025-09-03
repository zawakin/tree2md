use std::sync::{Arc, Mutex};

/// Progress tracker for tree building
#[derive(Clone)]
pub struct ProgressTracker {
    inner: Arc<Mutex<ProgressState>>,
}

struct ProgressState {
    total_files: usize,
    processed_files: usize,
    total_dirs: usize,
    processed_dirs: usize,
    #[allow(dead_code)]
    current_path: Option<String>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressState {
                total_files: 0,
                processed_files: 0,
                total_dirs: 0,
                processed_dirs: 0,
                current_path: None,
            })),
        }
    }

    /// Estimate total items (called after initial walk)
    #[allow(dead_code)]
    pub fn set_estimated_total(&self, files: usize, dirs: usize) {
        if let Ok(mut state) = self.inner.lock() {
            state.total_files = files;
            state.total_dirs = dirs;
        }
    }

    /// Mark a file as processed
    #[allow(dead_code)]
    pub fn process_file(&self, path: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state.processed_files += 1;
            state.current_path = Some(path.to_string());
        }
    }

    /// Mark a directory as processed
    #[allow(dead_code)]
    pub fn process_dir(&self, path: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state.processed_dirs += 1;
            state.current_path = Some(path.to_string());
        }
    }

    /// Get current progress as a percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        if let Ok(state) = self.inner.lock() {
            let total = state.total_files + state.total_dirs;
            let processed = state.processed_files + state.processed_dirs;

            if total == 0 {
                return 0.0;
            }

            (processed as f32 / total as f32).min(1.0)
        } else {
            0.0
        }
    }

    /// Get current processing path
    #[allow(dead_code)]
    pub fn get_current_path(&self) -> Option<String> {
        if let Ok(state) = self.inner.lock() {
            state.current_path.clone()
        } else {
            None
        }
    }

    /// Get detailed stats
    #[allow(dead_code)]
    pub fn get_stats(&self) -> (usize, usize, usize, usize) {
        if let Ok(state) = self.inner.lock() {
            (
                state.processed_files,
                state.total_files,
                state.processed_dirs,
                state.total_dirs,
            )
        } else {
            (0, 0, 0, 0)
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}
