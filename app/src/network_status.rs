use crate::wifi::{Connectivity, NetworkState, NetworkStatus, check_network_status};
use std::time::{Duration, Instant};

fn check_network_status_or_unknown() -> NetworkStatus {
    check_network_status().unwrap_or(NetworkStatus {
        state: NetworkState::Unknown,
        connectivity: Connectivity::Unknown,
        resolving: false,
    })
}
pub struct NetworkStatusCache {
    last_check: Instant,
    pub last_result: NetworkStatus,
    interval: Duration,
}

impl NetworkStatusCache {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_check: Instant::now() - interval,
            last_result: check_network_status_or_unknown(),
            interval,
        }
    }

    pub async fn get(&mut self) -> NetworkStatus {
        if self.last_check.elapsed() >= self.interval {
            self.last_result = check_network_status_or_unknown();
            self.last_check = Instant::now();
        }
        self.last_result
    }
}
