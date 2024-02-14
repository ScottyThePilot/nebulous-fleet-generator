#[macro_use]
pub mod utils;
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
  pub const fn from_u32(num: u32) -> Option<Self> {
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
