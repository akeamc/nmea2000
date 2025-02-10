//! This crate provides basic decoding of NMEA 2000 (N2K) messages. It uses no
//! heap allocation whatsoever and is designed to be used in embedded systems.
//! As the author wrote this just a few hours after first learning about N2K,
//! it is very much a work in progress. Nevertheless, [the Canboat project's
//! reverse engineering documentation](https://canboat.github.io/canboat/canboat.html)
//! proved to be very helpful in understanding the protocol.
//!
//! It has been tested on the ESP32-S3 microcontroller with the ESP-IDF SDK,
//! but there is no reason it should not work on other platforms.

#![no_std]

mod buf;
pub mod fast_packet;

use generic_array::{ArrayLength, GenericArray};

pub use buf::MessageBuf;
pub use fast_packet::FastPacket;
pub use generic_array::typenum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier(u32);

impl Identifier {
    pub const fn from_can_id(can_id: u32) -> Self {
        Self(can_id)
    }

    pub const fn priority(self) -> u8 {
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
    fn decode(data: &GenericArray<u8, Self::EncodedLen>) -> Result<Self, Self::DecodeError>
    where
        Self: Sized;
}
