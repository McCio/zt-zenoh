use crate::utils::signals::rnum::NewRand;
use rand::Rng;
use std::str::FromStr;
use strum_macros::Display;
use zenoh::bytes::ZBytes;

#[derive(strum_macros::IntoStaticStr, Debug, Display, Clone)]
pub enum PerimeterStatus {
    NoMovement,
    SlightMovement,
    Movement,
}

impl From<&PerimeterStatus> for ZBytes {
    fn from(value: &PerimeterStatus) -> Self {
        <&'static str>::from(value).into()
    }
}

impl From<PerimeterStatus> for ZBytes {
    fn from(value: PerimeterStatus) -> Self {
        <&'static str>::from(value).into()
    }
}

impl FromStr for PerimeterStatus {
    type Err = zenoh::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NoMovement" => Ok(PerimeterStatus::NoMovement),
            "SlightMovement" => Ok(PerimeterStatus::SlightMovement),
            "Movement" => Ok(PerimeterStatus::Movement),
            _ => Err(Self::Err::from("Unknown Status")),
        }
    }
}

impl NewRand<PerimeterStatus> for PerimeterStatus {
    fn new_rand() -> PerimeterStatus {
        let mut rng = rand::thread_rng();
        PerimeterStatus::from(&rng.gen_range(0u32..=100u32))
    }
}

impl From<&u32> for PerimeterStatus {
    fn from(value: &u32) -> Self {
        match *value {
            x if x < 10 => PerimeterStatus::NoMovement,
            x if x > 90 => PerimeterStatus::Movement,
            _ => PerimeterStatus::SlightMovement,
        }
    }
}

impl TryFrom<&ZBytes> for PerimeterStatus {
    type Error = zenoh::Error;

    fn try_from(value: &ZBytes) -> Result<Self, Self::Error> {
        match value.try_to_string() {
            Ok(v) => v.to_string().parse::<PerimeterStatus>(),
            Err(e) => Err(e.into()),
        }
    }
}
