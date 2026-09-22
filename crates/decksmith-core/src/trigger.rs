//! Deterministic press-family resolver, instantiated per device/control/binding.
//! Call advance from a monotonic scheduler even when no input arrives.
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct TimingPolicy {
    pub long_ms: u64,
    pub double_ms: u64,
    pub repeat_delay_ms: u64,
    pub repeat_interval_ms: u64,
}
impl Default for TimingPolicy {
    fn default() -> Self {
        Self {
            long_ms: 500,
            double_ms: 300,
            repeat_delay_ms: 600,
            repeat_interval_ms: 100,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Bindings {
    pub double: bool,
    pub repeat: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerKind {
    RawPress,
    RawRelease,
    ShortPress,
    DoublePress,
    LongPress,
    HoldRepeat { ordinal: u64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trigger {
    pub timestamp_ms: u64,
    pub sequence: u64,
    pub kind: TriggerKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerError {
    InvalidTiming,
    TimeReversed,
    TimeOverflow,
    TooManyDueEvents,
}
impl fmt::Display for TriggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidTiming =>
                    "timings must be positive and repeat delay must not precede long press",
                Self::TimeReversed => "monotonic time moved backwards",
                Self::TimeOverflow => "timestamp or sequence overflow",
                Self::TooManyDueEvents =>
                    "more than 4096 triggers due; advance the clock in smaller steps",
            }
        )
    }
}
impl std::error::Error for TriggerError {}
#[derive(Debug, Clone)]
struct Held {
    sequence: u64,
    long_at: u64,
    long: bool,
    repeat_at: Option<u64>,
    ordinal: u64,
}
#[derive(Debug, Clone)]
pub struct PressResolver {
    policy: TimingPolicy,
    bindings: Bindings,
    now: u64,
    sequence: u64,
    held: Option<Held>,
    pending: Option<(u64, u64)>,
}
impl PressResolver {
    pub fn new(policy: TimingPolicy, bindings: Bindings) -> Result<Self, TriggerError> {
        if policy.long_ms == 0
            || policy.double_ms == 0
            || policy.repeat_interval_ms == 0
            || policy.repeat_delay_ms < policy.long_ms
        {
            return Err(TriggerError::InvalidTiming);
        }
        Ok(Self {
            policy,
            bindings,
            now: 0,
            sequence: 0,
            held: None,
            pending: None,
        })
    }
    /// Cancel state on disconnect, lock, or binding replacement. No deferred action fires.
    pub fn cancel(&mut self) {
        self.held = None;
        self.pending = None;
    }
    /// Duplicate down/up reports are ignored. Errors leave the resolver unchanged.
    pub fn edge(&mut self, timestamp_ms: u64, pressed: bool) -> Result<Vec<Trigger>, TriggerError> {
        let mut next = self.clone();
        // A release exactly at the double deadline qualifies. A release exactly
        // at long threshold is a long press; repeat at release is suppressed.
        let mut out = next.tick(timestamp_ms, true)?;
        if pressed && next.held.is_none() {
            next.sequence = next
                .sequence
                .checked_add(1)
                .ok_or(TriggerError::TimeOverflow)?;
            next.held = Some(Held {
                sequence: next.sequence,
                long_at: add(timestamp_ms, next.policy.long_ms)?,
                long: false,
                repeat_at: if next.bindings.repeat {
                    Some(add(timestamp_ms, next.policy.repeat_delay_ms)?)
                } else {
                    None
                },
                ordinal: 0,
            });
            out.push(Trigger {
                timestamp_ms,
                sequence: next.sequence,
                kind: TriggerKind::RawPress,
            });
        } else if !pressed && let Some(held) = next.held.take() {
            out.push(Trigger {
                timestamp_ms,
                sequence: held.sequence,
                kind: TriggerKind::RawRelease,
            });
            if !held.long {
                if next.pending.take().is_some() {
                    out.push(Trigger {
                        timestamp_ms,
                        sequence: held.sequence,
                        kind: TriggerKind::DoublePress,
                    });
                } else if next.bindings.double {
                    next.pending = Some((add(timestamp_ms, next.policy.double_ms)?, held.sequence));
                } else {
                    out.push(Trigger {
                        timestamp_ms,
                        sequence: held.sequence,
                        kind: TriggerKind::ShortPress,
                    });
                }
            }
        }
        *self = next;
        Ok(out)
    }
    /// Deadlines are inclusive. Input edges at a timestamp must precede advance
    /// at that same timestamp, so double-release boundary handling is consistent.
    pub fn advance(&mut self, timestamp_ms: u64) -> Result<Vec<Trigger>, TriggerError> {
        let mut next = self.clone();
        let out = next.tick(timestamp_ms, false)?;
        *self = next;
        Ok(out)
    }
    fn tick(&mut self, timestamp_ms: u64, edge: bool) -> Result<Vec<Trigger>, TriggerError> {
        if timestamp_ms < self.now {
            return Err(TriggerError::TimeReversed);
        }
        let mut out = Vec::new();
        loop {
            let pending = self.pending.map(|(t, _)| (t, 0));
            let long = self
                .held
                .as_ref()
                .filter(|h| !h.long)
                .map(|h| (h.long_at, 1));
            let repeat = self.held.as_ref().and_then(|h| h.repeat_at.map(|t| (t, 2)));
            let due = [pending, long, repeat]
                .into_iter()
                .flatten()
                .filter(|(t, k)| *t < timestamp_ms || (*t == timestamp_ms && (!edge || *k == 1)))
                .min();
            let Some((at, kind)) = due else { break };
            if out.len() >= 4096 {
                return Err(TriggerError::TooManyDueEvents);
            }
            let (sequence, kind) = match kind {
                0 => {
                    let (_, seq) = self.pending.take().unwrap();
                    (seq, TriggerKind::ShortPress)
                }
                1 => {
                    let h = self.held.as_mut().unwrap();
                    h.long = true;
                    self.pending = None;
                    (h.sequence, TriggerKind::LongPress)
                }
                _ => {
                    let h = self.held.as_mut().unwrap();
                    h.ordinal += 1;
                    h.repeat_at = Some(add(at, self.policy.repeat_interval_ms)?);
                    (h.sequence, TriggerKind::HoldRepeat { ordinal: h.ordinal })
                }
            };
            out.push(Trigger {
                timestamp_ms: at,
                sequence,
                kind,
            });
        }
        self.now = timestamp_ms;
        Ok(out)
    }
}
fn add(a: u64, b: u64) -> Result<u64, TriggerError> {
    a.checked_add(b).ok_or(TriggerError::TimeOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn resolver(double: bool, repeat: bool) -> PressResolver {
        PressResolver::new(TimingPolicy::default(), Bindings { double, repeat }).unwrap()
    }
    fn kinds(v: Vec<Trigger>) -> Vec<TriggerKind> {
        v.into_iter().map(|t| t.kind).collect()
    }
    #[test]
    fn short_without_double_is_immediate() {
        let mut r = resolver(false, false);
        r.edge(0, true).unwrap();
        assert_eq!(
            kinds(r.edge(499, false).unwrap()),
            vec![TriggerKind::RawRelease, TriggerKind::ShortPress]
        );
    }
    #[test]
    fn double_defers_single_and_completes_at_deadline() {
        let mut r = resolver(true, false);
        r.edge(0, true).unwrap();
        r.edge(50, false).unwrap();
        assert!(r.advance(349).unwrap().is_empty());
        r.edge(349, true).unwrap();
        assert_eq!(
            kinds(r.edge(350, false).unwrap()),
            vec![TriggerKind::RawRelease, TriggerKind::DoublePress]
        );
        assert!(r.advance(1000).unwrap().is_empty());
    }
    #[test]
    fn incomplete_second_press_does_not_count_as_double() {
        let mut r = resolver(true, false);
        r.edge(0, true).unwrap();
        r.edge(50, false).unwrap();
        r.edge(300, true).unwrap();
        assert_eq!(
            kinds(r.advance(350).unwrap()),
            vec![TriggerKind::ShortPress]
        );
        assert_eq!(
            kinds(r.edge(400, false).unwrap()),
            vec![TriggerKind::RawRelease]
        );
        assert_eq!(
            kinds(r.advance(700).unwrap()),
            vec![TriggerKind::ShortPress]
        );
    }
    #[test]
    fn long_threshold_suppresses_short() {
        let mut r = resolver(true, false);
        r.edge(100, true).unwrap();
        assert_eq!(
            kinds(r.edge(600, false).unwrap()),
            vec![TriggerKind::LongPress, TriggerKind::RawRelease]
        );
        assert!(r.advance(1000).unwrap().is_empty());
    }
    #[test]
    fn repeat_stops_at_release_and_preserves_deadline_timestamps() {
        let mut r = resolver(false, true);
        r.edge(0, true).unwrap();
        let v = r.advance(799).unwrap();
        assert_eq!(
            v.iter().map(|t| t.timestamp_ms).collect::<Vec<_>>(),
            vec![500, 600, 700]
        );
        assert_eq!(v[2].kind, TriggerKind::HoldRepeat { ordinal: 2 });
        assert_eq!(
            kinds(r.edge(800, false).unwrap()),
            vec![TriggerKind::RawRelease]
        );
        assert!(r.advance(2000).unwrap().is_empty());
    }
    #[test]
    fn cancel_drops_held_and_pending_actions() {
        let mut r = resolver(true, true);
        r.edge(0, true).unwrap();
        r.edge(10, false).unwrap();
        r.cancel();
        assert!(r.advance(900).unwrap().is_empty());
        r.edge(1000, true).unwrap();
        r.cancel();
        assert!(r.advance(2000).unwrap().is_empty());
    }
    #[test]
    fn duplicate_edges_do_not_restart_timing() {
        let mut r = resolver(false, false);
        r.edge(0, true).unwrap();
        assert!(r.edge(200, true).unwrap().is_empty());
        assert_eq!(kinds(r.advance(500).unwrap()), vec![TriggerKind::LongPress]);
        r.edge(501, false).unwrap();
        assert!(r.edge(502, false).unwrap().is_empty());
    }
    #[test]
    fn invalid_time_is_atomic() {
        let mut r = resolver(false, false);
        r.edge(100, true).unwrap();
        assert_eq!(r.edge(99, false), Err(TriggerError::TimeReversed));
        assert_eq!(kinds(r.advance(600).unwrap()), vec![TriggerKind::LongPress]);
        assert!(
            PressResolver::new(
                TimingPolicy {
                    repeat_interval_ms: 0,
                    ..TimingPolicy::default()
                },
                Bindings::default()
            )
            .is_err()
        );
        let mut r = resolver(false, false);
        assert_eq!(r.edge(u64::MAX, true), Err(TriggerError::TimeOverflow));
        assert!(r.edge(0, true).is_ok());
    }
}
