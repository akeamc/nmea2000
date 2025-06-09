use generic_array::{
    typenum::{self, U8},
    GenericArray,
};

use crate::{Id, Message};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NmeaFrame {
    pub id: Id,
    pub data: heapless::Vec<u8, 8>,
}

impl NmeaFrame {
    pub const DEFAULT: Self = Self {
        id: Id::new(0, 0, 0, 0),
        data: heapless::Vec::new(),
    };

    pub fn new(id: Id, data: heapless::Vec<u8, 8>) -> Self {
        Self { id, data }
    }

    pub fn to_can_frame<T: embedded_can::Frame>(&self) -> T {
        T::new(self.id.as_can_id(), &self.data).unwrap()
    }

    /// Convert a message to a NMEA frame. The typenum bounds are used to
    /// ensure that the message is not larger than 8 bytes. If it is, use
    /// [`crate::fast_packet`] instead.
    pub fn from_message<T: Message>(id: Id, msg: &T) -> Self
    where
        T::EncodedLen: typenum::IsLessOrEqual<U8>,
    {
        let mut buf = GenericArray::<u8, T::EncodedLen>::default();
        msg.encode(&mut buf);

        Self {
            id,
            data: unsafe { heapless::Vec::from_slice(buf.as_slice()).unwrap_unchecked() },
        }
    }
}
