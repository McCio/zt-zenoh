use crate::utils::signals::rnum::NewRand;
use rand::Rng;
use std::str::FromStr;
use strum_macros::Display;
use zenoh::bytes::ZBytes;

#[derive(strum_macros::IntoStaticStr, Debug, Display)]
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

impl From<ZBytes> for PerimeterStatus {
    fn from(value: ZBytes) -> Self {
        let o2: String = value.deserialize().unwrap();
        o2.parse().unwrap()
    }
}

impl zenoh::bytes::Deserialize<PerimeterStatus> for zenoh::bytes::ZSerde {
    type Input<'a> = &'a ZBytes;
    type Error = zenoh::Error;

    fn deserialize(self, t: Self::Input<'_>) -> Result<PerimeterStatus, Self::Error> {
        match t.deserialize::<String>() {
            Ok(x) => x.parse(),
            Err(e) => Err(e.into()),
        }
    }
}
