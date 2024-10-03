use rand::Rng;

pub type RandFloat = f64;
pub type RandInt = i64;
pub type RandUint = u64;

pub trait NewRand<T> {
    fn new_rand() -> T;
}

impl NewRand<RandFloat> for RandFloat {
    fn new_rand() -> RandFloat {
        let mut rng = rand::thread_rng();
        rng.gen_range(0f64..=100f64)
    }
}

impl NewRand<RandInt> for RandInt {
    fn new_rand() -> RandInt {
        let mut rng = rand::thread_rng();
        rng.gen_range(0i64..=100i64)
    }
}

impl NewRand<RandUint> for RandUint {
    fn new_rand() -> RandUint {
        let mut rng = rand::thread_rng();
        rng.gen_range(0u64..=100u64)
    }
}
