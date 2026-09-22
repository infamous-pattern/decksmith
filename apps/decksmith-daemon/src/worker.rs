//! One owner per device session; no hardware calls on the output thread.
use decksmith_core::InputEvent;
use decksmith_device::{DeckDevice, DeviceError};
use serde::Serialize;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{SyncSender, TrySendError},
    },
    time::Duration,
};

#[derive(Debug, Serialize)]
#[serde(tag = "record", rename_all = "snake_case")]
pub enum Record {
    Waiting,
    DisplaySettled {
        session: u64,
        page: u8,
    },
    ActionQueued {
        session: u64,
        action: crate::pages::Action,
        input: InputEvent,
    },
    AudioWorkerFailed {
        error_code: &'static str,
    },
    AudioState {
        session: u64,
        state: Option<crate::audio::State>,
    },
    RenderFailed,
    ActionResult {
        page: u8,
        session: u64,
        action: crate::pages::Action,
        success: bool,
        error_code: Option<&'static str>,
        input: InputEvent,
    },
    Connected {
        session: u64,
    },
    Input {
        session: u64,
        input: InputEvent,
    },
    Disconnected {
        session: u64,
    },
    PageRequested {
        session: u64,
        page: u8,
        input: InputEvent,
    },
}

struct Session<D> {
    resume_pending: bool,
    reopen_after: Option<std::time::Instant>,
    foreground_revision: u64,
    lock_epoch: u64,
    locked: bool,
    drain_until: Option<std::time::Instant>,
    device: Option<D>,
    generation: u64,
    waiting_reported: bool,
    pages: Option<crate::pages::Pages>,
    audio: Option<crate::audio_worker::Worker>,
    meter: Option<crate::meter::Worker>,
    feedback: Option<crate::feedback::Worker>,
    ptt: Option<crate::ptt::Controller>,
    display: crate::display::Display,
    settlement_due: bool,
    controls: Option<crate::control::Context>,
}
impl<D: DeckDevice> Session<D> {
    fn resumed(&mut self) {
        if let Some(audio) = &self.audio {
            audio.session(0);
        }
        if let Some(ptt) = &self.ptt {
            ptt.cancel();
        }
        self.generation += 1;
        self.device = None;
        self.display = crate::display::Display::default();
        self.waiting_reported = false;
        self.resume_pending = true;
        // USB devices may still be restoring power when userspace resumes.
        self.reopen_after = Some(std::time::Instant::now() + Duration::from_secs(2));
        self.drain_until = Some(std::time::Instant::now() + Duration::from_secs(3));
        if let Some(context) = &self.controls {
            while let Ok((command, _, _)) = context.incoming.try_recv() {
                let answer = match command {
                    crate::control::Command::TestControl { answer, .. }
                    | crate::control::Command::Page { answer, .. }
                    | crate::control::Command::Brightness { answer, .. }
                    | crate::control::Command::Layout { answer, .. } => answer,
                };
                let _ = answer.try_send(Err("device_resuming"));
            }
        }
    }

    fn step(&mut self, open: &mut impl FnMut() -> Result<D, DeviceError>) -> Option<Record> {
        if self
            .reopen_after
            .is_some_and(|t| std::time::Instant::now() < t)
        {
            return None;
        }
        self.reopen_after = None;
        let lock = self.controls.as_ref().map(|c| c.gate.snapshot());
        if let Some(lock) = &lock
            && lock.epoch != self.lock_epoch
        {
            let changed = self.lock_epoch != 0 || lock.enabled;
            self.lock_epoch = lock.epoch;
            self.locked = lock.locked;
            self.generation += 1;
            if let Some(audio) = &self.audio {
                audio.session(if self.locked { 0 } else { self.generation });
            }
            if let Some(ptt) = &self.ptt {
                ptt.cancel();
            }
            if let Some(pages) = self.pages.as_mut() {
                let _ = pages.show(&mut self.display, pages.index);
            }
            if changed {
                self.drain_until = Some(std::time::Instant::now() + Duration::from_millis(250));
                self.settlement_due = true;
            }
        }
        if !self.locked
            && self.device.is_some()
            && let Some(context) = &self.controls
        {
            let focus = context.foreground.snapshot();
            if focus.revision != self.foreground_revision {
                self.foreground_revision = focus.revision;
                if focus.available
                    && let Some(id) = focus.application.as_deref()
                    && let Some(pages) = self.pages.as_mut()
                    && let Some(target) = pages.application_page(id)
                    && target != pages.index
                {
                    self.generation += 1;
                    if let Some(audio) = &self.audio {
                        audio.session(self.generation);
                    }
                    if let Some(ptt) = &self.ptt {
                        ptt.cancel();
                    }
                    if pages.show(&mut self.display, target).is_ok() {
                        self.settlement_due = true;
                        self.drain_until =
                            Some(std::time::Instant::now() + Duration::from_millis(250));
                    }
                }
            }
        }
        if let Some(worker) = &self.feedback
            && let Some(pages) = self.pages.as_mut()
        {
            if let Some(notices) = worker.poll(
                if self.device.is_some() {
                    self.generation
                } else {
                    0
                },
                pages.health_checks(),
            ) {
                let _ = pages.update_health(&mut self.display, notices);
            }
            let _ = pages.expire_feedback(&mut self.display);
        }
        if let Some(meter) = &self.meter {
            let targets = self
                .pages
                .as_ref()
                .map(|p| p.meter_targets())
                .unwrap_or_default();
            let levels = meter.levels(
                if self.device.is_some() {
                    self.generation
                } else {
                    0
                },
                targets,
            );
            if let Some(pages) = self.pages.as_mut() {
                let _ = pages.apply_levels(&mut self.display, levels);
            }
        }
        if let Some(ptt) = &self.ptt {
            ptt.tick();
            if let Some(pages) = self.pages.as_mut() {
                for target in ptt.take_failures() {
                    for check in pages
                        .health_checks()
                        .into_iter()
                        .filter(|c| c.target == target)
                    {
                        if pages.ptt_target(check.slot).as_deref() != Some(target.as_str()) {
                            continue;
                        }
                        let input = InputEvent {
                            timestamp_ms: 0,
                            event: if check.slot < 8 {
                                decksmith_core::RawEvent::Key {
                                    index: check.slot,
                                    pressed: true,
                                }
                            } else {
                                decksmith_core::RawEvent::DialPush {
                                    index: check.slot - 8,
                                    pressed: true,
                                }
                            },
                        };
                        let action = crate::pages::Action::PushToTalk {
                            target: target.clone(),
                        };
                        let _ = pages.action_feedback(
                            &mut self.display,
                            pages.index,
                            &action,
                            &input,
                            Some("ptt_guard_failed"),
                        );
                    }
                }
            }
            if self.device.is_none() {
                ptt.cancel();
            }
        }
        if let Some(context) = &self.controls
            && let Ok((command, epoch, focus_revision)) = context.incoming.try_recv()
        {
            if self.locked
                || !context.gate.accepts(epoch)
                || (matches!(&command, crate::control::Command::Page { .. })
                    && focus_revision != context.foreground.snapshot().revision)
            {
                let answer = match command {
                    crate::control::Command::TestControl { answer, .. }
                    | crate::control::Command::Page { answer, .. }
                    | crate::control::Command::Brightness { answer, .. }
                    | crate::control::Command::Layout { answer, .. } => answer,
                };
                let _ = answer.try_send(Err("session_locked"));
            } else {
                match command {
                    crate::control::Command::TestControl {
                        layout_json,
                        page,
                        slot,
                        answer,
                    } => {
                        let result = if self.device.is_none() {
                            Err("device_disconnected")
                        } else if self.display.pending() || self.drain_until.is_some() {
                            Err("display_updating")
                        } else if let Some(pages) = self.pages.as_mut() {
                            if pages.index != page || pages.json() != layout_json {
                                Err("save_and_select_page_first")
                            } else if pages.ptt_target(slot).is_some() {
                                Err("test_push_to_talk_on_device")
                            } else {
                                let event = if slot < 8 {
                                    decksmith_core::RawEvent::Key {
                                        index: slot,
                                        pressed: false,
                                    }
                                } else {
                                    decksmith_core::RawEvent::DialPush {
                                        index: slot - 8,
                                        pressed: false,
                                    }
                                };
                                let input = InputEvent {
                                    timestamp_ms: 0,
                                    event,
                                };
                                match pages.test_target(slot) {
                                    Some(crate::pages::Action::GoToPage { page }) => {
                                        let result = pages
                                            .show(&mut self.display, page)
                                            .map_err(|_| "invalid_page");
                                        if result.is_ok() {
                                            self.settlement_due = true;
                                        }
                                        result
                                    }
                                    Some(action) => {
                                        self.audio.as_ref().ok_or("audio_worker_stopped").and_then(
                                            |audio| {
                                                audio.submit(crate::audio_worker::Request::Action {
                                                    page: pages.index,
                                                    session: self.generation,
                                                    action,
                                                    input,
                                                })
                                            },
                                        )
                                    }
                                    None => Err("no_action_assigned"),
                                }
                            }
                        } else {
                            Err("layout_unavailable")
                        };
                        let _ = answer.try_send(result);
                    }
                    crate::control::Command::Page { page, answer } => {
                        if let Some(ptt) = &self.ptt {
                            ptt.cancel();
                        }
                        let result = if self.device.is_none() {
                            Err("device_disconnected")
                        } else {
                            self.pages
                                .as_mut()
                                .ok_or("layout_unavailable")
                                .and_then(|pages| {
                                    pages
                                        .show(&mut self.display, page)
                                        .map_err(|_| "invalid_page")
                                })
                        };
                        if result.is_ok() {
                            self.settlement_due = true;
                        }
                        let _ = answer.try_send(result);
                    }

                    crate::control::Command::Brightness { percent, answer } => {
                        let result =
                            self.device
                                .as_mut()
                                .ok_or("device_disconnected")
                                .and_then(|device| {
                                    device
                                        .set_brightness(percent)
                                        .map_err(|_| "brightness_failed")
                                });
                        if result.is_ok() {
                            context.status.lock().unwrap().brightness = Some(percent);
                        } else if result == Err("brightness_failed") {
                            if let Some(audio) = &self.audio {
                                audio.session(0);
                            }
                            if let Some(ptt) = &self.ptt {
                                ptt.cancel();
                            }
                            self.device = None;
                            self.waiting_reported = false;
                        }
                        let _ = answer.try_send(result);
                    }
                    crate::control::Command::Layout {
                        mut pages,
                        name,
                        answer,
                    } => {
                        if let Some(ptt) = &self.ptt {
                            ptt.cancel();
                        }
                        let result = pages
                            .show(&mut self.display, pages.default_page())
                            .map_err(|_| "layout_failed");
                        if result.is_ok() {
                            let layout_json = pages.json();
                            self.pages = Some(*pages);
                            self.settlement_due = true;
                            let mut status = context.status.lock().unwrap();
                            status.layout = name;
                            status.layout_json = layout_json;
                        }
                        let _ = answer.try_send(result);
                    }
                }
            }
        }
        if let Some(context) = &self.controls
            && let Some(pages) = self.pages.as_mut()
        {
            let value = context.status.lock().unwrap().brightness;
            if matches!(pages.update_brightness(&mut self.display, value), Ok(true)) {
                self.settlement_due = true;
            }
        }
        if self.locked {
            let _ = crate::session_lock::paint(&mut self.display);
        }
        if let Some(device) = self.device.as_mut() {
            match self.display.flush_one(device) {
                Ok(_) => (),
                Err(_) => {
                    if let Some(audio) = &self.audio {
                        audio.session(0);
                    }
                    if let Some(ptt) = &self.ptt {
                        ptt.cancel();
                    }
                    self.device = None;
                    self.waiting_reported = false;
                    return Some(Record::Disconnected {
                        session: self.generation,
                    });
                }
            }
            if self.settlement_due && !self.display.pending() {
                self.settlement_due = false;
                if let Some(pages) = &self.pages {
                    return Some(Record::DisplaySettled {
                        session: self.generation,
                        page: pages.index,
                    });
                }
            }
            if let Some(audio) = &self.audio {
                match audio.poll() {
                    Ok(Some(crate::audio_worker::Reply::Action {
                        page,
                        session,
                        action,
                        input,
                        result,
                    })) if session == self.generation => {
                        if let Some(pages) = self.pages.as_mut() {
                            pages.invalidate_audio();
                        }
                        return Some(Record::ActionResult {
                            page,
                            session,
                            action,
                            input,
                            success: result.is_ok(),
                            error_code: result.err(),
                        });
                    }
                    Ok(Some(crate::audio_worker::Reply::State {
                        system,
                        session,
                        state,
                        targets,
                    })) if session == self.generation => {
                        if let Some(pages) = self.pages.as_mut() {
                            if targets.iter().map(|(t, _)| t.clone()).collect::<Vec<_>>()
                                != pages.audio_targets()
                            {
                                return None;
                            }
                            if pages.system_commands() != system.keys().cloned().collect::<Vec<_>>()
                            {
                                return None;
                            }
                            if pages.apply_system(&mut self.display, system).is_err() {
                                self.device = None;
                                return Some(Record::Disconnected { session });
                            }
                            pages.apply_targets(targets);
                            match pages.apply_audio(&mut self.display, state) {
                                Ok(true) => {
                                    self.settlement_due = true;
                                    return Some(Record::AudioState { session, state });
                                }
                                Ok(false) => (),
                                Err(_) => {
                                    audio.session(0);
                                    if let Some(ptt) = &self.ptt {
                                        ptt.cancel();
                                    }
                                    self.device = None;
                                    self.waiting_reported = false;
                                    return Some(Record::Disconnected { session });
                                }
                            }
                        }
                    }
                    Err(error_code) => {
                        self.audio = None;
                        return Some(Record::AudioWorkerFailed { error_code });
                    }
                    _ => (),
                }
                if let Some(pages) = self.pages.as_mut()
                    && pages.audio_due()
                    && let Some(audio) = &self.audio
                {
                    let _ = audio.submit(crate::audio_worker::Request::Read {
                        system: pages.system_commands(),
                        session: self.generation,
                        targets: pages.audio_targets(),
                    });
                }
            }
            return match device.poll_event() {
                Ok(Some(input)) => {
                    if self.locked || self.drain_until.is_some() {
                        self.drain_until =
                            Some(std::time::Instant::now() + Duration::from_millis(250));
                        return None;
                    }
                    let edge = match &input.event {
                        decksmith_core::RawEvent::Key { index, pressed } if *index < 8 => {
                            Some((*index, *pressed))
                        }
                        decksmith_core::RawEvent::DialPush { index, pressed } if *index < 4 => {
                            Some((*index + 8, *pressed))
                        }
                        _ => None,
                    };
                    if let Some((slot, pressed)) = edge
                        && let Some(ptt) = &self.ptt
                    {
                        if !pressed {
                            ptt.release(slot);
                        }
                        if let Some(target) =
                            self.pages.as_ref().and_then(|pages| pages.ptt_target(slot))
                        {
                            if pressed && !self.display.pending() {
                                ptt.press(slot, target);
                            }
                            return Some(Record::Input {
                                session: self.generation,
                                input,
                            });
                        }
                    }
                    let switching = self.display.pending()
                        && matches!(
                            input.event,
                            decksmith_core::RawEvent::Key { .. }
                                | decksmith_core::RawEvent::DialPush { .. }
                                | decksmith_core::RawEvent::Touch {
                                    gesture: decksmith_core::TouchGesture::Tap,
                                    ..
                                }
                        );
                    if !switching
                        && let Some(pages) = self.pages.as_mut()
                        && let Some(target) = pages.target(&input.event)
                    {
                        if let crate::pages::Action::BrightnessAdjust { percent } = target {
                            let result = if let Some(context) = &self.controls {
                                let current = context.status.lock().unwrap().brightness;
                                if let Some(current) = current {
                                    let value = (i16::from(current) + percent).clamp(0, 100) as u8;
                                    device
                                        .set_brightness(value)
                                        .map_err(|_| "brightness_failed")
                                        .map(|()| {
                                            context.status.lock().unwrap().brightness = Some(value);
                                            crate::control::persist_later(context.status.clone());
                                        })
                                } else {
                                    Err("brightness_not_set")
                                }
                            } else {
                                Err("control_interface_required")
                            };
                            if result == Err("brightness_failed") {
                                if let Some(ptt) = &self.ptt {
                                    ptt.cancel();
                                }
                                self.device = None;
                                self.waiting_reported = false;
                                if let Some(audio) = &self.audio {
                                    audio.session(0);
                                }
                            }
                            return Some(Record::ActionResult {
                                page: pages.index,
                                session: self.generation,
                                action: target,
                                input,
                                success: result.is_ok(),
                                error_code: result.err(),
                            });
                        }
                        let crate::pages::Action::GoToPage { page: target } = target else {
                            let result =
                                self.audio.as_ref().ok_or("audio_worker_stopped").and_then(
                                    |audio| {
                                        audio.submit(crate::audio_worker::Request::Action {
                                            page: pages.index,
                                            session: self.generation,
                                            action: target.clone(),
                                            input: input.clone(),
                                        })
                                    },
                                );
                            if result.is_ok() {
                                return Some(Record::ActionQueued {
                                    session: self.generation,
                                    action: target,
                                    input,
                                });
                            }
                            return Some(Record::ActionResult {
                                page: pages.index,
                                session: self.generation,
                                action: target,
                                success: result.is_ok(),
                                error_code: result.err(),
                                input,
                            });
                        };
                        if let Some(ptt) = &self.ptt {
                            ptt.cancel();
                        }
                        if pages.show(&mut self.display, target).is_err() {
                            if let Some(audio) = &self.audio {
                                audio.session(0);
                            }
                            if let Some(ptt) = &self.ptt {
                                ptt.cancel();
                            }
                            self.device = None;
                            self.waiting_reported = false;
                            return Some(Record::Disconnected {
                                session: self.generation,
                            });
                        }
                        self.settlement_due = true;
                        return Some(Record::PageRequested {
                            session: self.generation,
                            page: target,
                            input,
                        });
                    }
                    Some(Record::Input {
                        session: self.generation,
                        input,
                    })
                }
                Ok(None) => {
                    if !self.locked
                        && !self.display.pending()
                        && self
                            .drain_until
                            .is_some_and(|t| std::time::Instant::now() >= t)
                    {
                        self.drain_until = None;
                    }
                    None
                }
                Err(_) => {
                    // Drop the handle, queued inputs and state before any reconnect.
                    if let Some(audio) = &self.audio {
                        audio.session(0);
                    }
                    if let Some(ptt) = &self.ptt {
                        ptt.cancel();
                    }
                    self.device = None;
                    self.waiting_reported = false;
                    Some(Record::Disconnected {
                        session: self.generation,
                    })
                }
            };
        }
        match open() {
            Ok(mut device) => {
                self.foreground_revision = 0;
                self.display = crate::display::Display::default();
                if let Some(pages) = self.pages.as_mut()
                    && pages
                        .show(
                            &mut self.display,
                            if self.locked || self.resume_pending {
                                pages.index
                            } else {
                                pages.default_page()
                            },
                        )
                        .is_err()
                {
                    if self.waiting_reported {
                        return None;
                    }
                    self.waiting_reported = true;
                    return Some(Record::RenderFailed);
                }
                if let Some(context) = &self.controls
                    && let Some(percent) = context.status.lock().unwrap().brightness
                    && device.set_brightness(percent).is_err()
                {
                    return Some(Record::RenderFailed);
                }
                self.settlement_due = self.pages.is_some();
                self.generation += 1;
                if let Some(audio) = &self.audio {
                    audio.session(if self.locked { 0 } else { self.generation });
                }
                self.device = Some(device);
                self.resume_pending = false;
                self.waiting_reported = false;
                Some(Record::Connected {
                    session: self.generation,
                })
            }
            Err(_) if !self.waiting_reported => {
                self.waiting_reported = true;
                Some(Record::Waiting)
            }
            Err(_) => None,
        }
    }
}

pub fn run<D: DeckDevice>(
    mut open: impl FnMut() -> Result<D, DeviceError>,
    sender: SyncSender<Record>,
    stop: Arc<AtomicBool>,
    pages: Option<crate::pages::Pages>,
    controls: Option<crate::control::Context>,
) -> Result<(), &'static str> {
    let gate = controls
        .as_ref()
        .map(|c| c.gate.clone())
        .unwrap_or_default();
    let foreground = controls
        .as_ref()
        .map(|c| c.foreground.clone())
        .unwrap_or_default();
    let mut state = Session {
        resume_pending: false,
        reopen_after: None,
        foreground_revision: 0,
        lock_epoch: 0,
        locked: false,
        drain_until: None,
        device: None,
        generation: 0,
        waiting_reported: false,
        audio: pages.as_ref().map(|_| {
            crate::audio_worker::Worker::start_with_context(gate.clone(), foreground.clone())
        }),
        meter: pages.as_ref().map(|_| crate::meter::Worker::start()),
        feedback: pages.as_ref().map(|_| crate::feedback::Worker::start()),
        ptt: Some(crate::ptt::Controller::start()),
        pages,
        display: crate::display::Display::default(),
        settlement_due: false,
        controls,
    };
    let mut resume = crate::resume::Monitor::default();
    // Establish the clock baseline before the first possible suspend.
    resume.poll();
    let mut outcome = Ok(());
    while !stop.load(Ordering::Relaxed) {
        if resume.poll() {
            state.resumed();
            tracing::info!(
                operation = "resume",
                "Reopening device and restoring display after suspend"
            );
        }
        let record = state.step(&mut open);
        if let Some(Record::ActionResult {
            page,
            action,
            input,
            error_code,
            ..
        }) = &record
            && let Some(pages) = state.pages.as_mut()
        {
            let _ = pages.action_feedback(&mut state.display, *page, action, input, *error_code);
        }

        if let Some(context) = &state.controls {
            let mut status = context.status.lock().unwrap();
            let sent = if state.device.is_some() {
                state.display.sent_touch()
            } else {
                None
            };
            if status.touch_frame.as_ref() != sent {
                status.touch_frame = sent.cloned();
            }
            status.touch_state = state
                .pages
                .as_ref()
                .map(|pages| pages.touch_state())
                .unwrap_or_default();
            status.attention = if state.device.is_some() {
                state
                    .pages
                    .as_ref()
                    .map(|p| p.attention())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            status.connected = state.device.is_some();
            status.display_ready = state.device.is_some() && !state.display.pending();
            status.active_page = if status.display_ready {
                state.pages.as_ref().map(|pages| pages.index)
            } else {
                None
            };
        }
        if let Some(record) = record {
            match sender.try_send(record) {
                Ok(()) => (),
                // Never silently lose a release/lifecycle event. End this worker;
                // the caller must invalidate the entire stream on this error.
                Err(TrySendError::Full(_)) => {
                    outcome = Err("event_queue_full");
                    break;
                }
                Err(TrySendError::Disconnected(_)) => {
                    outcome = Err("event_consumer_closed");
                    break;
                }
            }
        }
        if state.device.is_none() {
            // One-second retry, interruptible in short increments during shutdown.
            for _ in 0..20 {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    // Stop accepting control work before clearing every display surface.
    if let Some(audio) = &state.audio {
        audio.session(0);
    }
    if let Some(ptt) = &state.ptt {
        ptt.cancel();
    }
    if let Some(device) = state.device.as_mut() {
        if device.blank().is_err() {
            tracing::warn!("could not clear every device display on shutdown");
        } else {
            tracing::info!("cleared keys and touch strip on shutdown");
        }
    }
    if let Some(context) = &state.controls
        && crate::control::save_shared(&context.status).is_err()
    {
        tracing::warn!("could not save settings on shutdown");
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graceful_stop_blanks_the_open_device() {
        struct Probe {
            stop: Arc<AtomicBool>,
            cleared: Arc<AtomicBool>,
        }
        impl DeckDevice for Probe {
            fn geometry(&self) -> decksmith_core::Geometry {
                decksmith_core::Geometry::plus()
            }
            fn set_key_image(&mut self, _: u8, _: &[u8]) -> Result<(), DeviceError> {
                Ok(())
            }
            fn set_touch_image(&mut self, _: &[u8]) -> Result<(), DeviceError> {
                Ok(())
            }
            fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
                Ok(())
            }
            fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
                self.stop.store(true, Ordering::Relaxed);
                Ok(None)
            }
            fn blank(&mut self) -> Result<(), DeviceError> {
                self.cleared.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
        let stop = Arc::new(AtomicBool::new(false));
        let cleared = Arc::new(AtomicBool::new(false));
        let (tx, _rx) = std::sync::mpsc::sync_channel(16);
        run(
            || {
                Ok(Probe {
                    stop: stop.clone(),
                    cleared: cleared.clone(),
                })
            },
            tx,
            stop.clone(),
            None,
            None,
        )
        .unwrap();
        assert!(cleared.load(Ordering::Relaxed));
    }

    use decksmith_device::VirtualDeck;
    #[test]
    fn editor_test_requires_matching_saved_layout_connection_and_unlocked_epoch() {
        use crate::control::{Command, Context, Status};
        let gate = crate::session_lock::Gate::default();
        let (tx, rx) = std::sync::mpsc::sync_channel(4);
        let pages =
            crate::pages::Pages::parse(include_bytes!("../../../config/navigation.json")).unwrap();
        let json = pages.json();
        let status = Arc::new(std::sync::Mutex::new(Status {
            api_version: 1,
            connected: true,
            layout: "navigation".into(),
            brightness: None,
            display_ready: true,
            active_page: Some(0),
            layout_json: json.clone(),
            touch_frame: None,
            touch_state: Default::default(),
            attention: vec![],
        }));
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: gate.snapshot().epoch,
            locked: false,
            drain_until: None,
            device: Some(VirtualDeck::default()),
            generation: 1,
            waiting_reported: false,
            pages: Some(pages),
            audio: None,
            meter: None,
            feedback: None,
            ptt: None,
            display: Default::default(),
            settlement_due: false,
            controls: Some(Context {
                foreground: Default::default(),
                gate: gate.clone(),
                incoming: rx,
                status,
            }),
        };
        let mut open = || Err(DeviceError::Disconnected);
        let send = |raw: String, page, epoch| {
            let (answer, reply) = std::sync::mpsc::sync_channel(1);
            tx.send((
                Command::TestControl {
                    layout_json: raw,
                    page,
                    slot: 1,
                    answer,
                },
                epoch,
                0,
            ))
            .unwrap();
            reply
        };
        let reply = send("{}".into(), 0, gate.snapshot().epoch);
        s.step(&mut open);
        assert_eq!(reply.try_recv().unwrap(), Err("save_and_select_page_first"));
        let reply = send(json.clone(), 0, gate.snapshot().epoch);
        s.step(&mut open);
        assert_eq!(reply.try_recv().unwrap(), Ok(()));
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        let reply = send(json.clone(), 0, gate.snapshot().epoch);
        s.step(&mut open);
        assert!(reply.try_recv().unwrap().is_err());
        s.device = None;
        let reply = send(json.clone(), 1, gate.snapshot().epoch);
        s.step(&mut open);
        assert_eq!(reply.try_recv().unwrap(), Err("device_disconnected"));
        let old_epoch = gate.snapshot().epoch;
        gate.enable(true);
        gate.observe(Some(true), "test");
        let reply = send(json, 1, old_epoch);
        s.step(&mut open);
        assert_eq!(reply.try_recv().unwrap(), Err("session_locked"));
    }
    #[test]
    fn application_changes_switch_immediately_without_sticky_manual_override() {
        use crate::control::{Command, Context, Status};
        let focus = crate::foreground::Foreground::default();
        let gate = crate::session_lock::Gate::default();
        let (tx, rx) = std::sync::mpsc::sync_channel(4);
        let status = Arc::new(std::sync::Mutex::new(Status {
            api_version: 1,
            connected: true,
            layout: "navigation".into(),
            brightness: Some(50),
            display_ready: true,
            active_page: Some(0),
            layout_json: "{}".into(),
            touch_frame: None,
            touch_state: Default::default(),
            attention: vec![],
        }));
        let mut raw: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        raw["pages"][0]["application"] = "brave-browser.desktop".into();
        raw["pages"][1]["application"] = "teams.desktop".into();
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: gate.snapshot().epoch,
            locked: false,
            drain_until: None,
            device: Some(VirtualDeck::default()),
            generation: 1,
            waiting_reported: false,
            pages: Some(crate::pages::Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap()),
            audio: None,
            meter: None,
            feedback: None,
            ptt: Some(crate::ptt::Controller::fixture()),
            display: Default::default(),
            settlement_due: false,
            controls: Some(Context {
                incoming: rx,
                status,
                gate: gate.clone(),
                foreground: focus.clone(),
            }),
        };
        let mut open = || Ok(VirtualDeck::default());
        s.ptt.as_ref().unwrap().press(0, "input:test".into());
        focus.report("teams.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
        let (answer, reply) = std::sync::mpsc::sync_channel(1);
        tx.send((
            Command::Page { page: 0, answer },
            gate.snapshot().epoch,
            focus.snapshot().revision,
        ))
        .unwrap();
        s.step(&mut open);
        assert_eq!(reply.try_recv().unwrap(), Ok(()));
        focus.report("teams.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 0);
        focus.report("brave-browser.desktop").unwrap();
        s.step(&mut open);
        focus.report("teams.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        focus.report("unmatched.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 0);
        focus.report("teams.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        focus.report(crate::foreground::STUDIO).unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        gate.enable(true);
        gate.observe(Some(true), "test");
        focus.report("brave-browser.desktop").unwrap();
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        gate.observe(Some(false), "test");
        s.step(&mut open);
        assert_eq!(s.pages.as_ref().unwrap().index, 0);
        let (answer, reply) = std::sync::mpsc::sync_channel(1);
        tx.send((
            Command::Page { page: 0, answer },
            gate.snapshot().epoch,
            focus.snapshot().revision,
        ))
        .unwrap();
        focus.report("teams.desktop").unwrap();
        s.step(&mut open);
        assert!(reply.try_recv().unwrap().is_err());
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
    }
    #[test]
    fn lock_blocks_inputs_cancels_holds_preserves_page_and_restores_after_drain() {
        use crate::control::{Context, Status};
        use decksmith_core::RawEvent;
        let gate = crate::session_lock::Gate::new(true);
        gate.observe(Some(false), "test");
        let (_tx, rx) = std::sync::mpsc::sync_channel(4);
        let status = Arc::new(std::sync::Mutex::new(Status {
            api_version: 1,
            connected: true,
            layout: "audio".into(),
            brightness: Some(50),
            display_ready: true,
            active_page: Some(1),
            layout_json: "{}".into(),
            touch_frame: None,
            touch_state: Default::default(),
            attention: vec![],
        }));
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: gate.snapshot().epoch,
            locked: false,
            drain_until: None,
            device: Some(VirtualDeck::default()),
            generation: 1,
            waiting_reported: false,
            pages: Some(
                crate::pages::Pages::parse(include_bytes!("../../../config/audio.json")).unwrap(),
            ),
            audio: None,
            meter: None,
            feedback: None,
            ptt: Some(crate::ptt::Controller::fixture()),
            display: Default::default(),
            settlement_due: false,
            controls: Some(Context {
                foreground: Default::default(),
                incoming: rx,
                status,
                gate: gate.clone(),
            }),
        };
        s.pages.as_mut().unwrap().show(&mut s.display, 1).unwrap();
        for _ in 0..12 {
            s.step(&mut || Ok(VirtualDeck::default()));
        }
        let original = s.device.as_ref().unwrap().touch_image().to_vec();
        s.ptt.as_ref().unwrap().press(0, "input:test".into());
        s.pages.as_mut().unwrap().target(&RawEvent::Key {
            index: 0,
            pressed: true,
        });
        gate.observe(Some(true), "test");
        for event in [
            RawEvent::Key {
                index: 0,
                pressed: false,
            },
            RawEvent::DialRotate { index: 0, ticks: 2 },
            RawEvent::DialPush {
                index: 0,
                pressed: true,
            },
            RawEvent::Touch {
                gesture: decksmith_core::TouchGesture::FlickLeft,
                x: 200,
                y: 50,
            },
        ] {
            s.device
                .as_mut()
                .unwrap()
                .inject(InputEvent {
                    timestamp_ms: 1,
                    event,
                })
                .unwrap();
        }
        for _ in 0..16 {
            assert!(!matches!(
                s.step(&mut || Ok(VirtualDeck::default())),
                Some(
                    Record::ActionQueued { .. }
                        | Record::ActionResult { .. }
                        | Record::PageRequested { .. }
                )
            ));
        }
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
        assert_ne!(s.device.as_ref().unwrap().touch_image(), original);
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        // Reconnect while locked must render only the locked screen.
        s.device = None;
        s.step(&mut || Ok(VirtualDeck::default()));
        for _ in 0..12 {
            s.step(&mut || Ok(VirtualDeck::default()));
        }
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        gate.observe(None, "test");
        s.step(&mut || Ok(VirtualDeck::default()));
        assert!(s.locked);
        gate.observe(Some(false), "test");
        for _ in 0..12 {
            s.step(&mut || Ok(VirtualDeck::default()));
        }
        assert_eq!(s.pages.as_ref().unwrap().index, 1);
        assert_eq!(s.device.as_ref().unwrap().touch_image(), original);
        assert!(
            s.pages
                .as_mut()
                .unwrap()
                .target(&RawEvent::Key {
                    index: 0,
                    pressed: false
                })
                .is_none()
        );
        s.drain_until = Some(std::time::Instant::now());
        s.step(&mut || Ok(VirtualDeck::default()));
        assert!(s.drain_until.is_none());
    }
    #[test]
    fn page_override_switch_cancels_held_dial_before_new_target() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        for (page, target) in [(0, "input:first"), (1, "input:second")] {
            value["pages"][page]["dial_overrides"] = serde_json::json!([
                {"label":"Mic","rotation":"volume","step":1,"audio_target":target,"press":{"type":"push_to_talk","target":target}},null,null,null]);
        }
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: Some(VirtualDeck::default()),
            generation: 1,
            waiting_reported: false,
            pages: Some(crate::pages::Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap()),
            audio: None,
            meter: None,
            ptt: Some(crate::ptt::Controller::fixture()),
            feedback: None,
            display: Default::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Ok(VirtualDeck::default());
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 1,
                event: decksmith_core::RawEvent::DialPush {
                    index: 0,
                    pressed: true,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 1);
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 2,
                event: decksmith_core::RawEvent::Touch {
                    x: 50,
                    y: 50,
                    gesture: decksmith_core::TouchGesture::FlickLeft,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
        assert_eq!(
            s.pages.as_ref().unwrap().ptt_target(8),
            Some("input:second".into())
        );
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 3,
                event: decksmith_core::RawEvent::DialPush {
                    index: 0,
                    pressed: false,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
    }
    #[test]
    fn ptt_release_bypasses_redraw_and_disconnect_cancels_holds() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0]["action"] =
            serde_json::json!({"type":"push_to_talk","target":"input:mic"});
        let pages = crate::pages::Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: Some(VirtualDeck::default()),
            generation: 1,
            waiting_reported: false,
            pages: Some(pages),
            audio: None,
            meter: None,
            ptt: Some(crate::ptt::Controller::fixture()),
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Ok(VirtualDeck::default());
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 1,
                event: decksmith_core::RawEvent::Key {
                    index: 0,
                    pressed: true,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 1);
        s.display.set_key_image(7, &[0; 120 * 120 * 3]).unwrap();
        s.display.set_key_image(6, &[0; 120 * 120 * 3]).unwrap();
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 2,
                event: decksmith_core::RawEvent::Key {
                    index: 0,
                    pressed: false,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
        s.ptt.as_ref().unwrap().press(0, "input:mic".into());
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 3,
                event: decksmith_core::RawEvent::Touch {
                    x: 100,
                    y: 50,
                    gesture: decksmith_core::TouchGesture::FlickLeft,
                },
            })
            .unwrap();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
        s.ptt.as_ref().unwrap().press(0, "input:mic".into());
        s.device.as_mut().unwrap().disconnect();
        s.step(&mut open);
        assert_eq!(s.ptt.as_ref().unwrap().held_count(), 0);
    }
    #[test]
    fn reconnect_uses_new_session_and_discards_stale_input() {
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: None,
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Ok(VirtualDeck::default());
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Connected { session: 1 })
        ));
        s.device.as_mut().unwrap().disconnect();
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Disconnected { session: 1 })
        ));
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Connected { session: 2 })
        ));
        assert!(s.step(&mut open).is_none());
    }
    #[test]
    fn resume_reopens_healthy_handle_repaints_page_and_respects_lock() {
        for locked in [false, true] {
            let mut s = Session {
                resume_pending: false,
                reopen_after: None,
                foreground_revision: 0,
                lock_epoch: 0,
                locked,
                drain_until: None,
                device: None,
                generation: 0,
                waiting_reported: false,
                pages: Some(crate::pages::Pages::default()),
                audio: None,
                meter: None,
                ptt: None,
                feedback: None,
                display: Default::default(),
                settlement_due: false,
                controls: None,
            };
            let gate = crate::session_lock::Gate::new(locked);
            gate.observe(Some(locked), "test");
            let (commands, incoming) = std::sync::mpsc::sync_channel(4);
            s.controls = Some(crate::control::Context {
                gate,
                incoming,
                foreground: Default::default(),
                status: Arc::new(std::sync::Mutex::new(crate::control::Status {
                    api_version: 1,
                    connected: true,
                    layout: "navigation".into(),
                    brightness: Some(55),
                    display_ready: true,
                    active_page: Some(1),
                    layout_json: s.pages.as_ref().unwrap().json(),
                    touch_frame: None,
                    touch_state: Default::default(),
                    attention: vec![],
                })),
            });
            let mut open = || Ok(VirtualDeck::default());
            s.step(&mut open);
            s.pages.as_mut().unwrap().show(&mut s.display, 1).unwrap();
            for _ in 0..12 {
                s.step(&mut open);
            }
            let old_generation = s.generation;
            assert!(!s.display.pending());
            assert!(s.device.as_ref().unwrap().is_connected());
            let (answer, reply) = std::sync::mpsc::sync_channel(1);
            commands
                .try_send((
                    crate::control::Command::Brightness { percent: 0, answer },
                    s.controls.as_ref().unwrap().gate.snapshot().epoch,
                    0,
                ))
                .unwrap();
            s.resumed();
            assert_eq!(reply.try_recv().unwrap(), Err("device_resuming"));
            assert!(s.device.is_none());
            assert!(s.generation > old_generation);
            assert!(
                s.step(&mut || panic!("must wait for USB recovery"))
                    .is_none()
            );
            s.reopen_after = None; // Advance the recovery delay without sleeping.
            assert!(matches!(s.step(&mut open), Some(Record::Connected { .. })));
            assert_eq!(s.pages.as_ref().unwrap().index, 1);
            assert!(s.display.pending());
            assert!(s.drain_until.is_some());
            for _ in 0..12 {
                s.step(&mut open);
            }
            assert!(!s.display.pending());
            let mut expected = VirtualDeck::default();
            if locked {
                crate::session_lock::paint(&mut expected).unwrap();
            } else {
                s.pages.as_mut().unwrap().show(&mut expected, 1).unwrap();
            }
            let actual = s.device.as_ref().unwrap();
            assert_eq!(actual.brightness, 55);
            for key in 0..8 {
                assert_eq!(actual.key_image(key), expected.key_image(key));
            }
            assert_eq!(actual.touch_image(), expected.touch_image());
        }
    }
    #[test]
    fn unavailable_device_does_not_flood_status() {
        let mut s: Session<VirtualDeck> = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: None,
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Err(DeviceError::DeviceSelection);
        assert!(matches!(s.step(&mut open), Some(Record::Waiting)));
        assert!(s.step(&mut open).is_none());
    }
    #[test]
    fn live_navigation_routes_swipe_and_repaints_after_reconnect() {
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: Some(crate::pages::Pages::default()),
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Ok(VirtualDeck::default());
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Connected { session: 1 })
        ));
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 1,
                event: decksmith_core::RawEvent::Touch {
                    x: 400,
                    y: 50,
                    gesture: decksmith_core::TouchGesture::FlickLeft,
                },
            })
            .unwrap();
        assert!(matches!(
            s.step(&mut open),
            Some(Record::PageRequested {
                session: 1,
                page: 1,
                ..
            })
        ));
        s.device.as_mut().unwrap().disconnect();
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Disconnected { session: 1 })
        ));
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Connected { session: 2 })
        ));
        assert_eq!(s.pages.as_ref().unwrap().index, 0);
    }
    #[test]
    fn deferred_render_failure_invalidates_connected_session() {
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: Some(crate::pages::Pages::default()),
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || {
            let mut deck = VirtualDeck::default();
            deck.disconnect();
            Ok(deck)
        };
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Connected { session: 1 })
        ));
        assert!(matches!(
            s.step(&mut open),
            Some(Record::Disconnected { session: 1 })
        ));
        assert!(s.device.is_none());
    }
    #[test]
    fn polls_input_during_redraw_but_does_not_dispatch_navigation_keys() {
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: Some(crate::pages::Pages::default()),
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: None,
        };
        let mut open = || Ok(VirtualDeck::default());
        s.step(&mut open);
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 1,
                event: decksmith_core::RawEvent::Key {
                    index: 1,
                    pressed: true,
                },
            })
            .unwrap();
        assert!(matches!(s.step(&mut open), Some(Record::Input { .. })));
        s.device
            .as_mut()
            .unwrap()
            .inject(InputEvent {
                timestamp_ms: 2,
                event: decksmith_core::RawEvent::Key {
                    index: 1,
                    pressed: false,
                },
            })
            .unwrap();
        assert!(matches!(s.step(&mut open), Some(Record::Input { .. })));
        assert!(s.display.pending());
        for _ in 0..6 {
            assert!(s.step(&mut open).is_none());
        }
        assert!(matches!(
            s.step(&mut open),
            Some(Record::DisplaySettled { page: 0, .. })
        ));
        assert!(!s.display.pending());
    }
    #[test]
    fn brightness_requests_use_owned_device_and_reject_disconnected_state() {
        use crate::control::{Command, Context, Status};
        let (tx, rx) = std::sync::mpsc::sync_channel(2);
        let status = std::sync::Arc::new(std::sync::Mutex::new(Status {
            api_version: 1,
            connected: false,
            layout: "audio".into(),
            brightness: None,
            display_ready: false,
            active_page: None,
            layout_json: "{}".into(),
            touch_frame: None,
            touch_state: Default::default(),
            attention: Vec::new(),
        }));
        let mut s = Session {
            resume_pending: false,
            reopen_after: None,
            foreground_revision: 0,
            lock_epoch: 0,
            locked: false,
            drain_until: None,
            device: None,
            generation: 0,
            waiting_reported: false,
            pages: None,
            audio: None,
            meter: None,
            ptt: None,
            feedback: None,
            display: crate::display::Display::default(),
            settlement_due: false,
            controls: Some(Context {
                foreground: Default::default(),
                gate: Default::default(),
                incoming: rx,
                status: status.clone(),
            }),
        };
        let (answer, response) = std::sync::mpsc::sync_channel(1);
        tx.send((
            Command::Brightness {
                percent: 30,
                answer,
            },
            1,
            0,
        ))
        .unwrap();
        s.step(&mut || Err::<VirtualDeck, _>(DeviceError::DeviceSelection));
        assert_eq!(response.try_recv().unwrap(), Err("device_disconnected"));
        assert_eq!(status.lock().unwrap().brightness, None);
        s.step(&mut || Ok(VirtualDeck::default()));
        let (answer, response) = std::sync::mpsc::sync_channel(1);
        tx.send((
            Command::Brightness {
                percent: 30,
                answer,
            },
            1,
            0,
        ))
        .unwrap();
        s.step(&mut || Ok(VirtualDeck::default()));
        assert_eq!(response.try_recv().unwrap(), Ok(()));
        assert_eq!(s.device.as_ref().unwrap().brightness, 30);
        assert_eq!(status.lock().unwrap().brightness, Some(30));
        s.pages =
            Some(crate::pages::Pages::parse(include_bytes!("../../../config/audio.json")).unwrap());
        for (page, expected) in [(1, Ok(())), (255, Err("invalid_page")), (0, Ok(()))] {
            let (answer, response) = std::sync::mpsc::sync_channel(1);
            tx.send((Command::Page { page, answer }, 1, 0)).unwrap();
            s.step(&mut || Ok(VirtualDeck::default()));
            assert_eq!(response.try_recv().unwrap(), expected);
            assert_eq!(
                s.pages.as_ref().unwrap().index,
                if page == 255 { 1 } else { page }
            );
        }
    }
    #[test]
    fn backpressure_fails_closed() {
        let (tx, _rx) = std::sync::mpsc::sync_channel(1);
        tx.send(Record::Waiting).unwrap();
        assert_eq!(
            run(
                || Ok(VirtualDeck::default()),
                tx,
                Arc::new(AtomicBool::new(false)),
                None,
                None
            ),
            Err("event_queue_full")
        );
    }
}
