pub mod components;
pub mod hulls;
pub mod missiles;
pub mod munitions;

use std::str::FromStr;



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Faction {
  Alliance,
  Protectorate
}

impl FromStr for Faction {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Stock/Alliance" => Ok(Self::Alliance),
      "Stock/Protectorate" => Ok(Self::Protectorate),
      _ => Err(())
    }
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
