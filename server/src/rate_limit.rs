//! Rate limiting: one token bucket per (route group, key), held in memory.
//! The buckets are lost at a restart, which is accepted (TS §24.5).

use std::collections::{BTreeMap, HashMap};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use offcut_types::UserId;
use tokio::time::Instant;

use crate::client_ip::client_ip;
use crate::error::AppError;
use crate::state::AppState;

/// One group per row of the limit column of TS §22.1. `POST /auth/magic-link`
/// has two limits and so two groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteGroup {
    Healthz,
    MagicLinkEmail,
    MagicLinkIp,
    Verify,
    Refresh,
    Logout,
    Me,
    Entitlement,
    UsageReceipts,
    Checkout,
    Portal,
    Webhook,
    Events,
    NotifyMe,
    AccountDelete,
    AccountExport,
}

/// `capacity` requests per `window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit {
    pub capacity: u32,
    pub window: Duration,
}

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;

const fn per(capacity: u32, window_secs: u64) -> Limit {
    Limit {
        capacity,
        window: Duration::from_secs(window_secs),
    }
}

impl RouteGroup {
    /// The whole TS §22.1 table.
    pub const fn limit(self) -> Limit {
        match self {
            Self::Healthz | Self::Events | Self::Me | Self::Entitlement => per(60, MINUTE),
            Self::MagicLinkEmail => per(3, 15 * MINUTE),
            Self::MagicLinkIp => per(10, HOUR),
            Self::Verify | Self::Checkout | Self::Portal => per(10, MINUTE),
            Self::Refresh | Self::Logout | Self::UsageReceipts => per(30, MINUTE),
            Self::Webhook => per(120, MINUTE),
            Self::NotifyMe => per(5, HOUR),
            Self::AccountDelete => per(3, HOUR),
            Self::AccountExport => per(6, HOUR),
        }
    }
}

/// Who a bucket belongs to. An email address is never a key: `EmailHash` is
/// the SHA-256 of the normalized address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RateKey {
    Ip(IpAddr),
    EmailHash([u8; 32]),
    User(UserId),
}

type BucketId = (RouteGroup, RateKey);

/// A bucket is stored as the instant at which it is full again. Each token
/// taken moves that instant `window / capacity` further away, and the passing
/// of time refills the bucket without any bookkeeping.
struct Bucket {
    full_at: Instant,
    last_used: u64,
}

#[derive(Default)]
struct Inner {
    buckets: HashMap<BucketId, Bucket>,
    /// The same buckets by `last_used`, so the least recently used is the first.
    by_last_used: BTreeMap<u64, BucketId>,
    next_seq: u64,
}

pub struct RateLimiter {
    inner: Mutex<Inner>,
    max_keys: usize,
}

impl RateLimiter {
    /// Holds at most `max_keys` buckets (50,000 in production, TS §24.5).
    pub fn new(max_keys: usize) -> RateLimiter {
        RateLimiter {
            inner: Mutex::default(),
            max_keys: max_keys.max(1),
        }
    }

    /// Takes one token from the bucket of `(group, key)`. With none left, the
    /// error is the number of seconds until one is back, rounded up.
    pub fn check(&self, group: RouteGroup, key: RateKey, now: Instant) -> Result<(), u32> {
        let limit = group.limit();
        // The time one token takes to come back.
        let Some(per_token) = limit.window.checked_div(limit.capacity) else {
            return Err(u32::MAX);
        };
        let id = (group, key);
        let mut inner = self.lock();

        let seq = inner.next_seq;
        inner.next_seq = seq.wrapping_add(1);
        // A refused request also counts as use, so a client that keeps
        // sending stays limited instead of being evicted and starting afresh.
        let previous = inner.buckets.remove(&id);
        if let Some(bucket) = &previous {
            inner.by_last_used.remove(&bucket.last_used);
        } else if inner.buckets.len() >= self.max_keys
            && let Some((_, oldest)) = inner.by_last_used.pop_first()
        {
            inner.buckets.remove(&oldest);
        }

        let refill = previous.map_or(Duration::ZERO, |bucket| {
            bucket.full_at.saturating_duration_since(now)
        });
        // How long the bucket would need to refill if this request took a token.
        let refill_after = refill.saturating_add(per_token);
        let allowed = refill_after <= limit.window;
        let refill = if allowed { refill_after } else { refill };

        let bucket = Bucket {
            full_at: now.checked_add(refill).unwrap_or(now),
            last_used: seq,
        };
        inner.buckets.insert(id, bucket);
        inner.by_last_used.insert(seq, id);

        if allowed {
            Ok(())
        } else {
            Err(ceil_secs(refill_after.saturating_sub(limit.window)))
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A poisoned lock is still used: the worst a half-finished `check`
        // leaves behind is one forgotten bucket.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn ceil_secs(wait: Duration) -> u32 {
    let secs = wait.as_secs() + u64::from(wait.subsec_nanos() > 0);
    u32::try_from(secs).unwrap_or(u32::MAX)
}

/// Middleware: limits `group` by the client's IP address. Use it through
/// `axum::middleware::from_fn_with_state`. The server must be served with
/// `into_make_service_with_connect_info::<SocketAddr>()`; without the peer
/// address the request fails with 500.
pub async fn by_ip(
    group: RouteGroup,
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let peer = req.extensions().get::<ConnectInfo<SocketAddr>>();
    let ConnectInfo(peer) = peer.ok_or(AppError::Internal)?;
    let ip = client_ip(req.headers(), peer.ip(), state.config.trusted_proxy_hops);

    let checked = state.limiter.check(group, RateKey::Ip(ip), Instant::now());
    checked.map_err(|retry_after_secs| AppError::RateLimited { retry_after_secs })?;
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use offcut_types::UserId;
    use tokio::time::advance;

    use super::*;

    const GROUPS: [RouteGroup; 16] = [
        RouteGroup::Healthz,
        RouteGroup::MagicLinkEmail,
        RouteGroup::MagicLinkIp,
        RouteGroup::Verify,
        RouteGroup::Refresh,
        RouteGroup::Logout,
        RouteGroup::Me,
        RouteGroup::Entitlement,
        RouteGroup::UsageReceipts,
        RouteGroup::Checkout,
        RouteGroup::Portal,
        RouteGroup::Webhook,
        RouteGroup::Events,
        RouteGroup::NotifyMe,
        RouteGroup::AccountDelete,
        RouteGroup::AccountExport,
    ];

    fn ip(last: u8) -> RateKey {
        RateKey::Ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, last)))
    }

    /// Sends `n` requests at the current instant and counts the accepted ones.
    fn accepted(limiter: &RateLimiter, group: RouteGroup, key: RateKey, n: u32) -> u32 {
        let now = Instant::now();
        let results = (0..n).map(|_| limiter.check(group, key, now));
        results.filter(Result::is_ok).count() as u32
    }

    #[test]
    fn every_group_has_the_limit_of_the_spec_table() {
        // (capacity, window in seconds), in the order of the enum; TS §22.1.
        let expected: [(u32, u64); 16] = [
            (60, 60),   // Healthz
            (3, 900),   // MagicLinkEmail
            (10, 3600), // MagicLinkIp
            (10, 60),   // Verify
            (30, 60),   // Refresh
            (30, 60),   // Logout
            (60, 60),   // Me
            (60, 60),   // Entitlement
            (30, 60),   // UsageReceipts
            (10, 60),   // Checkout
            (10, 60),   // Portal
            (120, 60),  // Webhook
            (60, 60),   // Events
            (5, 3600),  // NotifyMe
            (3, 3600),  // AccountDelete
            (6, 3600),  // AccountExport
        ];
        for (group, (capacity, window_secs)) in GROUPS.into_iter().zip(expected) {
            assert_eq!(group.limit(), per(capacity, window_secs), "{group:?}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn capacity_is_honoured_for_every_group() {
        let limiter = RateLimiter::new(100);
        for group in GROUPS {
            let capacity = group.limit().capacity;
            assert!(capacity > 0, "{group:?}");
            assert_eq!(accepted(&limiter, group, ip(1), capacity + 5), capacity);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn requests_spread_over_time_use_the_refill() {
        let limiter = RateLimiter::new(100);
        // One request every 500 ms: 60 are accepted within the first 30 s.
        for n in 0..60 {
            let checked = limiter.check(RouteGroup::Events, ip(1), Instant::now());
            assert_eq!(checked, Ok(()), "request {n}");
            advance(Duration::from_millis(500)).await;
        }
        // 30 tokens have come back in those 30 s, so the 61st to the 90th
        // pass as well. A burst then finds the bucket empty.
        assert_eq!(accepted(&limiter, RouteGroup::Events, ip(1), 100), 30);
        let refused = limiter.check(RouteGroup::Events, ip(1), Instant::now());
        // One token per second at 60 per minute.
        assert_eq!(refused, Err(1));
    }

    #[tokio::test(start_paused = true)]
    async fn the_61st_request_in_a_minute_is_refused_with_a_plausible_retry() {
        let limiter = RateLimiter::new(100);
        let now = Instant::now();
        for n in 0..60 {
            assert_eq!(limiter.check(RouteGroup::Events, ip(1), now), Ok(()), "{n}");
        }
        assert_eq!(limiter.check(RouteGroup::Events, ip(1), now), Err(1));
    }

    #[tokio::test(start_paused = true)]
    async fn retry_after_is_rounded_up() {
        let limiter = RateLimiter::new(100);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 5), 5);
        // 5 per hour: one token every 720 s.
        let refused =
            |limiter: &RateLimiter| limiter.check(RouteGroup::NotifyMe, ip(1), Instant::now());
        assert_eq!(refused(&limiter), Err(720));
        advance(Duration::from_millis(500)).await;
        assert_eq!(refused(&limiter), Err(720));
        advance(Duration::from_millis(500)).await;
        assert_eq!(refused(&limiter), Err(719));
        advance(Duration::from_secs(718)).await;
        assert_eq!(refused(&limiter), Err(1));
    }

    #[tokio::test(start_paused = true)]
    async fn tokens_return_as_time_advances() {
        let limiter = RateLimiter::new(100);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 6), 5);

        // One token is back after 720 s, and only one.
        advance(Duration::from_secs(719)).await;
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 1), 0);
        advance(Duration::from_secs(1)).await;
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 2), 1);

        // A refused request costs nothing: the bucket is full after one window.
        advance(Duration::from_secs(3600)).await;
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 6), 5);

        // A full bucket does not grow past its capacity.
        advance(Duration::from_secs(10 * 3600)).await;
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 6), 5);
    }

    #[tokio::test(start_paused = true)]
    async fn two_ips_do_not_share_a_bucket() {
        let limiter = RateLimiter::new(100);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 6), 5);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(2), 6), 5);
    }

    #[tokio::test(start_paused = true)]
    async fn two_groups_do_not_share_a_bucket() {
        let limiter = RateLimiter::new(100);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 6), 5);
        assert_eq!(accepted(&limiter, RouteGroup::Events, ip(1), 61), 60);
    }

    #[tokio::test(start_paused = true)]
    async fn the_three_kinds_of_key_do_not_share_a_bucket() {
        let limiter = RateLimiter::new(100);
        let user = RateKey::User(UserId::new(uuid::Uuid::from_u128(7)));
        let email = RateKey::EmailHash([7; 32]);
        for key in [ip(7), user, email] {
            assert_eq!(accepted(&limiter, RouteGroup::AccountDelete, key, 4), 3);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn eviction_keeps_the_newest_keys() {
        let limiter = RateLimiter::new(3);
        // Empty the buckets of four keys in turn. The fourth evicts the first.
        for last in 1..=4 {
            assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(last), 5), 5);
        }
        // The three newest are still empty.
        for last in 2..=4 {
            assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(last), 1), 0);
        }
        // The oldest was forgotten, so it has a full bucket again.
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 5), 5);
        assert_eq!(limiter.lock().buckets.len(), 3);
        assert_eq!(limiter.lock().by_last_used.len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn eviction_goes_by_last_use_not_by_first_use() {
        let limiter = RateLimiter::new(2);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 5), 5);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(2), 5), 5);
        // A refused request is a use: key 1 is now the newer of the two.
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 1), 0);
        // Key 3 evicts key 2.
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(3), 1), 1);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(1), 1), 0);
        assert_eq!(accepted(&limiter, RouteGroup::NotifyMe, ip(2), 5), 5);
    }
}
