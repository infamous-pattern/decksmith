//! One read-only observation owner for the initial native pair acceptance.
use crate::{audio::State, audio_target::Client};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
type Publication = (u64, Instant, Vec<Option<State>>);
struct Shared {
    latest: Mutex<Option<Publication>>,
    control: Mutex<Control>,
    wake: Condvar,
}
struct Control {
    stop: bool,
    epoch: u64,
    targets: Vec<String>,
}
pub struct Observer {
    targets: Vec<String>,
    epoch: u64,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    shown: Option<Option<Instant>>,
}
impl Observer {
    pub fn start_many(mut targets: Vec<String>) -> Result<Self> {
        targets.sort();
        targets.dedup();
        if targets.len() > 32 || targets.iter().any(|t| !crate::audio_target::valid(t)) {
            return Err("invalid_native_audio_target");
        }
        let shared = Arc::new(Shared {
            latest: Mutex::new(None),
            control: Mutex::new(Control {
                stop: false,
                epoch: 1,
                targets: targets.clone(),
            }),
            wake: Condvar::new(),
        });
        let state = shared.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-shared-observation".into())
            .spawn(move || {
                let mut reader = Client::default();
                loop {
                    let (epoch, wanted) = {
                        let mut control = state.control.lock().unwrap();
                        while !control.stop && control.targets.is_empty() {
                            control = state.wake.wait(control).unwrap();
                        }
                        if control.stop {
                            return;
                        }
                        (control.epoch, control.targets.clone())
                    };
                    let started = Instant::now();
                    let mut values: Vec<_> = wanted
                        .chunks(13)
                        .flat_map(|batch| reader.read(batch))
                        .collect();
                    if started.elapsed() >= Duration::from_millis(500) {
                        values.fill(None);
                    }
                    *state.latest.lock().unwrap() = Some((epoch, Instant::now(), values));
                    let control = state.control.lock().unwrap();
                    if control.stop {
                        return;
                    }
                    if control.epoch == epoch {
                        let _ = state.wake.wait_timeout(control, Duration::from_millis(200));
                    }
                }
            })
            .map_err(|_| "native_observer_spawn_failed")?;
        Ok(Self {
            targets,
            epoch: 1,
            shared,
            thread: Some(thread),
            shown: None,
        })
    }
    /// Pure bounded reconfiguration: no join, process spawn, disk or audio call
    /// on the coordinator. Late replies retain their previous epoch and expire.
    pub fn configure(&mut self, mut targets: Vec<String>) -> Result<()> {
        targets.sort();
        targets.dedup();
        if targets.len() > 32 || targets.iter().any(|t| !crate::audio_target::valid(t)) {
            return Err("invalid_native_audio_target");
        }
        if targets == self.targets {
            return Ok(());
        }
        let epoch = self
            .epoch
            .checked_add(1)
            .ok_or("native_observation_epoch_exhausted")?;
        let mut control = self.shared.control.lock().unwrap();
        if control.stop {
            return Err("native_observer_stopped");
        }
        control.targets = targets.clone();
        control.epoch = epoch;
        self.targets = targets;
        self.epoch = epoch;
        self.shown = None;
        self.shared.wake.notify_all();
        Ok(())
    }
    pub fn snapshot(&mut self) -> Option<Vec<(String, Option<State>)>> {
        let latest = self.shared.latest.lock().unwrap();
        let values = latest.as_ref().filter(|(epoch, at, _)| {
            *epoch == self.epoch && at.elapsed() < Duration::from_millis(500)
        });
        let stamp = values.map(|(_, at, _)| *at);
        if self.shown == Some(stamp) {
            return None;
        }
        self.shown = Some(stamp);
        Some(
            self.targets
                .iter()
                .enumerate()
                .map(|(i, target)| {
                    (
                        target.clone(),
                        values.and_then(|(_, _, states)| states.get(i).copied().flatten()),
                    )
                })
                .collect(),
        )
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        self.shared.control.lock().unwrap().stop = true;
        self.shared.wake.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_snapshots_return_unavailable_for_every_saved_target() {
        let mut observer = Observer {
            targets: vec!["system".into(), "output:fixture".into()],
            epoch: 1,
            shared: Arc::new(Shared {
                latest: Mutex::new(Some((
                    1,
                    Instant::now() - Duration::from_secs(1),
                    vec![
                        Some(State {
                            percent: 43,
                            muted: false,
                            icon: crate::audio_icon::Kind::Unknown,
                            active: None,
                        }),
                        None,
                    ],
                ))),
                control: Mutex::new(Control {
                    stop: false,
                    epoch: 1,
                    targets: vec!["system".into(), "output:fixture".into()],
                }),
                wake: Condvar::new(),
            }),
            thread: None,
            shown: None,
        };
        assert!(
            observer
                .snapshot()
                .unwrap()
                .iter()
                .all(|(_, s)| s.is_none())
        );
        assert!(observer.snapshot().is_none());
        *observer.shared.latest.lock().unwrap() = Some((1, Instant::now(), vec![None, None]));
        assert_eq!(observer.snapshot().unwrap().len(), 2);
        observer
            .configure(vec!["app:application.name=New".into()])
            .unwrap();
        assert_eq!(
            observer.snapshot().unwrap(),
            vec![("app:application.name=New".into(), None)]
        );
        *observer.shared.latest.lock().unwrap() = Some((
            1,
            Instant::now(),
            vec![Some(State {
                percent: 99,
                muted: false,
                active: None,
                icon: Default::default(),
            })],
        ));
        assert!(
            observer.snapshot().is_none(),
            "Late old epoch must never become the new target's value"
        );
    }
}
