//! Shared device contract. Raw input remains separate from semantic triggers.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Geometry {
    pub columns: u8,
    pub rows: u8,
    pub dials: u8,
    pub key_pixels: (u16, u16),
    pub touch_pixels: (u16, u16),
}
impl Geometry {
    pub fn plus() -> Self {
        Self {
            columns: 4,
            rows: 2,
            dials: 4,
            key_pixels: (120, 120),
            touch_pixels: (800, 100),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawEvent {
    Key {
        index: u8,
        pressed: bool,
    },
    DialPush {
        index: u8,
        pressed: bool,
    },
    DialRotate {
        index: u8,
        ticks: i16,
    },
    Touch {
        x: u16,
        y: u16,
        gesture: TouchGesture,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TouchGesture {
    Tap,
    LongPress,
    FlickLeft,
    FlickRight,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputEvent {
    pub timestamp_ms: u64,
    pub event: RawEvent,
}

pub mod trigger;
