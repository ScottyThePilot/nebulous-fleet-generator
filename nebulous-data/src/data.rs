pub mod components;
pub mod hulls;
pub mod missiles;
pub mod munitions;

use float_ord::FloatOrd;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::fmt;
use std::str::FromStr;



xml::impl_deserialize_nodes_parse! {
  Faction,
  self::components::ComponentKey,
  self::hulls::HullKey,
  self::missiles::Maneuvers,
  self::missiles::bodies::MissileBodyKey,
  self::missiles::seekers::SeekerMode,
  self::munitions::MunitionKey
}

xml::impl_serialize_nodes_display! {
  Faction,
  self::components::ComponentKey,
  self::hulls::HullKey,
  self::missiles::Maneuvers,
  self::missiles::bodies::MissileBodyKey,
  self::missiles::seekers::SeekerMode,
  self::munitions::MunitionKey
}



#[derive(Debug, Error, Clone, Copy)]
pub enum InvalidKey {
  #[error("invalid faction key")]
  Faction,
  #[error("invalid component key")]
  Component,
  #[error("invalid hull key")]
  Hull,
  #[error("invalid seeker key")]
  Seeker,
  #[error("invalid missile body key")]
  MissileBody,
  #[error("invalid missile component key")]
  MissileComponent,
  #[error("invalid missile size mask")]
  MissileSizeMask,
  #[error("invalid missile role")]
  MissileRole,
  #[error("invalid target type")]
  AntiRadiationTargetType,
  #[error("invalid target type")]
  DefensiveTargetType,
  #[error("invalid maneuvers")]
  Maneuvers,
  #[error("invalid munition key")]
  Munition,
  #[error("invalid ordering")]
  Ordering
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Faction {
  #[cfg_attr(feature = "serde", serde(alias = "ans"))]
  Alliance,
  #[cfg_attr(feature = "serde", serde(alias = "osp"))]
  Protectorate
}

impl Faction {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::Alliance => "Stock/Alliance",
      Self::Protectorate => "Stock/Protectorate"
    }
  }

  pub const fn name(self) -> &'static str {
    match self {
      Self::Alliance => "Shelter Alliance",
      Self::Protectorate => "Outlying Systems Protectorate"
    }
  }
}

impl FromStr for Faction {
  type Err = InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Stock/Alliance" => Ok(Self::Alliance),
      "Stock/Protectorate" => Ok(Self::Protectorate),
      _ => Err(InvalidKey::Faction)
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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MissileSize {
  #[cfg_attr(feature = "serde", serde(alias = "1"))]
  Size1 = 1,
  #[cfg_attr(feature = "serde", serde(alias = "2"))]
  Size2 = 2,
  #[cfg_attr(feature = "serde", serde(alias = "3"))]
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

impl FromStr for MissileSize {
  type Err = ParseMissileSizeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "size1" | "1" => Ok(MissileSize::Size1),
      "size2" | "2" => Ok(MissileSize::Size2),
      "size3" | "3" => Ok(MissileSize::Size3),
      _ => Err(ParseMissileSizeError)
    }
  }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Default)]
#[error("failed to parse missile size")]
pub struct ParseMissileSizeError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Direction {
  Up, Down, Left, Right, Fore, Aft
}

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Buffs {
  pub angular_thrust: f32,
  pub burst_duration_beam: f32,
  pub burst_duration_emitters: f32,
  pub casemate_elevation_rate: f32,
  pub casemate_traverse_rate: f32,
  pub catastrophic_event_prob_cell_launcher: f32,
  pub catastrophic_event_prob_magazine: f32,
  pub catastrophic_event_prob_reactor: f32,
  pub cooldown_time_beam: f32,
  pub cooldown_time_emitters: f32,
  pub cooldown_time_energy: f32,
  pub crew_vulnerability: f32,
  pub damage_multiplier_beam: f32,
  pub elevation_rate: f32,
  pub flank_damage_probability: f32,
  pub identification_difficulty: f32,
  pub intelligence_accuracy: f32,
  pub intelligence_effort: f32,
  pub jamming_lob_accuracy: f32,
  pub launcher_reload_time: f32,
  pub linear_thrust: f32,
  pub max_repair: f32,
  pub missile_programming_channels: i32,
  pub missile_programming_speed: f32,
  pub noise_filtering: f32,
  pub overheat_damage_chance_emitters: f32,
  pub overheat_damage_chance_beam: f32,
  pub positional_error: f32,
  pub powerplant_efficiency: f32,
  pub radar_signature: f32,
  pub recycle_time: f32,
  pub recycle_time_energy: f32,
  pub reload_time: f32,
  pub reload_time_energy: f32,
  pub repair_speed: f32,
  pub repair_team_move_speed: f32,
  pub sensitivity: f32,
  pub spread: f32,
  pub top_speed: f32,
  pub transmit_power: f32,
  pub traverse_rate: f32,
  pub turn_rate: f32,
  pub velocity_error: f32
}

impl FromIterator<Buff> for Buffs {
  fn from_iter<T: IntoIterator<Item = Buff>>(buffs: T) -> Self {
    type Float = Reverse<FloatOrd<f32>>;

    fn stacking_percentage_sort(percentages: BinaryHeap<Float>) -> f32 {
      stacking_percentage(percentages.into_sorted_vec().into_iter().map(|Reverse(FloatOrd(value))| value))
    }

    let mut buffs_list_float = <HashMap<BuffKey, BinaryHeap<Float>>>::new();
    let mut buffs_list_integer = <HashMap<BuffKey, Vec<i32>>>::new();
    for buff in buffs {
      let (buff_key, buff_value) = buff.to_key_value();
      match buff_value {
        BuffValue::Float(value) => {
          let value = Reverse(FloatOrd(value));
          buffs_list_float.entry(buff_key).or_default().push(value);
        },
        BuffValue::Integer(value) => {
          buffs_list_integer.entry(buff_key).or_default().push(value);
        },
      };
    };

    let buffs_float = buffs_list_float.into_iter()
      .map(|(key, value)| (key, stacking_percentage_sort(value)))
      .collect::<HashMap<BuffKey, f32>>();
    let buffs_integer = buffs_list_integer.into_iter()
      .map(|(key, value)| (key, value.into_iter().sum()))
      .collect::<HashMap<BuffKey, i32>>();

    let get_float = |key: BuffKey| buffs_float.get(&key).copied().unwrap_or_default();
    let get_integer = |key: BuffKey| buffs_integer.get(&key).copied().unwrap_or_default();

    Buffs {
      angular_thrust: get_float(BuffKey::AngularThrust),
      burst_duration_beam: get_float(BuffKey::BurstDurationBeam),
      burst_duration_emitters: get_float(BuffKey::BurstDurationEmitters),
      casemate_elevation_rate: get_float(BuffKey::CasemateElevationRate),
      casemate_traverse_rate: get_float(BuffKey::CasemateTraverseRate),
      catastrophic_event_prob_cell_launcher: get_float(BuffKey::CatastrophicEventProbCellLauncher),
      catastrophic_event_prob_magazine: get_float(BuffKey::CatastrophicEventProbMagazine),
      catastrophic_event_prob_reactor: get_float(BuffKey::CatastrophicEventProbReactor),
      cooldown_time_beam: get_float(BuffKey::CooldownTimeBeam),
      cooldown_time_emitters: get_float(BuffKey::CooldownTimeEmitters),
      cooldown_time_energy: get_float(BuffKey::CooldownTimeEnergy),
      crew_vulnerability: get_float(BuffKey::CrewVulnerability),
      damage_multiplier_beam: get_float(BuffKey::DamageMultiplierBeam),
      elevation_rate: get_float(BuffKey::ElevationRate),
      flank_damage_probability: get_float(BuffKey::FlankDamageProbability),
      identification_difficulty: get_float(BuffKey::IdentificationDifficulty),
      intelligence_accuracy: get_float(BuffKey::IntelligenceAccuracy),
      intelligence_effort: get_float(BuffKey::IntelligenceEffort),
      jamming_lob_accuracy: get_float(BuffKey::JammingLobAccuracy),
      launcher_reload_time: get_float(BuffKey::LauncherReloadTime),
      linear_thrust: get_float(BuffKey::LinearThrust),
      max_repair: get_float(BuffKey::MaxRepair),
      missile_programming_channels: get_integer(BuffKey::MissileProgrammingChannels),
      missile_programming_speed: get_float(BuffKey::MissileProgrammingSpeed),
      noise_filtering: get_float(BuffKey::NoiseFiltering),
      overheat_damage_chance_emitters: get_float(BuffKey::OverheatDamageChanceEmitters),
      overheat_damage_chance_beam: get_float(BuffKey::OverheatDamageChanceBeam),
      positional_error: get_float(BuffKey::PositionalError),
      powerplant_efficiency: get_float(BuffKey::PowerplantEfficiency),
      radar_signature: get_float(BuffKey::RadarSignature),
      recycle_time: get_float(BuffKey::RecycleTime),
      recycle_time_energy: get_float(BuffKey::RecycleTimeEnergy),
      reload_time: get_float(BuffKey::ReloadTime),
      reload_time_energy: get_float(BuffKey::ReloadTimeEnergy),
      repair_speed: get_float(BuffKey::RepairSpeed),
      repair_team_move_speed: get_float(BuffKey::RepairTeamMoveSpeed),
      sensitivity: get_float(BuffKey::Sensitivity),
      spread: get_float(BuffKey::Spread),
      top_speed: get_float(BuffKey::TopSpeed),
      transmit_power: get_float(BuffKey::TransmitPower),
      traverse_rate: get_float(BuffKey::TraverseRate),
      turn_rate: get_float(BuffKey::TurnRate),
      velocity_error: get_float(BuffKey::VelocityError)
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Buff {
  AngularThrust(f32),
  BurstDurationBeam(f32),
  BurstDurationEmitters(f32),
  CasemateElevationRate(f32),
  CasemateTraverseRate(f32),
  CatastrophicEventProbCellLauncher(f32),
  CatastrophicEventProbMagazine(f32),
  CatastrophicEventProbReactor(f32),
  CooldownTimeBeam(f32),
  CooldownTimeEmitters(f32),
  CooldownTimeEnergy(f32),
  CrewVulnerability(f32),
  DamageMultiplierBeam(f32),
  ElevationRate(f32),
  FlankDamageProbability(f32),
  IdentificationDifficulty(f32),
  IntelligenceAccuracy(f32),
  IntelligenceEffort(f32),
  JammingLobAccuracy(f32),
  LauncherReloadTime(f32),
  LinearThrust(f32),
  MaxRepair(f32),
  MissileProgrammingChannels(i32),
  MissileProgrammingSpeed(f32),
  NoiseFiltering(f32),
  OverheatDamageChanceEmitters(f32),
  OverheatDamageChanceBeam(f32),
  PositionalError(f32),
  PowerplantEfficiency(f32),
  RadarSignature(f32),
  RecycleTime(f32),
  RecycleTimeEnergy(f32),
  ReloadTime(f32),
  ReloadTimeEnergy(f32),
  RepairSpeed(f32),
  RepairTeamMoveSpeed(f32),
  Sensitivity(f32),
  Spread(f32),
  TopSpeed(f32),
  TransmitPower(f32),
  TraverseRate(f32),
  TurnRate(f32),
  VelocityError(f32)
}

impl Buff {
  pub const fn to_key_value(self) -> (BuffKey, BuffValue) {
    match self {
      Self::AngularThrust(value) => (BuffKey::AngularThrust, BuffValue::Float(value)),
      Self::BurstDurationBeam(value) => (BuffKey::BurstDurationBeam, BuffValue::Float(value)),
      Self::BurstDurationEmitters(value) => (BuffKey::BurstDurationEmitters, BuffValue::Float(value)),
      Self::CasemateElevationRate(value) => (BuffKey::CasemateElevationRate, BuffValue::Float(value)),
      Self::CasemateTraverseRate(value) => (BuffKey::CasemateTraverseRate, BuffValue::Float(value)),
      Self::CatastrophicEventProbCellLauncher(value) => (BuffKey::CatastrophicEventProbCellLauncher, BuffValue::Float(value)),
      Self::CatastrophicEventProbMagazine(value) => (BuffKey::CatastrophicEventProbMagazine, BuffValue::Float(value)),
      Self::CatastrophicEventProbReactor(value) => (BuffKey::CatastrophicEventProbReactor, BuffValue::Float(value)),
      Self::CooldownTimeBeam(value) => (BuffKey::CooldownTimeBeam, BuffValue::Float(value)),
      Self::CooldownTimeEmitters(value) => (BuffKey::CooldownTimeEmitters, BuffValue::Float(value)),
      Self::CooldownTimeEnergy(value) => (BuffKey::CooldownTimeEnergy, BuffValue::Float(value)),
      Self::CrewVulnerability(value) => (BuffKey::CrewVulnerability, BuffValue::Float(value)),
      Self::DamageMultiplierBeam(value) => (BuffKey::DamageMultiplierBeam, BuffValue::Float(value)),
      Self::ElevationRate(value) => (BuffKey::ElevationRate, BuffValue::Float(value)),
      Self::FlankDamageProbability(value) => (BuffKey::FlankDamageProbability, BuffValue::Float(value)),
      Self::IdentificationDifficulty(value) => (BuffKey::IdentificationDifficulty, BuffValue::Float(value)),
      Self::IntelligenceAccuracy(value) => (BuffKey::IntelligenceAccuracy, BuffValue::Float(value)),
      Self::IntelligenceEffort(value) => (BuffKey::IntelligenceEffort, BuffValue::Float(value)),
      Self::JammingLobAccuracy(value) => (BuffKey::JammingLobAccuracy, BuffValue::Float(value)),
      Self::LauncherReloadTime(value) => (BuffKey::LauncherReloadTime, BuffValue::Float(value)),
      Self::LinearThrust(value) => (BuffKey::LinearThrust, BuffValue::Float(value)),
      Self::MaxRepair(value) => (BuffKey::MaxRepair, BuffValue::Float(value)),
      Self::MissileProgrammingChannels(value) => (BuffKey::MissileProgrammingChannels, BuffValue::Integer(value)),
      Self::MissileProgrammingSpeed(value) => (BuffKey::MissileProgrammingSpeed, BuffValue::Float(value)),
      Self::NoiseFiltering(value) => (BuffKey::NoiseFiltering, BuffValue::Float(value)),
      Self::OverheatDamageChanceEmitters(value) => (BuffKey::OverheatDamageChanceEmitters, BuffValue::Float(value)),
      Self::OverheatDamageChanceBeam(value) => (BuffKey::OverheatDamageChanceBeam, BuffValue::Float(value)),
      Self::PositionalError(value) => (BuffKey::PositionalError, BuffValue::Float(value)),
      Self::PowerplantEfficiency(value) => (BuffKey::PowerplantEfficiency, BuffValue::Float(value)),
      Self::RadarSignature(value) => (BuffKey::RadarSignature, BuffValue::Float(value)),
      Self::RecycleTime(value) => (BuffKey::RecycleTime, BuffValue::Float(value)),
      Self::RecycleTimeEnergy(value) => (BuffKey::RecycleTimeEnergy, BuffValue::Float(value)),
      Self::ReloadTime(value) => (BuffKey::ReloadTime, BuffValue::Float(value)),
      Self::ReloadTimeEnergy(value) => (BuffKey::ReloadTimeEnergy, BuffValue::Float(value)),
      Self::RepairSpeed(value) => (BuffKey::RepairSpeed, BuffValue::Float(value)),
      Self::RepairTeamMoveSpeed(value) => (BuffKey::RepairTeamMoveSpeed, BuffValue::Float(value)),
      Self::Sensitivity(value) => (BuffKey::Sensitivity, BuffValue::Float(value)),
      Self::Spread(value) => (BuffKey::Spread, BuffValue::Float(value)),
      Self::TopSpeed(value) => (BuffKey::TopSpeed, BuffValue::Float(value)),
      Self::TransmitPower(value) => (BuffKey::TransmitPower, BuffValue::Float(value)),
      Self::TraverseRate(value) => (BuffKey::TraverseRate, BuffValue::Float(value)),
      Self::TurnRate(value) => (BuffKey::TurnRate, BuffValue::Float(value)),
      Self::VelocityError(value) => (BuffKey::VelocityError, BuffValue::Float(value))
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum BuffKey {
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum BuffValue {
  Float(f32),
  Integer(i32)
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
