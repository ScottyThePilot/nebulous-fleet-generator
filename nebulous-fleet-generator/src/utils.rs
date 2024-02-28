#![allow(dead_code)]
use rand::SeedableRng;
use rand::rngs::OsRng;
use rand_xoshiro::Xoroshiro128StarStar;



pub type Random = Xoroshiro128StarStar;

#[inline]
pub fn new_rng() -> Random {
  Random::from_rng(OsRng).expect("failed to create rng")
}

#[inline]
pub fn new_rng_from_seed(seed: u64) -> Random {
  Random::seed_from_u64(seed)
}

#[inline]
pub fn new_rng_derive(rng: &mut Random) -> Random {
  Random::from_rng(rng).expect("failed to create rng")
}
