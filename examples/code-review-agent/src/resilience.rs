//! Resilience patterns: retry with exponential backoff and circuit breaker.

use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::Result;
use tracing::{debug, warn};

/// Configuration for retry with exponential backoff.
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 means no retries, just the initial attempt).
    pub max_retries: u32,
    /// Base delay in milliseconds before the first retry.
    pub base_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
        }
    }
}

/// Maximum cap for backoff delay (30 seconds).
const MAX_BACKOFF_MS: u64 = 30_000;

/// Execute an async operation with retry and exponential backoff.
///
/// The operation is called up to `1 + config.max_retries` times.
/// Delay between attempts: `base_delay_ms * 2^attempt`, capped at 30 seconds.
pub async fn with_retry<F, Fut, T>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(value) => {
                if attempt > 0 {
                    debug!(attempt, "Operation succeeded after retry");
                }
                return Ok(value);
            }
            Err(e) => {
                if attempt < config.max_retries {
                    let delay_ms = config
                        .base_delay_ms
                        .saturating_mul(2u64.saturating_pow(attempt))
                        .min(MAX_BACKOFF_MS);
                    warn!(
                        attempt,
                        max_retries = config.max_retries,
                        delay_ms,
                        error = %e,
                        "Operation failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("retry loop completed with no attempts")))
}

/// A simple circuit breaker that trips after consecutive failures.
///
/// Once the failure count reaches `max_failures`, the circuit opens and
/// `check()` returns `false` until `record_success()` resets it.
pub struct CircuitBreaker {
    /// Failure threshold.
    max_failures: u32,
    /// Current consecutive failure count (atomic for thread safety).
    failures: AtomicU32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given failure threshold.
    /// `max_failures` must be >= 1 (clamped to 1 if 0 is passed).
    pub fn new(max_failures: u32) -> Self {
        Self {
            max_failures: max_failures.max(1),
            failures: AtomicU32::new(0),
        }
    }

    /// Check whether the circuit is closed (healthy).
    ///
    /// Returns `true` if operations should proceed, `false` if the circuit is open.
    pub fn check(&self) -> bool {
        self.failures.load(Ordering::Relaxed) < self.max_failures
    }

    /// Record a failure, incrementing the counter toward the threshold.
    ///
    /// Uses `fetch_update` with saturating add to prevent u32 overflow.
    /// Only logs the "tripped" warning on the exact transition (prev < threshold,
    /// new >= threshold), avoiding spurious warnings on repeated failures.
    pub fn record_failure(&self) {
        let prev = self
            .failures
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_add(1))
            })
            .unwrap_or_else(|v| v);
        let new = prev.saturating_add(1);
        // Only warn on the exact threshold crossing, not on every subsequent failure.
        if new >= self.max_failures && prev < self.max_failures {
            warn!(
                failures = new,
                threshold = self.max_failures,
                "Circuit breaker tripped — circuit is now OPEN"
            );
        }
    }

    /// Record a success, resetting the failure counter to zero.
    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }

    /// Current failure count.
    #[cfg(test)]
    pub fn failure_count(&self) -> u32 {
        self.failures.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_succeeds_immediately() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
        };
        let result = with_retry(&config, || async { Ok::<_, anyhow::Error>(42) }).await;
        assert_eq!(result.expect("should succeed"), 42);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
        };

        let attempt = std::sync::Arc::new(AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result = with_retry(&config, move || {
            let attempt = attempt_clone.clone();
            async move {
                let n = attempt.fetch_add(1, Ordering::Relaxed);
                if n < 2 {
                    anyhow::bail!("transient failure {n}")
                } else {
                    Ok(99)
                }
            }
        })
        .await;

        assert_eq!(result.expect("should succeed on 3rd attempt"), 99);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
        };

        let result =
            with_retry(&config, || async { Err::<i32, _>(anyhow::anyhow!("always fails")) }).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_breaker_closed_initially() {
        let cb = CircuitBreaker::new(3);
        assert!(cb.check());
    }

    #[test]
    fn test_circuit_breaker_opens_at_threshold() {
        let cb = CircuitBreaker::new(2);
        cb.record_failure();
        assert!(cb.check()); // 1 < 2
        cb.record_failure();
        assert!(!cb.check()); // 2 >= 2, circuit open
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new(2);
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.check());
        cb.record_success();
        assert!(cb.check());
        assert_eq!(cb.failure_count(), 0);
    }
}
