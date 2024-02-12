pub mod components;
pub mod hulls;
pub mod missiles;
pub mod munitions;
pub mod utils;

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
