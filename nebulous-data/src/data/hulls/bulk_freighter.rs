#[cfg(feature = "rand")]
use rand::distributions::Distribution;
#[cfg(feature = "rand")]
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HullConfigBulkFreighter;

#[cfg(feature = "rand")]
impl Distribution<crate::format::HullConfig> for HullConfigBulkFreighter {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> crate::format::HullConfig {
    todo!()
  }
}
