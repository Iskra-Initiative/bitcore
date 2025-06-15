// -- configuration for bitcore operations

use core::time::Duration;

/// retry configuration for operations
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// maximum number of retry attempts
    pub max_attempts: usize,
    /// delay between retry attempts
    pub retry_delay: Duration,
    /// exponential backoff multiplier (1.0 = no backoff)
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay: Duration::from_millis(100),
            backoff_multiplier: 1.5,
        }
    }
}

impl RetryConfig {
    /// create new retry config with specified attempts
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// create retry config with custom delay
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// create retry config with exponential backoff
    pub fn with_backoff(mut self, multiplier: f32) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// calculate delay for given attempt number
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if (self.backoff_multiplier - 1.0).abs() < f32::EPSILON {
            self.retry_delay
        } else {
            let multiplier = self.backoff_multiplier.powi(attempt as i32);
            Duration::from_nanos((self.retry_delay.as_nanos() as f32 * multiplier) as u64)
        }
    }
}
