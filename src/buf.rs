pub trait MessageBuf {
    fn get_u8(&mut self) -> u8;

    fn get_i8(&mut self) -> i8;

    fn get_u16(&mut self) -> u16;

    fn get_i16(&mut self) -> i16;

    fn get_u32(&mut self) -> u32;

    fn get_i32(&mut self) -> i32;

    /// Get a floating point number from a fixed point 2-byte representation.
    ///
    /// ```
    /// # use nmea2000::MessageBuf;
    /// let mut buf = &[0, 11][..];
    /// assert_eq!(buf.get_word_f32(0.01), Some(28.16)); // something something endianess
    /// ```
    #[inline]
    fn get_fixed_f32(&mut self, precision: f32) -> Option<f32> {
        match self.get_i16() {
            0x7fff => None, // 0x7fff signals that the value is not available
            value => Some(value as f32 * precision),
        }
    }
}

impl<T> MessageBuf for T
where
    T: bytes::Buf,
{
    #[inline]
    fn get_u8(&mut self) -> u8 {
        self.get_u8()
    }

    #[inline]
    fn get_i8(&mut self) -> i8 {
        self.get_i8()
    }

    #[inline]
    fn get_u16(&mut self) -> u16 {
        self.get_u16_le()
    }

    #[inline]
    fn get_i16(&mut self) -> i16 {
        self.get_i16_le()
    }

    #[inline]
    fn get_u32(&mut self) -> u32 {
        self.get_u32_le()
    }

    #[inline]
    fn get_i32(&mut self) -> i32 {
        self.get_i32_le()
    }
}
