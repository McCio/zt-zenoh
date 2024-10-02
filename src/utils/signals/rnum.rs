use crate::utils::publisher;
use crate::utils::rate_producer::RateProducer;
use rand::Rng;

pub type RandFloat = f64;

impl publisher::Computer<RandFloat> for RateProducer<'_> {
    fn compute(&self) -> RandFloat {
        let mut rng = rand::thread_rng();
        rng.gen_range(1f64..10f64)
    }
}

impl publisher::FixedIntervalPublisher<RandFloat> for RateProducer<'_> {}
