use embassy_executor::Executor;
use embassy_futures::block_on;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel, signal::Signal,
    zerocopy_channel,
};
use nmea2000::{typenum, Buf, BufMut, EventLoop, Id, Message, NmeaFrame};
use static_cell::StaticCell;

use crate::bus::{FakeCan, Frame};

mod bus;

// async fn mock_bus_inner() {
//   let mut buf = [NmeaFrame::DEFAULT; 8];
//   let mut channel = zerocopy_channel::Channel::new(&mut buf);
//   let (mut event_loop, client) = Client::new(0xdead_beef, can, &mut channel);

//   loop {
//     event_loop.poll().await.unwrap();
//   }
// }

static ALICE_DONE: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static BOB_DONE: Signal<CriticalSectionRawMutex, u8> = Signal::new();

static CAN: PubSubChannel<CriticalSectionRawMutex, Frame, 8, 8, 8> = PubSubChannel::new();

struct HelloWorld {
    int: u64,
}

impl Message for HelloWorld {
    const PGN: u32 = 130_816;

    type EncodedLen = typenum::U8;
    type DecodeError = ();

    fn encode(&self, mut buf: &mut [u8]) {
        buf.put_u64(self.int);
    }

    fn decode(mut data: &[u8]) -> Result<Self, Self::DecodeError>
    where
        Self: Sized,
    {
        Ok(HelloWorld {
            int: data.get_u64(),
        })
    }
}

#[embassy_executor::task]
async fn alice() {
    let mut buf = [NmeaFrame::DEFAULT; 8];
    let mut channel = zerocopy_channel::Channel::new(&mut buf);
    let can = FakeCan::new(&CAN);
    let (mut event_loop, mut client) = EventLoop::new(0x1234_5678, can, &mut channel);

    client
        .send(NmeaFrame::from_message(
            Id::new(4, HelloWorld::PGN, 0, 0),
            &HelloWorld { int: 37 },
        ))
        .await;

    loop {
        let frame = event_loop.poll().await.unwrap();

        if frame.id.pgn() == HelloWorld::PGN {
            let msg = HelloWorld::decode(&frame.data).unwrap();

            if msg.int == 37 {
                ALICE_DONE.signal(event_loop.src());
            }
        }
    }
}

#[embassy_executor::task]
async fn bob() {
    let mut buf = [NmeaFrame::DEFAULT; 8];
    let mut channel = zerocopy_channel::Channel::new(&mut buf);
    let can = FakeCan::new(&CAN);
    let (mut event_loop, mut client) = EventLoop::new(0xdead_beef, can, &mut channel);

    client
        .send(NmeaFrame::from_message(
            Id::new(4, HelloWorld::PGN, 0, 0),
            &HelloWorld { int: 19 },
        ))
        .await;

    loop {
        let frame = event_loop.poll().await.unwrap();

        if frame.id.pgn() == HelloWorld::PGN {
            let msg = HelloWorld::decode(&frame.data).unwrap();

            if msg.int == 19 {
                BOB_DONE.signal(event_loop.src());
            }
        }
    }
}

#[test]
fn mock_bus() {
    static EXECUTOR: StaticCell<Executor> = StaticCell::new();

    std::thread::spawn(|| {
        EXECUTOR.init_with(Executor::new).run(|spawner| {
            spawner.must_spawn(alice());
            spawner.must_spawn(bob());
        });
    });

    let src_a = block_on(ALICE_DONE.wait());
    let src_b = block_on(BOB_DONE.wait());

    assert_ne!(src_a, src_b);
}
