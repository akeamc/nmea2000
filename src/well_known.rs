use generic_array::typenum;

use crate::{Buf, BufMut, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceName(pub u64);

impl From<u64> for DeviceName {
    fn from(value: u64) -> Self {
        DeviceName(value)
    }
}

pub struct IsoAddressClaim {
    // pub unique_number: u32,
    // pub manufacturer_code: u16,
    // pub device_instance: u8,
    // pub device_function: u8,
    // pub device_class: u8,
    // pub system_instance: u8,
    // pub industry_group: u8,
    pub name: DeviceName,
}

impl Message for IsoAddressClaim {
    const PGN: u32 = 60928;

    type EncodedLen = typenum::U8;

    type DecodeError = ();

    fn encode(&self, mut buf: &mut [u8]) {
        // buf.put_u32(
        //     self.unique_number & 0x1FFFFF | ((self.manufacturer_code & 0x7ff) as u32) << 21,
        // );
        // buf.put_u8(self.device_instance);
        // buf.put_u8(self.device_function);
        // buf.put_u8((self.device_class & 0x7f) << 1);
        // buf.put_u8(0x80 | ((self.industry_group & 0x7) << 4) | (self.system_instance & 0x0f));
        buf.put_u64(self.name.0);
    }

    fn decode(mut data: &[u8]) -> Result<Self, Self::DecodeError>
    where
        Self: Sized,
    {
        Ok(Self {
            name: DeviceName(data.get_u64()),
        })
    }
}
