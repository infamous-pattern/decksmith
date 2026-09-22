//! Explicit-clock replay for one virtual Plus. No hardware or wall-clock access.
use decksmith_core::{
    InputEvent, RawEvent,
    trigger::{Bindings, PressResolver, TimingPolicy, Trigger},
};
use decksmith_device::VirtualDeck;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Input { input: InputEvent },
    Advance { timestamp_ms: u64 },
    Disconnect { timestamp_ms: u64 },
    Reconnect { timestamp_ms: u64 },
}
#[derive(Serialize)]
#[serde(tag = "record", rename_all = "snake_case")]
pub enum Record {
    Trigger {
        device_id: &'static str,
        control: usize,
        trigger: Trigger,
    },
    Passthrough {
        device_id: &'static str,
        input: InputEvent,
    },
    Lifecycle {
        device_id: &'static str,
        timestamp_ms: u64,
        connected: bool,
    },
}
pub struct Replay {
    deck: VirtualDeck,
    controls: Vec<PressResolver>,
    now: u64,
}
impl Replay {
    pub fn new() -> Self {
        let resolver = PressResolver::new(
            TimingPolicy::default(),
            Bindings {
                double: true,
                repeat: true,
            },
        )
        .expect("valid default timing policy");
        Self {
            deck: VirtualDeck::default(),
            controls: vec![resolver; 12],
            now: 0,
        }
    }
    pub fn apply(&mut self, command: Command) -> Result<Vec<Record>, super::CliError> {
        let timestamp_ms = match &command {
            Command::Input { input } => input.timestamp_ms,
            Command::Advance { timestamp_ms }
            | Command::Disconnect { timestamp_ms }
            | Command::Reconnect { timestamp_ms } => *timestamp_ms,
        };
        if timestamp_ms < self.now {
            return Err(super::CliError::InvalidEvent);
        }
        let mut out = Vec::new();
        match command {
            Command::Input { input } => {
                self.deck
                    .inject(input)
                    .map_err(|_| super::CliError::InvalidEvent)?;
                let input = self
                    .deck
                    .receive_event()
                    .ok_or(super::CliError::Invariant)?;
                let edge = match input.event {
                    RawEvent::Key { index, pressed } => Some((index as usize, pressed)),
                    RawEvent::DialPush { index, pressed } => Some((8 + index as usize, pressed)),
                    _ => None,
                };
                if let Some((control, pressed)) = edge {
                    let triggers = self.controls[control]
                        .edge(timestamp_ms, pressed)
                        .map_err(|_| super::CliError::InvalidEvent)?;
                    out.extend(triggers.into_iter().map(|trigger| Record::Trigger {
                        device_id: "virtual-plus-0",
                        control,
                        trigger,
                    }));
                } else {
                    out.push(Record::Passthrough {
                        device_id: "virtual-plus-0",
                        input,
                    });
                }
            }
            Command::Advance { .. } => {
                // Commit all clocks together only after every resolver succeeds.
                let mut controls = self.controls.clone();
                for (control, r) in controls.iter_mut().enumerate() {
                    out.extend(
                        r.advance(timestamp_ms)
                            .map_err(|_| super::CliError::InvalidEvent)?
                            .into_iter()
                            .map(|trigger| Record::Trigger {
                                device_id: "virtual-plus-0",
                                control,
                                trigger,
                            }),
                    );
                }
                out.sort_by_key(|r| match r {
                    Record::Trigger {
                        control, trigger, ..
                    } => (trigger.timestamp_ms, *control),
                    _ => unreachable!(),
                });
                self.controls = controls;
            }
            Command::Disconnect { .. } => {
                self.deck.disconnect();
                for r in &mut self.controls {
                    r.cancel();
                }
                out.push(Record::Lifecycle {
                    device_id: "virtual-plus-0",
                    timestamp_ms,
                    connected: false,
                });
            }
            Command::Reconnect { .. } => {
                self.deck.reconnect();
                out.push(Record::Lifecycle {
                    device_id: "virtual-plus-0",
                    timestamp_ms,
                    connected: true,
                });
            }
        }
        self.now = timestamp_ms;
        Ok(out)
    }
}
