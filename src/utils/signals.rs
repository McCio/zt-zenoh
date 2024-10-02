use std::str::FromStr;
use strum_macros::Display;
use zenoh::bytes::ZBytes;
use crate::utils::publisher;
use crate::utils::rate_producer::RateProducer;

#[derive(strum_macros::IntoStaticStr, Debug, Display)]
pub enum Status {
    Closed,
    Opened,
}

impl From<Status> for ZBytes {
    fn from(value: Status) -> Self {
        <&'static str>::from(value).into()
    }
}

impl FromStr for Status {
    type Err = zenoh::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Opened" => Ok(Status::Opened),
            "Closed" => Ok(Status::Closed),
            _ => Err(Self::Err::from("Unknown Status")),
        }
    }
}

impl From<ZBytes> for Status {
    fn from(value: ZBytes) -> Self {
        let o2: String = value.deserialize().unwrap();
        o2.parse().unwrap()
    }
}
impl zenoh::bytes::Deserialize<Status> for zenoh::bytes::ZSerde {
    type Input<'a> = &'a ZBytes;
    type Error = zenoh::Error;

    fn deserialize(self, t: Self::Input<'_>) -> Result<Status, Self::Error> {
        match t.deserialize::<String>() {
            Ok(x) => x.parse(),
            Err(e) => Err(e.into()),
        }
    }
}

impl publisher::Computer<Status> for RateProducer<'_> {
    fn compute(&self) -> Status {
        Status::Closed
    }
}
impl publisher::FixedIntervalPublisher<Status> for RateProducer<'_> {}
