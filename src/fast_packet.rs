//! Because the maximum size of a frame is 8 bytes, NMEA 2000 splits larger
//! messages into multiple frames, so-called Fast Packets. Each frame contains
//! some group number, frame number, and the actual data. The first frame also
//! contains the length of the total message transmitted ([`FastPacket::total_len`]).

use generic_array::{
    typenum::{self, IsGreater, True, Unsigned},
    GenericArray,
};

use crate::Message;

/// See the [module-level documentation](self) for more information.
pub struct FastPacket(pub [u8; 8]);

impl FastPacket {
    /// The sequence number of the frame within the group, starting from 0.
    #[inline]
    #[must_use]
    pub const fn frame_no(&self) -> u8 {
        self.0[0] & 0b1111
    }

    /// The group number of the frames. All frames of the same group can be
    /// combined to form the original message.
    #[inline]
    #[must_use]
    pub const fn group_no(&self) -> u8 {
        self.0[0] >> 4
    }

    #[inline]
    #[must_use]
    pub const fn is_first(&self) -> bool {
        self.frame_no() == 0
    }

    #[inline]
    #[must_use]
    pub const fn total_len(&self) -> Option<u8> {
        if self.is_first() {
            Some(self.0[1])
        } else {
            None
        }
    }

    /// The data contained in the frame.
    ///
    /// If this is the last frame of the group, the returned slice might be
    /// padded, so you should save the value of  [`FastPacket::total_len`] at
    /// the beginning of the group and only use the first `total_len` bytes.
    #[inline]
    #[must_use]
    pub fn data(&self) -> &[u8] {
        if self.is_first() {
            &self.0[2..]
        } else {
            &self.0[1..]
        }
    }
}

/// A marker trait for messages that are transmitted as [Fast Packets](FastPacket).
///
/// This trait is automatically implemented for all messages that have a length greater than 8 bytes.
pub trait FastPacketMessage: Message {}

// all messages with a length greater than 8 bytes are fast packet messages
impl<T> FastPacketMessage for T
where
    T: Message,
    T::EncodedLen: IsGreater<typenum::U8, Output = True>,
{
}

/// A reader for fast packets that combines the frames of a group into a single message.
pub struct Reader<T: FastPacketMessage> {
    buf: GenericArray<u8, T::EncodedLen>,
    group_no: u8,
    buf_pos: usize,
    _marker: core::marker::PhantomData<T>,
}

impl<T> Default for Reader<T>
where
    T: FastPacketMessage,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Reader<T>
where
    T: FastPacketMessage,
{
    pub fn new() -> Self {
        Self {
            buf: Default::default(),
            // since the group number is 4 bits, this will always be different from the first group's number
            group_no: 0xff,
            buf_pos: 0,
            _marker: core::marker::PhantomData,
        }
    }

    const fn expected_frame_no(&self) -> u8 {
        (self.buf_pos as u8 + 1) / 7
    }

    fn bytes_remaining(&self) -> usize {
        T::EncodedLen::to_usize() - self.buf_pos
    }

    /// Reads a fast packet and tries to decode the message if all frames have been received.
    ///
    /// Packets that belong to a different group than the previous packet are ignored unless
    /// they are the first packet of a new group.
    pub fn read(&mut self, packet: FastPacket) -> Option<Result<T, T::DecodeError>> {
        if packet.group_no() != self.group_no {
            if packet.is_first() {
                if packet.total_len() != Some(T::EncodedLen::to_u8()) {
                    // should we return an error here?
                    return None;
                }

                self.group_no = packet.group_no();
                self.buf_pos = 0;
            } else {
                return None;
            }
        }

        if packet.frame_no() != self.expected_frame_no() {
            // out of order?
            return None;
        }

        let data_len = packet.data().len().min(self.bytes_remaining());
        let data = &packet.data()[..data_len];
        self.buf[self.buf_pos..self.buf_pos + data.len()].copy_from_slice(data);
        self.buf_pos += data.len();

        if self.buf_pos == T::EncodedLen::to_usize() {
            Some(T::decode(&self.buf))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use generic_array::typenum;

    use crate::Message;

    use super::FastPacket;

    #[test]
    fn read_fast_packets() {
        #[derive(Debug, PartialEq)]
        struct TestMessage;

        impl Message for TestMessage {
            type EncodedLen = typenum::U10;

            type DecodeError = ();

            fn decode(data: &[u8]) -> Result<Self, Self::DecodeError>
            where
                Self: Sized,
            {
                if data == [0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x42, 0x42, 0x42, 0x42] {
                    Ok(Self)
                } else {
                    Err(())
                }
            }
        }

        let p1 = FastPacket([0b0010_0000, 10, 0xde, 0xad, 0xbe, 0xef, 0x00, 0x00]);
        let p2 = FastPacket([0b0010_0001, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00]);

        let mut reader = super::Reader::<TestMessage>::new();

        assert_eq!(reader.read(p1), None);
        assert_eq!(reader.read(p2), Some(Ok(TestMessage)));
    }
}
