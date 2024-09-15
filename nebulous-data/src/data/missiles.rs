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
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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

impl AuxiliaryKey {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::ColdGasBottle => "Stock/Cold Gas Bottle",
      Self::DecoyLauncher => "Stock/Decoy Launcher",
      Self::ClusterDecoyLauncher => "Stock/Cluster Decoy Launcher",
      Self::FastStartupModule => "Stock/Fast Startup Module",
      Self::HardenedSkin => "Stock/Hardened Skin",
      Self::RadarAbsorbentCoating => "Stock/Radar Absorbent Coating",
      Self::SelfScreeningJammer => "Stock/Self-Screening Jammer",
      Self::BoostedSelfScreeningJammer => "Stock/Boosted Self-Screening Jammer"
    }
  }

  pub const fn cost(self) -> usize {
    todo!()
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum AvionicsKey {
  DirectGuidance,
  CruiseGuidance
}

impl AvionicsKey {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::DirectGuidance => "Stock/Direct Guidance",
      Self::CruiseGuidance => "Stock/Cruise Guidance"
    }
  }

  pub const fn cost(self) -> usize {
    todo!()
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum WarheadKey {
  #[cfg_attr(feature = "serde", serde(rename = "he_impact"))]
  HEImpact,
  #[cfg_attr(feature = "serde", serde(rename = "he_kinetic_penetrator"))]
  HEKineticPenetrator,
  #[cfg_attr(feature = "serde", serde(rename = "blast_fragmentation"))]
  BlastFragmentation,
  #[cfg_attr(feature = "serde", serde(rename = "blast_fragmentation_el"))]
  BlastFragmentationEL
}

impl WarheadKey {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::HEImpact => "Stock/HE Impact",
      Self::HEKineticPenetrator => "Stock/HE Kinetic Penetrator",
      Self::BlastFragmentation => "Stock/Blast Fragmentation",
      Self::BlastFragmentationEL => "Stock/Blast Fragmentation EL"
    }
  }

  pub const fn cost(self) -> usize {
    todo!()
  }
}

impl fmt::Display for Maneuvers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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
