pub mod components;
pub mod hulls;
pub mod missiles;
pub mod munitions;

use std::fmt;
use std::str::FromStr;



xml::impl_deserialize_nodes_parse! {
  Faction,
  self::components::ComponentKey,
  self::hulls::HullKey,
  self::missiles::MissileBodyKey,
  self::munitions::MunitionKey
}

xml::impl_serialize_nodes_display! {
  Faction,
  self::components::ComponentKey,
  self::hulls::HullKey,
  self::missiles::MissileBodyKey,
  self::munitions::MunitionKey
}



#[derive(Debug, Error, Clone, Copy)]
#[error("invalid key")]
pub struct InvalidKey;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Faction {
  Alliance,
  Protectorate
}

impl Faction {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::Alliance => "Stock/Alliance",
      Self::Protectorate => "Stock/Protectorate"
    }
  }
}

impl FromStr for Faction {
  type Err = InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Stock/Alliance" => Ok(Self::Alliance),
      "Stock/Protectorate" => Ok(Self::Protectorate),
      _ => Err(InvalidKey)
    }
  }
}

impl fmt::Display for Faction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MissileSize {
  Size1 = 1,
  Size2 = 2,
  Size3 = 3
}

impl MissileSize {
  pub const fn from_num(num: usize) -> Option<Self> {
    match num {
      1 => Some(MissileSize::Size1),
      2 => Some(MissileSize::Size2),
      3 => Some(MissileSize::Size3),
      _ => None
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
  Up, Down, Left, Right, Fore, Aft
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Buff {
  AngularThrust,
  BurstDurationBeam,
  BurstDurationEmitters,
  CasemateElevationRate,
  CasemateTraverseRate,
  CatastrophicEventProbCellLauncher,
  CatastrophicEventProbMagazine,
  CatastrophicEventProbReactor,
  CooldownTimeBeam,
  CooldownTimeEmitters,
  CooldownTimeEnergy,
  CrewVulnerability,
  DamageMultiplierBeam,
  ElevationRate,
  FlankDamageProbability,
  IdentificationDifficulty,
  IntelligenceAccuracy,
  IntelligenceEffort,
  JammingLobAccuracy,
  LauncherReloadTime,
  LinearThrust,
  MaxRepair,
  MissileProgrammingChannels,
  MissileProgrammingSpeed,
  NoiseFiltering,
  OverheatDamageChanceEmitters,
  OverheatDamageChanceBeam,
  PositionalError,
  PowerplantEfficiency,
  RadarSignature,
  RecycleTime,
  RecycleTimeEnergy,
  ReloadTime,
  ReloadTimeEnergy,
  RepairSpeed,
  RepairTeamMoveSpeed,
  Sensitivity,
  Spread,
  TopSpeed,
  TransmitPower,
  TraverseRate,
  TurnRate,
  VelocityError
}

pub fn percentage_modifier(n: usize) -> f32 {
  (-(n as f32 / 3.5).powi(2)).exp()
}

pub fn percentage_modifier_iter() -> std::iter::Map<std::ops::RangeFrom<usize>, fn(usize) -> f32> {
  (0..).map(percentage_modifier)
}

/// Combine the list of percentages (ordered in descending order)
/// according to the stacking penalty rules.
pub fn stacking_percentage(percentages: impl IntoIterator<Item = f32>) -> f32 {
  percentages.into_iter().zip(percentage_modifier_iter()).map(|(p, m)| p * m).sum::<f32>()
}
