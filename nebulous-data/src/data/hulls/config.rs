pub mod bulk_freighter;
pub mod container_liner;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
  V0, V1, V2
}

impl Variant {
  pub const fn select<T: Copy>(self, v0: T, v1: T, v2: T) -> T {
    match self {
      Self::V0 => v0,
      Self::V1 => v1,
      Self::V2 => v2
    }
  }

  pub const fn select_array<T>(self, array: &[T; 3]) -> &T {
    match self {
      Self::V0 => &array[0],
      Self::V1 => &array[1],
      Self::V2 => &array[2]
    }
  }

  pub const fn from_num(num: u32) -> Option<Self> {
    match num {
      0 => Some(Variant::V0),
      1 => Some(Variant::V1),
      2 => Some(Variant::V2),
      _ => None
    }
  }
}

#[cfg(feature = "rand")]
use rand::distributions::{Distribution, Standard};
#[cfg(feature = "rand")]
use rand::Rng;

#[cfg(feature = "rand")]
impl Distribution<Variant> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Variant {
    match Variant::from_num(rng.gen_range(0u32..3u32)) {
      Some(variant) => variant,
      None => unreachable!()
    }
  }
}
