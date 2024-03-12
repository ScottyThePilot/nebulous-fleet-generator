#![allow(dead_code)]
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::OsRng;
use rand_xoshiro::Xoroshiro128StarStar;



type Inner = Xoroshiro128StarStar;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Random {
  inner: Inner
}

impl Random {
  #[inline]
  pub fn new() -> Self {
    Random { inner: Inner::from_rng(OsRng).unwrap() }
  }

  #[inline]
  pub fn from_seed(seed: u64) -> Self {
    Random { inner: Inner::seed_from_u64(seed) }
  }

  #[inline]
  pub fn derive(rng: &mut impl Rng) -> Self {
    Random { inner: Inner::from_rng(rng).unwrap() }
  }
}

impl RngCore for Random {
  #[inline]
  fn next_u32(&mut self) -> u32 {
    self.inner.next_u32()
  }

  #[inline]
  fn next_u64(&mut self) -> u64 {
    self.inner.next_u64()
  }

  #[inline]
  fn fill_bytes(&mut self, dest: &mut [u8]) {
    self.inner.fill_bytes(dest)
  }

  #[inline]
  fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
    self.inner.try_fill_bytes(dest)
  }
}
