pub mod bulk_freighter;
pub mod container_liner;

use crate::format::HullConfig;

#[cfg(feature = "rand")]
use rand::distributions::{Distribution, Standard};
#[cfg(feature = "rand")]
use rand::Rng;



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HullConfigTemplate {
  BulkFreighter(self::bulk_freighter::HullConfigBulkFreighter),
  ContainerLiner(self::container_liner::HullConfigContainerLiner)
}

#[cfg(feature = "rand")]
impl Distribution<crate::format::HullConfig> for HullConfigTemplate {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> crate::format::HullConfig {
    match self {
      Self::BulkFreighter(config) => config.sample(rng),
      Self::ContainerLiner(config) => config.sample(rng)
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HullConfigTemplateFull {
  BulkFreighter(self::bulk_freighter::HullConfigBulkFreighterFull),
  ContainerLiner(self::container_liner::HullConfigContainerLinerFull)
}

impl HullConfigTemplateFull {
  pub const BULK_FREIGHTER: Self = Self::BulkFreighter(self::bulk_freighter::HullConfigBulkFreighterFull);
  pub const CONTAINER_LINER: Self = Self::ContainerLiner(self::container_liner::HullConfigContainerLinerFull);

  pub const fn with_variants(self, variants: [Variant; 3]) -> HullConfigTemplate {
    match self {
      Self::BulkFreighter(config) => HullConfigTemplate::BulkFreighter(config.with_variants(variants)),
      Self::ContainerLiner(config) => HullConfigTemplate::ContainerLiner(config.with_variants(variants))
    }
  }

  pub fn get_variants(self, hull_config: &HullConfig) -> Option<[Variant; 3]> {
    match self {
      Self::BulkFreighter(..) => self::bulk_freighter::get_variants(hull_config),
      Self::ContainerLiner(..) => self::container_liner::get_variants(hull_config)
    }
  }
}

#[cfg(feature = "rand")]
impl Distribution<crate::format::HullConfig> for HullConfigTemplateFull {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> crate::format::HullConfig {
    match self {
      Self::BulkFreighter(config) => config.sample(rng),
      Self::ContainerLiner(config) => config.sample(rng)
    }
  }
}



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

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Variant {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    <u32 as serde::Deserialize>::deserialize(deserializer).and_then(|num| {
      Self::from_num(num).ok_or(serde::de::Error::invalid_value(
        serde::de::Unexpected::Unsigned(num as u64),
        &"an integer between 1 and 3"
      ))
    })
  }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Variant {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    <u8 as serde::Serialize>::serialize(&(*self as u8), serializer)
  }
}

#[cfg(feature = "rand")]
impl Distribution<Variant> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Variant {
    match Variant::from_num(rng.gen_range(0u32..3u32)) {
      Some(variant) => variant,
      None => unreachable!()
    }
  }
}
