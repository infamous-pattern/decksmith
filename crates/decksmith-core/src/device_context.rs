//! Host-independent addressing. Physical serials and transports never enter IPC context.
use crate::device_model::ControlAddress;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Opaque durable identifier. The host supplies 128 random bits; never derive
/// this from enumeration order, a USB path, a serial or a display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId([u8; 16]);
impl DeviceId {
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, &'static str> {
        if bytes == [0; 16] {
            return Err("invalid_device_id");
        }
        Ok(Self(bytes))
    }
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        if text.len() != 32
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("invalid_device_id");
        }
        let mut bytes = [0; 16];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
                .map_err(|_| "invalid_device_id")?;
        }
        Self::from_bytes(bytes)
    }
}
impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
impl Serialize for DeviceId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A session fence, not an authorization or plugin-permission grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeviceContext {
    /// Process-local supervisor instance. A future wire API also needs a
    /// startup nonce so replies from an earlier daemon process cannot match.
    pub supervisor: u64,
    pub device: DeviceId,
    pub connection: u64,
    pub binding: u64,
    pub lock_epoch: u64,
    pub control: ControlAddress,
}
