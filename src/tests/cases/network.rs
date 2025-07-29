#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::dns;
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    };
    use tokio::time::Instant;

    #[tokio::test]
    async fn retry_should_succeed_after_3_attempts() {
        let counter = Arc::new(AtomicU8::new(0));
        let counter_clone = counter.clone();

        let start = Instant::now();

        let result = dns::retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let attempt = counter.fetch_add(1, Ordering::SeqCst);
                    if attempt >= 2 {
                        Some("Success".to_string())
                    } else {
                        None
                    }
                }
            },
            5,
            std::time::Duration::from_millis(50),
        )
        .await;

        let duration = start.elapsed();

        assert_eq!(result, Some("Success".to_string()));
        assert!(duration >= std::time::Duration::from_millis(50 + 100)); // 2 delays
        assert!(counter.load(Ordering::SeqCst) == 3); // called 3 times
    }

    #[tokio::test]
    async fn retry_should_fail_after_max_retries() {
        let counter = Arc::new(AtomicU8::new(0));
        let counter_clone = counter.clone();

        let result: Option<()> = dns::retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    None
                }
            },
            3,
            std::time::Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, None);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // initial try + 3 retries
    }
}
