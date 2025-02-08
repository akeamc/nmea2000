/// Because the maximum size of a frame is 8 bytes, NMEA 2000 splits larger
/// messages into multiple frames, so-called Fast Packets. Each frame contains
/// some group number, frame number, and the actual data. The first frame also
/// contains the length of the total message transmitted ([`FastPacket::total_len`]).
pub struct FastPacket([u8; 8]);

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
