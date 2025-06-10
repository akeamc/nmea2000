use std::{convert::Infallible, fmt::Debug};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use embedded_can::Id;
use nmea2000::client::AsyncCan;

#[derive(Debug, Clone)]
pub struct Frame {
    id: Id,
    is_remote: bool,
    dlc: usize,
    data: [u8; 8],
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None; // Data length exceeds CAN frame limit
        }

        let mut d = [0; 8];
        d[..data.len()].copy_from_slice(data);

        Some(Self {
            id: id.into(),
            is_remote: false,
            dlc: data.len(),
            data: d,
        })
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn dlc(&self) -> usize {
        self.dlc
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        Some(Self {
            id: id.into(),
            is_remote: true,
            dlc,
            data: [0; 8],
        })
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        self.is_remote
    }

    fn id(&self) -> Id {
        self.id
    }
}

pub struct FakeCan<'a, const CAP: usize, const SUBS: usize, const PUBS: usize> {
    rx: Subscriber<'a, CriticalSectionRawMutex, Frame, CAP, SUBS, PUBS>,
    tx: Publisher<'a, CriticalSectionRawMutex, Frame, CAP, SUBS, PUBS>,
}

impl<const CAP: usize, const SUBS: usize, const PUBS: usize> Debug
    for FakeCan<'_, CAP, SUBS, PUBS>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeCan").finish()
    }
}

impl<'a, const CAP: usize, const SUBS: usize, const PUBS: usize> FakeCan<'a, CAP, SUBS, PUBS> {
    pub fn new(
        channel: &'a PubSubChannel<CriticalSectionRawMutex, Frame, CAP, SUBS, PUBS>,
    ) -> Self {
        Self {
            rx: channel.subscriber().unwrap(),
            tx: channel.publisher().unwrap(),
        }
    }
}

impl<const CAP: usize, const SUBS: usize, const PUBS: usize> AsyncCan
    for FakeCan<'_, CAP, SUBS, PUBS>
{
    type Error = Infallible;

    type Frame = Frame;

    async fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        Ok(self.rx.next_message_pure().await)
    }

    async fn send(&mut self, frame: Self::Frame) -> Result<(), Self::Error> {
        self.tx.publish_immediate(frame);
        Ok(())
    }
}
