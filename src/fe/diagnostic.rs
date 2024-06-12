use std::time::{Duration, Instant};

pub struct Diagnostic {
    pub message: String,
    pub expires_at: Instant,
    _start_time: Instant,
}

impl Diagnostic {
    const DEFAULT_DURATION: Duration = Duration::from_secs(5);

    pub fn new(message: String, duration: Duration) -> Self {
        Self {
            message,
            expires_at: Instant::now() + duration,
            _start_time: Instant::now(),
        }
    }
    
    pub fn default(message: String) -> Self {
        Self::new(message, Self::DEFAULT_DURATION)
    }

    pub fn from_err(err: &dyn std::error::Error) -> Self {
        Self::default(format!("error: {:?}", err))
    }

    pub fn _progress(&self) -> f32 {
        let elapsed = self._start_time.elapsed().as_secs_f32();
        let total_duration = (self.expires_at - self._start_time).as_secs_f32();
        (elapsed / total_duration).min(1.0)
    }
}
