#![no_std]

pub mod fast_packet;

pub use fast_packet::FastPacket;
use generic_array::ArrayLength;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanId(pub u32);

impl CanId {
    pub const fn prio(self) -> u8 {
        (self.0 >> 26) as u8 & 0x7
    }

    pub const fn pgn(self) -> u32 {
        (self.0 >> 8) & 0x3ffff
    }

    pub const fn src(self) -> u8 {
        self.0 as u8
    }
}

/// Parse a 2-byte float value with the given precision.
pub const fn parse_2_byte_float(bytes: [u8; 2], precision: f32) -> Option<f32> {
    match i16::from_le_bytes(bytes) {
        0x7fff => None, // This value signals that the data is not available
        value => Some(value as f32 * precision),
    }
}

/// A NMEA 2000 message. This trait is very much inspired by [the gRPC library
/// Prost's trait with the same name](https://docs.rs/prost/latest/prost/trait.Message.html).
pub trait Message {
    /// Total length of the encoded message in bytes.
    type EncodedLen: ArrayLength;

    /// The error type returned when a message fails to decode.
    type DecodeError;

    /// Decode a message from its encoded form.
    fn decode(data: &[u8]) -> Result<Self, Self::DecodeError>
    where
        Self: Sized;
}
