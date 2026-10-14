use std::{
    net::IpAddr,
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

const ACCOUNT_ATTEMPTS_PER_MINUTE: u32 = 5;
const CLIENT_ATTEMPTS_PER_MINUTE: u32 = 30;

// Periodically remove limiter keys whose state is equivalent to a fresh key.
// This prevents stale arbitrary identifiers from accumulating indefinitely.
const HOUSEKEEPING_INTERVAL: u64 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ThrottleDecision {
    Allowed,
    Throttled,
}

pub(super) struct LoginThrottle {
    accounts: DefaultKeyedRateLimiter<String>,
    client_ips: DefaultKeyedRateLimiter<Option<IpAddr>>,
    checks: AtomicU64,
}

impl LoginThrottle {
    pub(super) fn new() -> Self {
        Self {
            accounts: RateLimiter::keyed(Quota::per_minute(
                NonZeroU32::new(ACCOUNT_ATTEMPTS_PER_MINUTE)
                    .expect("account login quota must be non-zero"),
            )),

            client_ips: RateLimiter::keyed(Quota::per_minute(
                NonZeroU32::new(CLIENT_ATTEMPTS_PER_MINUTE)
                    .expect("client login quota must be non-zero"),
            )),

            checks: AtomicU64::new(0),
        }
    }

    pub(super) fn check(&self, account_key: &str, client_ip: Option<IpAddr>) -> ThrottleDecision {
        self.housekeep_if_needed();

        // Consume the client budget first. If the client has already
        // exhausted it, avoid creating additional arbitrary account keys.
        if self.client_ips.check_key(&client_ip).is_err() {
            return ThrottleDecision::Throttled;
        }

        if self.accounts.check_key(&account_key.to_owned()).is_err() {
            return ThrottleDecision::Throttled;
        }

        ThrottleDecision::Allowed
    }

    fn housekeep_if_needed(&self) {
        let check = self.checks.fetch_add(1, Ordering::Relaxed) + 1;

        if check % HOUSEKEEPING_INTERVAL == 0 {
            self.accounts.retain_recent();
            self.client_ips.retain_recent();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::{
        ACCOUNT_ATTEMPTS_PER_MINUTE, CLIENT_ATTEMPTS_PER_MINUTE, LoginThrottle, ThrottleDecision,
    };

    fn client_ip(last_octet: u8) -> Option<IpAddr> {
        Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, last_octet)))
    }

    #[test]
    fn account_budget_throttles_repeated_attempts() {
        let throttle = LoginThrottle::new();

        for _ in 0..ACCOUNT_ATTEMPTS_PER_MINUTE {
            assert_eq!(
                throttle.check("ada@example.com", client_ip(10),),
                ThrottleDecision::Allowed,
            );
        }

        // Different IP proves the account budget itself is exhausted.
        assert_eq!(
            throttle.check("ada@example.com", client_ip(11),),
            ThrottleDecision::Throttled,
        );
    }

    #[test]
    fn client_budget_spans_different_account_identifiers() {
        let throttle = LoginThrottle::new();
        let ip = client_ip(20);

        for attempt in 0..CLIENT_ATTEMPTS_PER_MINUTE {
            let account = format!("user-{attempt}@example.com");

            assert_eq!(throttle.check(&account, ip), ThrottleDecision::Allowed,);
        }

        assert_eq!(
            throttle.check("another@example.com", ip,),
            ThrottleDecision::Throttled,
        );
    }

    #[test]
    fn missing_client_ip_uses_shared_fail_safe_budget() {
        let throttle = LoginThrottle::new();

        for attempt in 0..CLIENT_ATTEMPTS_PER_MINUTE {
            let account = format!("unknown-ip-{attempt}@example.com");

            assert_eq!(throttle.check(&account, None), ThrottleDecision::Allowed,);
        }

        assert_eq!(
            throttle.check("another@example.com", None,),
            ThrottleDecision::Throttled,
        );
    }
}
