use std::ops::Mul;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TokenBucket {
    rate: u32,
    capacity: usize,
    tokens: usize,
    last_time: Instant,
}

impl TokenBucket {
    pub fn new(rate: u32, capacity: usize, initial_tokens: Option<usize>) -> Self {
        let last_time = Instant::now();
        Self {
            rate,
            capacity,
            tokens: initial_tokens.unwrap_or(capacity).min(capacity),
            last_time,
        }
    }

    pub fn consume(&mut self, tokens: usize) -> bool {
        if tokens == 0 {
            panic!("tokens should be a positive number")
        }

        println!(
            "Consuming {} tokens. Current tokens: {}",
            tokens, self.tokens
        );
        println!("self: {:?}", self);

        // refill before consuming tokens
        self.refill();
        println!("self - 2: {:?}", self);
        if tokens <= self.tokens {
            self.tokens -= tokens;
            return true;
        }
        false
    }

    fn refill(&mut self) {
        let time_now = Instant::now();

        if self.last_time > time_now {
            return;
        }

        println!("Refilling tokens. Current tokens: {}", self.tokens);
        // tokens to add = (time now - last time) * rate
        // total tokens = min(capacity, current tokens + tokens to add)
        let time_delta = time_now.duration_since(self.last_time).as_secs_f64();
        let token_to_add = time_delta.mul(self.rate as f64).trunc() as u32;
        self.tokens = self
            .tokens
            .saturating_add(token_to_add as usize)
            .min(self.capacity);
        self.last_time = time_now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    fn make_bucket(rate: u32, capacity: usize, initial: Option<usize>) -> TokenBucket {
        TokenBucket::new(rate, capacity, initial)
    }

    #[test]
    fn test_initial_tokens_default_to_capacity() {
        let b = make_bucket(10, 5, None);
        assert_eq!(b.tokens, 5);
    }

    #[test]
    fn test_initial_tokens_respects_value() {
        let b = make_bucket(10, 5, Some(3));
        assert_eq!(b.tokens, 3);
    }

    #[test]
    fn test_initial_tokens_cannot_exceed_capacity() {
        let b = make_bucket(10, 5, Some(10));
        assert_eq!(b.tokens, 5);
    }

    #[test]
    fn test_consume_success() {
        let mut b = make_bucket(10, 5, None);
        assert!(b.consume(1));
        assert_eq!(b.tokens, 2);
    }

    #[test]
    fn test_consume_failure() {
        let mut b = make_bucket(10, 5, Some(2));
        assert!(!b.consume(6));
        assert_eq!(b.tokens, 5);
    }

    #[test]
    fn test_refill_adds_tokens() {
        let mut b = make_bucket(10, 10, Some(0));
        assert_eq!(b.tokens, 0);

        sleep(Duration::from_millis(200)); // 0.2s

        b.refill();
        // rate = 10 tokens per sec, so after 0.2 sec we expect ~2 tokens
        assert!(b.tokens >= 1 && b.tokens <= 3, "tokens={}", b.tokens);
    }

    #[test]
    fn test_refill_never_exceeds_capacity() {
        let mut b = make_bucket(1000, 5, Some(0));

        sleep(Duration::from_secs(1));
        b.refill();

        assert_eq!(b.tokens, 5);
    }

    #[test]
    #[should_panic(expected = "tokens should be a positive number")]
    fn test_consume_zero_panics() {
        let mut b = make_bucket(10, 5, None);
        b.consume(0);
    }

    #[test]
    fn test_multiple_refills_accumulate() {
        let mut b = make_bucket(5, 10, Some(0));

        sleep(Duration::from_millis(200));
        b.refill();
        let first_tokens = b.tokens;

        sleep(Duration::from_millis(200));
        b.refill();
        assert!(b.tokens > first_tokens);
    }
}
