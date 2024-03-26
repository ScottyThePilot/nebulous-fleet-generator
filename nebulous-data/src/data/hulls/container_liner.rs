#[cfg(feature = "rand")]
use rand::distributions::Distribution;
#[cfg(feature = "rand")]
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HullConfigContainerLiner;

#[cfg(feature = "rand")]
impl Distribution<crate::format::HullConfig> for HullConfigContainerLiner {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> crate::format::HullConfig {
    todo!()
  }
}
