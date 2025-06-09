#[cfg(feature = "defmt")]
use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    zerocopy_channel::{Channel, Receiver, Sender},
};
use embassy_time::{Duration, Timer};
use embedded_can::Frame;
use generic_array::GenericArray;

use crate::{
    id::DESTINATION_BROADCAST,
    well_known::{DeviceName, IsoAddressClaim},
    Id, Message, NmeaFrame,
};

mod async_can;

pub use async_can::AsyncCan;

pub const ADDRESS_CLAIM_TIMEOUT: Duration = Duration::from_millis(250);
pub const MIN_SRC: u8 = 1;
pub const MAX_SRC: u8 = 254;

/// NMEA 2000 addresses are distributed to the devices on the bus
/// by having a new device A sending an address claim message upon
/// startup. If another device B has the same address, it will
/// refute the address claim and A will have to choose a new address.
/// The process is repeated until A has a uniqie address.
struct AddressClaimState {
    timer: Option<Timer>,
}

impl AddressClaimState {
    const fn new() -> Self {
        Self { timer: None }
    }

    /// Returns true if the address claim timer is running or has expired,
    /// both of which indicate that the address claim process has started
    /// (and might have finished).
    fn is_started(&self) -> bool {
        self.timer.is_some()
    }

    fn restart_timer(&mut self) {
        self.timer = Some(Timer::after(ADDRESS_CLAIM_TIMEOUT));
    }
}

pub struct Client<'ch, C: AsyncCan> {
    name: DeviceName,
    src: u8,
    can: C,
    address_claim: AddressClaimState,
    rx: Receiver<'ch, CriticalSectionRawMutex, NmeaFrame>,
}

pub enum Error<C: AsyncCan> {
    Can(C::Error),
    Decode,
}

pub struct ClientHandle<'ch> {
    tx: Sender<'ch, CriticalSectionRawMutex, NmeaFrame>,
    group_no: u8,
}

async fn receive_n2k<C>(mut can: C) -> Result<Option<NmeaFrame>, C::Error>
where
    C: AsyncCan,
{
    let frame = can.receive().await?;

    let id = match frame.id() {
        embedded_can::Id::Extended(extended_id) => Id::from_can_id(extended_id),
        embedded_can::Id::Standard(_) => return Ok(None), // confused
    };

    Ok(Some(NmeaFrame::new(
        id,
        frame
            .data()
            .try_into()
            .expect("frame data should not be larger than 8 bytes"),
    )))
}

impl<'ch, C: AsyncCan> Client<'ch, C> {
    pub fn new(
        name: impl Into<DeviceName>,
        can: C,
        channel: &'ch mut Channel<'_, CriticalSectionRawMutex, NmeaFrame>,
    ) -> (Self, ClientHandle<'ch>) {
        let (tx, rx) = channel.split();

        let client = Self::from_receiver(name, can, rx);
        let handle = ClientHandle { tx, group_no: 0 };

        (client, handle)
    }

    pub fn from_receiver(
        name: impl Into<DeviceName>,
        can: C,
        rx: Receiver<'ch, CriticalSectionRawMutex, NmeaFrame>,
    ) -> Self {
        Self {
            name: name.into(),
            src: MIN_SRC,
            can,
            address_claim: AddressClaimState::new(),
            rx,
        }
    }

    fn incr_src(&mut self) {
        if self.src > MAX_SRC {
            self.src = MIN_SRC;
        } else {
            self.src += 1;
        }
    }

    async fn handle_system_message(&mut self, frame: &NmeaFrame) -> Result<(), Error<C>> {
        if frame.id.pgn() == IsoAddressClaim::PGN {
            self.handle_incoming_address_claim(
                frame.id.source(),
                IsoAddressClaim::decode(&frame.data).unwrap(),
            )
            .await
            .map_err(Error::Can)?;
        }

        Ok(())
    }

    async fn handle_incoming_address_claim(
        &mut self,
        src: u8,
        claim: IsoAddressClaim,
    ) -> Result<(), C::Error> {
        #[cfg(feature = "defmt")]
        info!("Received ISO Address Claim from {}", src);

        if src != self.src {
            // ignore claims from other sources than our own
            return Ok(());
        }

        if self.name < claim.name {
            // re-claim address
            #[cfg(feature = "defmt")]
            info!("Reclaiming address {}", src);
            self.send_address_claim().await?;
        } else {
            self.incr_src();
            self.start_address_claim().await?;
        }

        Ok(())
    }

    async fn start_address_claim(&mut self) -> Result<(), C::Error> {
        self.send_address_claim().await?;
        self.address_claim.restart_timer();
        Ok(())
    }

    pub async fn send_address_claim(&mut self) -> Result<(), C::Error> {
        let id = Id::new(6, IsoAddressClaim::PGN, self.src, DESTINATION_BROADCAST);
        let frame = NmeaFrame::from_message(id, &IsoAddressClaim { name: self.name });
        self.can.send(frame.to_can_frame()).await
    }

    pub async fn poll(&mut self) -> Result<NmeaFrame, Error<C>> {
        loop {
            if !self.address_claim.is_started() {
                self.start_address_claim().await.map_err(Error::Can)?;
            }

            let send_fut = async {
                // wait for the address claim timer to expire before sending
                if let Some(timer) = &mut self.address_claim.timer {
                    timer.await;
                }

                self.rx.receive().await
            };

            match select(send_fut, receive_n2k(&mut self.can)).await {
                Either::First(f) => {
                    #[cfg(feature = "defmt")]
                    info!("Sending frame");

                    f.id.set_source(self.src);
                    self.can.send(f.to_can_frame()).await.map_err(Error::Can)?;

                    #[cfg(feature = "defmt")]
                    info!("Sent NMEA frame: {:?}", f);

                    self.rx.receive_done();
                }
                Either::Second(res) => {
                    if let Some(f) = res.map_err(Error::Can)? {
                        self.handle_system_message(&f).await?;

                        return Ok(f);
                    }
                }
            }
        }
    }
}

impl<'a> ClientHandle<'a> {
    pub async fn send(&mut self, frame: NmeaFrame) {
        *self.tx.send().await = frame;
        self.tx.send_done();
    }

    /// Send a fast packet message. See [`crate::fast_packet`] for more
    /// information.
    pub async fn send_fast_packet<T>(&mut self, msg: T, prio: u8, dest: u8)
    where
        T: Message,
    {
        let id = Id::new(prio, T::PGN, 0, dest);

        let mut buf: GenericArray<u8, T::EncodedLen> = GenericArray::default();

        self.group_no = self.group_no.wrapping_add(1);

        for fast_packet in msg.encode_to_fast_packets(buf.as_mut_slice(), self.group_no) {
            let frame = NmeaFrame::new(id, fast_packet.0.as_ref().try_into().unwrap());
            self.send(frame).await;
        }
    }
}
