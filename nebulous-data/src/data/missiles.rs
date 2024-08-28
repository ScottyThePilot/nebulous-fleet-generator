pub mod bodies;
pub mod engines;
pub mod seekers;

use bytemuck::Contiguous;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum AuxiliaryKey {
  ColdGasBottle,
  DecoyLauncher,
  ClusterDecoyLauncher,
  FastStartupModule,
  HardenedSkin,
  RadarAbsorbentCoating,
  SelfScreeningJammer,
  BoostedSelfScreeningJammer
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum AvionicsKey {
  DirectGuidance,
  CruiseGuidance
}

impl AvionicsKey {
  pub const fn save_key(self) -> &'static str {
    match self {
      AvionicsKey::DirectGuidance => "Stock/Direct Guidance",
      AvionicsKey::CruiseGuidance => "Stock/Cruise Guidance"
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum WarheadKey {
  HEImpact,
  HEKineticPenetrator,
  BlastFragmentation,
  BlastFragmentationEL
}

impl fmt::Display for Maneuvers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Maneuvers {
  #[default] None, Weave, Corkscrew
}

impl Maneuvers {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::None => "None",
      Self::Weave => "Weave",
      Self::Corkscrew => "Corkscrew"
    }
  }
}

impl FromStr for Maneuvers {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "None" => Ok(Self::None),
      "Weave" => Ok(Self::Weave),
      "Corkscrew" => Ok(Self::Corkscrew),
      _ => Err(crate::data::InvalidKey::Maneuvers)
    }
  }
}
