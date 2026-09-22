//! Linux suspend detection without a helper process or wall-clock dependence.
//! BOOTTIME advances during sleep; MONOTONIC does not. Their difference is
//! cumulative sleep time, independent of NTP and time-zone changes.
use std::time::Duration;

#[derive(Default)]
pub struct Monitor {
    suspended: Option<Duration>,
}
impl Monitor {
    pub fn poll(&mut self) -> bool {
        let Some(sample) = suspended_time() else {
            return false;
        };
        self.observe(sample)
    }
    fn observe(&mut self, sample: Duration) -> bool {
        let previous = self.suspended.replace(sample);
        previous.is_some_and(|old| sample.saturating_sub(old) > Duration::from_millis(100))
    }
}

fn clock(id: libc::clockid_t) -> Option<Duration> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: value is an initialized timespec with a valid writable pointer.
    if unsafe { libc::clock_gettime(id, &mut value) } != 0 {
        return None;
    }
    Some(Duration::new(
        value.tv_sec.try_into().ok()?,
        value.tv_nsec.try_into().ok()?,
    ))
}
fn suspended_time() -> Option<Duration> {
    // Bracket the sample. A preemption longer than 10 ms is retried next turn,
    // avoiding a false resume from scheduler delays between the two clocks.
    let before = clock(libc::CLOCK_MONOTONIC)?;
    let boot = clock(libc::CLOCK_BOOTTIME)?;
    let after = clock(libc::CLOCK_MONOTONIC)?;
    if after.checked_sub(before)? > Duration::from_millis(10) {
        return None;
    }
    boot.checked_sub(before)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_sleep_once_without_confusing_uptime_or_sampling_jitter() {
        let mut monitor = Monitor::default();
        assert!(!monitor.observe(Duration::from_secs(100)));
        assert!(!monitor.observe(Duration::from_millis(100_002)));
        assert!(monitor.observe(Duration::from_secs(134)));
        assert!(!monitor.observe(Duration::from_millis(134_001)));
        assert!(!monitor.observe(Duration::from_millis(134_000)));
        assert!(monitor.observe(Duration::from_secs(140)));
    }
    #[test]
    fn live_linux_clocks_are_available() {
        assert!(suspended_time().is_some());
    }
}
