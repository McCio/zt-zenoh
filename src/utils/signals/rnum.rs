pub type RandFloat = f64;

pub trait NewRand<T> {
    fn new_rand() -> T;
}
