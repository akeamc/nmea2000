#![no_std]

mod buf;
pub mod fast_packet;

use generic_array::ArrayLength;

pub use buf::MessageBuf;
pub use fast_packet::FastPacket;
pub use generic_array::typenum;

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
