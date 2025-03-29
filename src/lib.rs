//! This crate provides basic decoding of NMEA 2000 (N2K) messages. It uses no
//! heap allocation whatsoever and is designed to be used in embedded systems.
//! As the author wrote this just a few hours after first learning about N2K,
//! it is very much a work in progress. Nevertheless, [the Canboat project's
//! reverse engineering documentation](https://canboat.github.io/canboat/canboat.html)
//! proved to be very helpful in understanding the protocol.
//!
//! It has only been tested on the ESP32 microcontroller with the ESP-IDF SDK,
//! but there is no reason it should not work on other platforms.

#![no_std]

mod buf;
pub mod fast_packet;

use embedded_can::ExtendedId;
use generic_array::{ArrayLength, GenericArray};

pub use buf::MessageBuf;
pub use fast_packet::FastPacket;
pub use generic_array::typenum;

/// A NMEA 2000 message identifier. According to N2K specification, this is a
/// 29-bit extended CAN ID with a 3-bit priority, a 19-bit parameter group
/// number (PGN), and an 8-bit source address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier(ExtendedId);

impl Identifier {
    /// Create a new identifier from an extended CAN ID.
    #[inline]
    #[must_use]
    pub const fn from_can_id(can_id: ExtendedId) -> Self {
        Self(can_id)
    }

    #[inline]
    #[must_use]
    pub const fn as_can_id(self) -> ExtendedId {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn priority(self) -> u8 {
        (self.0.as_raw() >> 26) as u8 & 0x7
    }

    #[inline]
    #[must_use]
    pub fn pgn(self) -> u32 {
        (self.0.as_raw() >> 8) & 0x3ffff
    }

    #[inline]
    #[must_use]
    pub fn source(self) -> u8 {
        self.0.as_raw() as u8
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
    fn decode(data: &GenericArray<u8, Self::EncodedLen>) -> Result<Self, Self::DecodeError>
    where
        Self: Sized;
}
