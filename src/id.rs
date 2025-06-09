use embedded_can::ExtendedId;

/// A NMEA 2000 message identifier. According to N2K specification, this is a
/// 29-bit extended CAN ID with a 3-bit priority, a 18-bit parameter group
/// number (PGN), and an 8-bit source address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(embedded_can::ExtendedId);

pub enum Format {
    Pdu1,
    Pdu2,
}

impl Format {
    #[inline]
    #[must_use]
    pub fn from_pgn(pgn: u32) -> Self {
        let id_pf = pgn >> 8;

        if id_pf < 240 {
            Self::Pdu1
        } else {
            Self::Pdu2
        }
    }
}

pub const DESTINATION_BROADCAST: u8 = 0xff;

impl Id {
    #[inline]
    #[must_use]
    pub const fn new(priority: u8, pgn: u32, source: u8, destination: u8) -> Self {
        debug_assert!(priority <= 7, "Priority must be in the range 0-7");
        debug_assert!(pgn <= 0x3ffff, "PGN must be less than 0x3ffff (18 bits");

        let id_pf = pgn >> 8;

        let id = if id_pf < 240 {
            // PDU1
            debug_assert!(pgn & 0xff == 0, "lowest byte of PGN has to be 0 for PDU1");
            (priority as u32 & 0x7) << 26 | pgn << 8 | (destination as u32) << 8 | source as u32
        } else {
            // PDU2
            //
            // The priority is in bits 26-28, the PGN in bits 8-25, and the source
            // address in bits 0-7.
            (priority as u32 & 0x7) << 26 | pgn << 8 | source as u32
        };

        Self(embedded_can::ExtendedId::new(id).unwrap())
    }

    /// Create a new identifier from an extended CAN ID.
    #[inline]
    #[must_use]
    pub const fn from_can_id(can_id: embedded_can::ExtendedId) -> Self {
        Self(can_id)
    }

    #[inline]
    #[must_use]
    pub const fn as_can_id(self) -> embedded_can::ExtendedId {
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
        match self.format() {
            Format::Pdu1 => (self.0.as_raw() >> 8) & 0x3ff00,
            Format::Pdu2 => (self.0.as_raw() >> 8) & 0x3ffff,
        }
    }

    #[inline]
    #[must_use]
    pub fn source(self) -> u8 {
        self.0.as_raw() as u8
    }

    #[inline]
    pub fn set_source(&mut self, source: u8) {
        self.0 = ExtendedId::new((self.0.as_raw() & 0xffffff00) | (source as u32)).unwrap();
    }

    #[inline]
    #[must_use]
    pub fn format(self) -> Format {
        if (self.0.as_raw() >> 16) < 240 {
            Format::Pdu1
        } else {
            Format::Pdu2
        }
    }

    #[inline]
    #[must_use]
    pub fn destination(self) -> u8 {
        match self.format() {
            Format::Pdu1 => (self.0.as_raw() >> 8) as u8,
            Format::Pdu2 => DESTINATION_BROADCAST, // implied global
        }
    }
}

impl From<embedded_can::ExtendedId> for Id {
    fn from(id: embedded_can::ExtendedId) -> Self {
        Self::from_can_id(id)
    }
}

impl From<Id> for embedded_can::Id {
    fn from(id: Id) -> Self {
        Self::Extended(id.as_can_id())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Id {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Id({:x})", self.0.as_raw())
    }
}
