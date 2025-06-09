pub trait Buf {
    fn get_u8(&mut self) -> u8;

    fn get_i8(&mut self) -> i8;

    fn get_u16(&mut self) -> u16;

    fn get_i16(&mut self) -> i16;

    fn get_u24(&mut self) -> u32;

    fn get_i24(&mut self) -> i32;

    fn get_u32(&mut self) -> u32;

    fn get_i32(&mut self) -> i32;

    fn get_u64(&mut self) -> u64;

    fn get_i64(&mut self) -> i64;

    /// Get a floating point number from a fixed point 2-byte representation.
    ///
    /// ```
    /// # use nmea2000::Buf;
    /// let mut buf = &[0, 11][..];
    /// assert_eq!(buf.get_fixed_f32(0.01), Some(28.16)); // something something endianess
    /// ```
    #[inline]
    fn get_fixed_f32(&mut self, precision: f32) -> Option<f32> {
        match self.get_i16() {
            0x7fff => None, // 0x7fff signals that the value is not available
            value => Some(value as f32 * precision),
        }
    }
}

impl Buf for &[u8] {
    #[inline]
    fn get_u8(&mut self) -> u8 {
        *self.split_off_first().unwrap()
    }

    #[inline]
    fn get_i8(&mut self) -> i8 {
        *self.split_off_first().unwrap() as i8
    }

    #[inline]
    fn get_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.split_off(..2).unwrap().try_into().unwrap())
    }

    #[inline]
    fn get_i16(&mut self) -> i16 {
        i16::from_le_bytes(self.split_off(..2).unwrap().try_into().unwrap())
    }

    #[inline]
    fn get_u24(&mut self) -> u32 {
        let mut bytes = [0; 4];
        bytes[..3].copy_from_slice(self.split_off(..3).unwrap());
        u32::from_le_bytes(bytes)
    }

    #[inline]
    fn get_i24(&mut self) -> i32 {
        let mut bytes = [0; 4];
        bytes[..3].copy_from_slice(self.split_off(..3).unwrap());
        i32::from_le_bytes(bytes)
    }

    #[inline]
    fn get_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.split_off(..4).unwrap().try_into().unwrap())
    }

    #[inline]
    fn get_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.split_off(..8).unwrap().try_into().unwrap())
    }

    #[inline]
    fn get_i64(&mut self) -> i64 {
        i64::from_le_bytes(self.split_off(..8).unwrap().try_into().unwrap())
    }

    #[inline]
    fn get_i32(&mut self) -> i32 {
        i32::from_le_bytes(self.split_off(..4).unwrap().try_into().unwrap())
    }
}

pub trait BufMut {
    fn put_u8(&mut self, value: u8);

    fn put_i8(&mut self, value: i8);

    fn put_u16(&mut self, value: u16);

    fn put_i16(&mut self, value: i16);

    fn put_u24(&mut self, value: u32);

    fn put_i24(&mut self, value: i32);

    fn put_u32(&mut self, value: u32);

    fn put_i32(&mut self, value: i32);

    fn put_u64(&mut self, value: u64);

    fn put_i64(&mut self, value: i64);

    /// Put a floating point number into a fixed point 2-byte representation.
    fn put_fixed_f32(&mut self, value: f32, precision: f32) {
        let value = (value / precision) as i16;
        self.put_i16(value);
    }
}

impl BufMut for &mut [u8] {
    #[inline]
    fn put_u8(&mut self, value: u8) {
        *self.split_off_first_mut().unwrap() = value;
    }

    #[inline]
    fn put_i8(&mut self, value: i8) {
        *self.split_off_first_mut().unwrap() = value as u8;
    }

    #[inline]
    fn put_u16(&mut self, value: u16) {
        self.split_off_mut(..2)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_i16(&mut self, value: i16) {
        self.split_off_mut(..2)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_u24(&mut self, value: u32) {
        let mut bytes = [0; 4];
        bytes[..3].copy_from_slice(&value.to_le_bytes()[..3]);
        self.split_off_mut(..3).unwrap().copy_from_slice(&bytes);
    }

    #[inline]
    fn put_i24(&mut self, value: i32) {
        let mut bytes = [0; 4];
        bytes[..3].copy_from_slice(&value.to_le_bytes()[..3]);
        self.split_off_mut(..3).unwrap().copy_from_slice(&bytes);
    }

    #[inline]
    fn put_u32(&mut self, value: u32) {
        self.split_off_mut(..4)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_u64(&mut self, value: u64) {
        self.split_off_mut(..8)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_i64(&mut self, value: i64) {
        self.split_off_mut(..8)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_i32(&mut self, value: i32) {
        self.split_off_mut(..4)
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
    }
}
