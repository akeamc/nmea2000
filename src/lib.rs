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
#![allow(async_fn_in_trait)]

mod buf;
#[cfg(feature = "client")]
pub mod client;
pub mod fast_packet;
mod frame;
pub mod id;
pub mod well_known;

use generic_array::{typenum::Unsigned, ArrayLength};

pub use buf::{Buf, BufMut};
#[cfg(feature = "client")]
pub use client::Client;
pub use fast_packet::FastPacket;
pub use frame::NmeaFrame;
pub use generic_array::{typenum, GenericArray};
pub use id::Id;

/// A NMEA 2000 message. This trait is very much inspired by [the gRPC library
/// Prost's trait with the same name](https://docs.rs/prost/latest/prost/trait.Message.html).
pub trait Message {
    const PGN: u32;

    /// Total length of the encoded message in bytes.
    type EncodedLen: ArrayLength;

    /// The error type returned when a message fails to decode.
    type DecodeError;

    fn encode(&self, buf: &mut [u8]);

    fn encode_to_fast_packets<'a>(&self, buf: &'a mut [u8], group_no: u8) -> fast_packet::Iter<'a> {
        self.encode(buf);
        fast_packet::Iter::new(&buf[..Self::EncodedLen::USIZE], group_no)
    }

    /// Decode a message from its encoded form.
    fn decode(data: &[u8]) -> Result<Self, Self::DecodeError>
    where
        Self: Sized;
}
