/// Until the [`embedded-can`] crate supports async, we need to define our own
/// trait.
pub trait AsyncCan {
    type Error;

    type Frame: embedded_can::Frame;

    async fn send(&mut self, frame: Self::Frame) -> Result<(), Self::Error>;

    async fn receive(&mut self) -> Result<Self::Frame, Self::Error>;
}

impl<T> AsyncCan for &mut T
where
    T: AsyncCan,
{
    type Error = T::Error;
    type Frame = T::Frame;

    async fn send(&mut self, frame: Self::Frame) -> Result<(), Self::Error> {
        (*self).send(frame).await
    }

    async fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        (*self).receive().await
    }
}
