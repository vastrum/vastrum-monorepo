#[cfg(feature = "madsim_compliant")]
mod inner {
    use madsim::rand::{Rng, RngCore, seq::SliceRandom};

    pub fn fill_bytes(buf: &mut [u8]) {
        madsim::rand::thread_rng().fill_bytes(buf);
    }
    pub fn random_range(range: std::ops::RangeInclusive<u64>) -> u64 {
        madsim::rand::thread_rng().gen_range(range)
    }
    pub fn choose<T>(slice: &[T]) -> Option<&T> {
        slice.choose(&mut madsim::rand::thread_rng())
    }
}

#[cfg(not(feature = "madsim_compliant"))]
mod inner {
    use rand::seq::IndexedRandom;

    pub fn fill_bytes(buf: &mut [u8]) {
        rand::fill(buf);
    }
    pub fn random_range(range: std::ops::RangeInclusive<u64>) -> u64 {
        rand::random_range(range)
    }
    pub fn choose<T>(slice: &[T]) -> Option<&T> {
        slice.choose(&mut rand::rng())
    }
}

pub use inner::{choose, fill_bytes, random_range};
