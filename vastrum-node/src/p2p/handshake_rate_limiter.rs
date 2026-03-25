const WINDOW: Duration = Duration::from_secs(60);
const MAX_PER_IP: u32 = 20;
const MAX_PER_SUBNET: u32 = 80;
const MAX_CONCURRENT_HANDSHAKES: usize = 128;
const MAX_TRACKED_ENTRIES: usize = 10_000;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(30);

struct RateEntry {
    count: u32,
    window_start: Instant,
}

pub struct HandshakeRateLimiter {
    per_ip: HashMap<IpAddr, RateEntry>,
    per_subnet: HashMap<IpAddr, RateEntry>,
    last_cleanup: Instant,
    max_concurrent: Arc<Semaphore>,
}

impl HandshakeRateLimiter {
    pub fn new() -> Self {
        HandshakeRateLimiter {
            per_ip: HashMap::new(),
            per_subnet: HashMap::new(),
            last_cleanup: Instant::now(),
            max_concurrent: Arc::new(Semaphore::new(MAX_CONCURRENT_HANDSHAKES)),
        }
    }

    pub fn can_accept_handshake(&mut self, ip: IpAddr) -> Option<OwnedSemaphorePermit> {
        self.maybe_cleanup();
        let now = Instant::now();
        let subnet = subnet_key(ip);

        if !Self::would_allow(&self.per_ip, ip, MAX_PER_IP, now) {
            return None;
        }
        if !Self::would_allow(&self.per_subnet, subnet, MAX_PER_SUBNET, now) {
            return None;
        }
        let permit = self.max_concurrent.clone().try_acquire_owned().ok()?;
        Self::increment(&mut self.per_ip, ip, now);
        Self::increment(&mut self.per_subnet, subnet, now);
        return Some(permit);
    }

    fn would_allow(
        map: &HashMap<IpAddr, RateEntry>,
        key: IpAddr,
        limit: u32,
        now: Instant,
    ) -> bool {
        let Some(entry) = map.get(&key) else {
            return map.len() < MAX_TRACKED_ENTRIES;
        };
        if now.duration_since(entry.window_start) >= WINDOW {
            return true;
        }
        return entry.count < limit;
    }

    fn increment(map: &mut HashMap<IpAddr, RateEntry>, key: IpAddr, now: Instant) {
        let entry = map.entry(key).or_insert(RateEntry { count: 0, window_start: now });
        if now.duration_since(entry.window_start) >= WINDOW {
            entry.count = 1;
            entry.window_start = now;
        } else {
            entry.count += 1;
        }
    }

    fn maybe_cleanup(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup) < CLEANUP_INTERVAL {
            return;
        }
        self.last_cleanup = now;
        self.per_ip.retain(|_, e| now.duration_since(e.window_start) < WINDOW);
        self.per_subnet.retain(|_, e| now.duration_since(e.window_start) < WINDOW);
    }
}

fn subnet_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 0))
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            IpAddr::V6(Ipv6Addr::new(segments[0], segments[1], segments[2], 0, 0, 0, 0, 0))
        }
    }
}

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;
