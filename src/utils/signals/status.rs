use crate::utils::signals::rnum::NewRand;
use rand::Rng;
use std::str::FromStr;
use strum_macros::Display;
use zenoh::bytes::ZBytes;

#[derive(strum_macros::IntoStaticStr, Debug, Display)]
pub enum WindowStatus {
    Closed,
    Opened,
}

impl From<&WindowStatus> for ZBytes {
    fn from(value: &WindowStatus) -> Self {
        <&'static str>::from(value).into()
    }
}

impl FromStr for WindowStatus {
    type Err = zenoh::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Opened" => Ok(WindowStatus::Opened),
            "Closed" => Ok(WindowStatus::Closed),
            _ => Err(Self::Err::from("Unknown Status")),
        }
    }
}

impl NewRand<WindowStatus> for WindowStatus {
    fn new_rand() -> WindowStatus {
        let mut rng = rand::thread_rng();
        WindowStatus::from(&rng.gen_range(0u32..=100u32))
    }
}

impl From<&u32> for WindowStatus {
    fn from(value: &u32) -> Self {
        match *value {
            x if x <= 50 => WindowStatus::Closed,
            _ => WindowStatus::Opened,
        }
    }
}

impl From<ZBytes> for WindowStatus {
    fn from(value: ZBytes) -> Self {
        let o2: String = value.deserialize().unwrap();
        o2.parse().unwrap()
    }
}

impl zenoh::bytes::Deserialize<WindowStatus> for zenoh::bytes::ZSerde {
    type Input<'a> = &'a ZBytes;
    type Error = zenoh::Error;

    fn deserialize(self, t: Self::Input<'_>) -> Result<WindowStatus, Self::Error> {
        match t.deserialize::<String>() {
            Ok(x) => x.parse(),
            Err(e) => Err(e.into()),
        }
    }
}
