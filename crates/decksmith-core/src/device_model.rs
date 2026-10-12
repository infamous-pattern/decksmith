//! Project-owned, bounded model descriptions. No HID types or host identity.
use crate::{Geometry, RawEvent, TouchGesture};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceModel {
    Plus,
    Mk2,
}

/// Typed addressing avoids treating a dial as a key with a model-specific offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "type", content = "index", rename_all = "snake_case")]
pub enum ControlAddress {
    Key(u8),
    DialPush(u8),
    DialRotation(u8),
    Touch,
}
impl RawEvent {
    pub fn control(&self) -> ControlAddress {
        match self {
            Self::Key { index, .. } => ControlAddress::Key(*index),
            Self::DialPush { index, .. } => ControlAddress::DialPush(*index),
            Self::DialRotate { index, .. } => ControlAddress::DialRotation(*index),
            Self::Touch { .. } => ControlAddress::Touch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TouchSurface {
    pub pixels: (u16, u16),
    pub tap: bool,
    pub long_press: bool,
    pub horizontal_swipe: bool,
}
impl TouchSurface {
    pub fn supports(self, gesture: &TouchGesture) -> bool {
        match gesture {
            TouchGesture::Tap => self.tap,
            TouchGesture::LongPress => self.long_press,
            TouchGesture::FlickLeft | TouchGesture::FlickRight => self.horizontal_swipe,
        }
    }
    pub fn rgb_bytes(self) -> usize {
        usize::from(self.pixels.0) * usize::from(self.pixels.1) * 3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeviceCapabilities {
    pub key_images: bool,
    pub brightness: bool,
    pub touch: Option<TouchSurface>,
}

/// Descriptors can only be constructed from the known, bounded model table.
/// No deserialization or arbitrary geometry constructor can bypass those bounds.
/// Physical identity and per-connection state belong to the future device manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceDescriptor {
    model: DeviceModel,
    geometry: Geometry,
    capabilities: DeviceCapabilities,
}
impl DeviceDescriptor {
    pub fn for_model(model: DeviceModel) -> Self {
        let geometry = match model {
            DeviceModel::Plus => Geometry::plus(),
            DeviceModel::Mk2 => Geometry::mk2(),
        };
        let touch = match model {
            DeviceModel::Plus => Some(TouchSurface {
                pixels: geometry.touch_pixels,
                tap: true,
                long_press: true,
                horizontal_swipe: true,
            }),
            DeviceModel::Mk2 => None,
        };
        Self {
            model,
            geometry,
            capabilities: DeviceCapabilities {
                key_images: true,
                brightness: true,
                touch,
            },
        }
    }
    pub fn model(&self) -> DeviceModel {
        self.model
    }
    pub fn geometry(&self) -> Geometry {
        self.geometry.clone()
    }
    pub fn capabilities(&self) -> DeviceCapabilities {
        self.capabilities
    }
    pub fn key_count(&self) -> u8 {
        // Private fields and the known-model constructor keep this within u8.
        self.geometry.columns * self.geometry.rows
    }
    pub fn key_rgb_bytes(&self) -> usize {
        usize::from(self.geometry.key_pixels.0) * usize::from(self.geometry.key_pixels.1) * 3
    }
    pub fn supports(&self, control: ControlAddress) -> bool {
        match control {
            ControlAddress::Key(index) => index < self.key_count(),
            ControlAddress::DialPush(index) | ControlAddress::DialRotation(index) => {
                index < self.geometry.dials
            }
            ControlAddress::Touch => self.capabilities.touch.is_some(),
        }
    }
}
