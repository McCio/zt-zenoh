use zenoh::bytes::ZBytes;
use crate::utils::publisher;
use crate::utils::rate_producer::RateProducer;

#[derive(strum_macros::IntoStaticStr)]
pub enum Status {
    Closed,
    Opened,
}

impl From<Status> for ZBytes {
    fn from(value: Status) -> Self {
        <&'static str>::from(value).into()
    }
}

impl publisher::Computer<Status> for RateProducer<'_> {
    fn compute(&self) -> Status {
        Status::Closed
    }
}
impl publisher::FixedIntervalPublisher<Status> for RateProducer<'_> {}
