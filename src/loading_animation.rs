/// Spinning Loading Animation
/// Shows a simple ASCII spinner animation during long operations
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub struct LoadingAnimation {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl LoadingAnimation {
    pub fn new(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();
        
        let handle = thread::spawn(move || {
            let mut frame = 0;
            
            // Print initial message to stderr (won't freeze GUI)
            eprintln!("\n[*] Large file detected - compilation in progress...");
            eprintln!("{}", message);
            
            while running_clone.load(Ordering::Relaxed) {
                // Just print a simple progress indicator to stderr
                // Don't use ANSI codes or emojis that don't work in Windows console
                let spinner_frames = ["|", "/", "-", "\\"];
                eprint!("\r[{}] Compiling... ", spinner_frames[frame % spinner_frames.len()]);
                
                frame += 1;
                thread::sleep(Duration::from_millis(250));
            }
            
            // Clear the progress line
            eprintln!("\r[OK] Compilation phase complete!                    ");
        });
        
        Self {
            running,
            handle: Some(handle),
        }
    }
    
    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for LoadingAnimation {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Progress tracker for assembly operations
pub struct AssemblyProgress {
    pub phase: Arc<Mutex<String>>,
    pub lines_processed: Arc<Mutex<usize>>,
    pub total_lines: Arc<Mutex<usize>>,
}

impl AssemblyProgress {
    pub fn new() -> Self {
        Self {
            phase: Arc::new(Mutex::new("Initializing...".to_string())),
            lines_processed: Arc::new(Mutex::new(0)),
            total_lines: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn set_phase(&self, phase: &str) {
        if let Ok(mut p) = self.phase.lock() {
            *p = phase.to_string();
        }
    }
    
    pub fn set_total_lines(&self, total: usize) {
        if let Ok(mut t) = self.total_lines.lock() {
            *t = total;
        }
    }
    
    pub fn increment_lines(&self) {
        if let Ok(mut l) = self.lines_processed.lock() {
            *l += 1;
        }
    }
    
    pub fn get_status(&self) -> String {
        let phase = self.phase.lock().ok().map(|p| p.clone()).unwrap_or_default();
        let processed = self.lines_processed.lock().ok().map(|p| *p).unwrap_or(0);
        let total = self.total_lines.lock().ok().map(|t| *t).unwrap_or(0);
        
        if total > 0 {
            format!("{} - {}/{} lines", phase, processed, total)
        } else {
            phase
        }
    }
}