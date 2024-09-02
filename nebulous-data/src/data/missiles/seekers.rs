use crate::data::Faction;
use crate::utils::ContiguousExt;

use bytemuck::Contiguous;
use itertools::Itertools;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::num::NonZeroUsize as zsize;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, Not};
use std::str::FromStr;
use std::sync::OnceLock;



/// A seeker that can be chosen in game.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SeekerKey {
  Command,
  FixedActiveRadar,
  SteerableActiveRadar,
  SteerableExtendedActiveRadar,
  FixedSemiActiveRadar,
  FixedAntiRadiation,
  FixedHomeOnJam,
  ElectroOptical,
  WakeHoming
}

impl SeekerKey {
  pub const fn cost(self) -> SeekerCost {
    match self {
      Self::Command => SeekerCost::new(3.50, 3.00),
      Self::FixedActiveRadar => SeekerCost::new(1.00, 0.25),
      Self::SteerableActiveRadar => SeekerCost::new(1.50, 0.50),
      Self::SteerableExtendedActiveRadar => SeekerCost::new(3.00, 1.00),
      Self::FixedSemiActiveRadar => SeekerCost::new(0.00, 0.50),
      Self::FixedAntiRadiation => SeekerCost::new(2.00, 2.00),
      Self::FixedHomeOnJam => SeekerCost::new(0.50, 0.50),
      Self::ElectroOptical => SeekerCost::new(8.00, 5.00),
      Self::WakeHoming => SeekerCost::new(0.25, 0.50)
    }
  }

  /// A metric used to inform the generator on a seeker's ability
  /// to correctly discriminate, track, and guide onto targets reliably.
  ///
  /// Range: `[0, 5]`
  pub const fn base_guidance_quality(self) -> f32 {
    match self {
      // Command Receivers are considered the highest guidance quality there is,
      // as they will only ever target tracks designated by the firing ship.
      Self::Command => 5.0,
      Self::FixedActiveRadar => 3.0,
      Self::SteerableActiveRadar => 3.0,
      Self::SteerableExtendedActiveRadar => 4.0,
      Self::FixedSemiActiveRadar => 3.5,
      // Anti-Radiation scores low due to being unable to discern distance.
      Self::FixedAntiRadiation => 2.0,
      // Home-On-Jam is far more restrictive than Anti-Radiation, so it scores lower.
      Self::FixedHomeOnJam => 1.5,
      // Electro-Optical Seekers, when provided with intel, are second
      // only to Command Receivers at discriminating targets correctly.
      Self::ElectroOptical => 4.5,
      Self::WakeHoming => 1.0
    }
  }

  pub const fn seeker_kind(self) -> SeekerKind {
    match self {
      Self::Command => SeekerKind::Command,
      Self::FixedActiveRadar => SeekerKind::ActiveRadar,
      Self::SteerableActiveRadar => SeekerKind::ActiveRadar,
      Self::SteerableExtendedActiveRadar => SeekerKind::ActiveRadar,
      Self::FixedSemiActiveRadar => SeekerKind::SemiActiveRadar,
      Self::FixedAntiRadiation => SeekerKind::AntiRadiation,
      Self::FixedHomeOnJam => SeekerKind::HomeOnJam,
      Self::ElectroOptical => SeekerKind::ElectroOptical,
      Self::WakeHoming => SeekerKind::WakeHoming
    }
  }
}

impl fmt::Display for SeekerKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.seeker_kind(), f)
  }
}

/// Describes a seeker's method of operation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SeekerKind {
  Command,
  ActiveRadar,
  SemiActiveRadar,
  AntiRadiation,
  HomeOnJam,
  ElectroOptical,
  WakeHoming
}

impl SeekerKind {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::Command => "CMD",
      Self::ActiveRadar => "ACT(RADAR)",
      Self::SemiActiveRadar => "SAH(RADAR)",
      Self::AntiRadiation => "ARAD(RADAR)",
      Self::HomeOnJam => "HOJ(RADAR)",
      Self::ElectroOptical => "PSV(EO)",
      Self::WakeHoming => "PSV(WAKE)"
    }
  }

  pub fn min_cost(self) -> SeekerCost {
    self.iter_seeker_keys()
      .map(SeekerKey::cost)
      .reduce(SeekerCost::min)
      .unwrap()
  }

  pub fn max_cost(self) -> SeekerCost {
    self.iter_seeker_keys()
      .map(SeekerKey::cost)
      .reduce(SeekerCost::max)
      .unwrap()
  }

  pub fn average_base_guidance_quality(self) -> f32 {
    let seeker_keys = self.seeker_keys();
    seeker_keys.iter().copied()
      .map(SeekerKey::base_guidance_quality)
      .reduce(|a, b| a + b)
      .unwrap() / seeker_keys.len() as f32
  }

  pub const fn can_measure_distance(self) -> bool {
    match self {
      Self::Command => true,
      Self::ActiveRadar => true,
      Self::SemiActiveRadar => false,
      Self::AntiRadiation => false,
      Self::HomeOnJam => false,
      Self::ElectroOptical => true,
      Self::WakeHoming => false
    }
  }

  pub const fn can_measure_velocity(self) -> bool {
    match self {
      Self::Command => true,
      Self::ActiveRadar => true,
      Self::SemiActiveRadar => true,
      Self::AntiRadiation => false,
      Self::HomeOnJam => false,
      Self::ElectroOptical => true,
      Self::WakeHoming => false
    }
  }

  pub const fn seeker_keys(self) -> &'static [SeekerKey] {
    match self {
      Self::Command => &[SeekerKey::Command],
      Self::ActiveRadar => &[
        SeekerKey::FixedActiveRadar,
        SeekerKey::SteerableActiveRadar,
        SeekerKey::SteerableExtendedActiveRadar
      ],
      Self::SemiActiveRadar => &[SeekerKey::FixedSemiActiveRadar],
      Self::AntiRadiation => &[SeekerKey::FixedAntiRadiation],
      Self::HomeOnJam => &[SeekerKey::FixedHomeOnJam],
      Self::ElectroOptical => &[SeekerKey::ElectroOptical],
      Self::WakeHoming => &[SeekerKey::WakeHoming]
    }
  }

  /// The mask of countermeasures that defeat this seeker.
  pub const fn defeating_countermeasures_mask(self) -> CountermeasuresMask {
    match self {
      Self::Command => {
        CountermeasuresMask { comms_jamming: true, ..CountermeasuresMask::NONE }
      },
      Self::ActiveRadar | Self::SemiActiveRadar => {
        CountermeasuresMask { radar_jamming: true, chaff_decoy: true, active_decoy: true, ..CountermeasuresMask::NONE }
      },
      Self::AntiRadiation => {
        CountermeasuresMask { active_decoy: true, cut_radar: true, ..CountermeasuresMask::NONE }
      },
      Self::HomeOnJam => {
        CountermeasuresMask::NONE
      },
      Self::ElectroOptical => {
        CountermeasuresMask { laser_dazzler: true, ..CountermeasuresMask::NONE }
      },
      Self::WakeHoming => {
        CountermeasuresMask { flare_decoy: true, cut_engines: true, ..CountermeasuresMask::NONE }
      }
    }
  }

  /// Whether or not this seeker (alone) is defeated by this single countermeasure.
  pub const fn is_defeated_by_one(self, cm: Countermeasure) -> bool {
    *self.defeating_countermeasures_mask().get(cm)
  }

  /// Whether or not this seeker (alone) is defeated by this set of countermeasures.
  pub const fn is_defeated_by(self, countermeasures: CountermeasuresMask) -> bool {
    self.get_defeating_countermeasures(countermeasures).is_some()
  }

  /// Whether or not this seeker (alone) is defeated by this set of countermeasures, and the subset of those countermeasures that defeated it.
  pub const fn get_defeating_countermeasures(self, countermeasures: CountermeasuresMask) -> Option<CountermeasuresMask> {
    // If any radar jamming is present, and the seeker is Anti-Radiation or Home-On-Jam, the seeker finds the target
    if matches!(self, Self::AntiRadiation | Self::HomeOnJam) && *countermeasures.get(Countermeasure::RadarJamming) { return None };

    //let countermeasures = countermeasures.map_with_tag(|cm, state| state && self.is_defeated_by_one(cm));
    let countermeasures = self.defeating_countermeasures_mask().and(countermeasures);
    if countermeasures.any() { Some(countermeasures) } else { None }
  }

  pub fn iter_seeker_keys(self) -> std::iter::Copied<std::slice::Iter<'static, SeekerKey>> {
    self.seeker_keys().iter().copied()
  }
}

impl From<SeekerKey> for SeekerKind {
  fn from(value: SeekerKey) -> Self {
    value.seeker_kind()
  }
}

impl fmt::Display for SeekerKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SeekerMode {
  Targeting,
  Validation
}

impl SeekerMode {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::Targeting => "Targeting",
      Self::Validation => "Validation"
    }
  }
}

impl FromStr for SeekerMode {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Targeting" => Ok(Self::Targeting),
      "Validation" => Ok(Self::Validation),
      _ => Err(crate::data::InvalidKey::Seeker)
    }
  }
}

impl fmt::Display for SeekerMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SeekerCost {
  pub targeting: f32,
  pub validation: f32
}

impl SeekerCost {
  pub const fn new(targeting: f32, validation: f32) -> Self {
    SeekerCost { targeting, validation }
  }

  fn min(self, other: Self) -> Self {
    SeekerCost {
      targeting: self.targeting.min(other.targeting),
      validation: self.validation.min(other.validation)
    }
  }

  fn max(self, other: Self) -> Self {
    SeekerCost {
      targeting: self.targeting.max(other.targeting),
      validation: self.validation.max(other.validation)
    }
  }
}

impl Index<SeekerMode> for SeekerCost {
  type Output = f32;

  fn index(&self, mode: SeekerMode) -> &Self::Output {
    match mode {
      SeekerMode::Targeting => &self.targeting,
      SeekerMode::Validation => &self.validation
    }
  }
}

pub type SeekerStrategyFull = SeekerStrategy<SeekerKey>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SeekerStrategy<S: Copy = SeekerKind> {
  pub primary: S,
  pub secondaries: Box<[(S, SeekerMode)]>
}

impl<S: Copy> SeekerStrategy<S> {
  pub fn new(primary: S, secondaries: impl Into<Box<[(S, SeekerMode)]>>) -> Self {
    SeekerStrategy { primary, secondaries: secondaries.into() }
  }

  pub fn new_single(primary: S) -> Self {
    SeekerStrategy { primary, secondaries: Box::new([]) }
  }

  pub const fn len(&self) -> zsize {
    zsize!(self.secondaries.len().wrapping_add(1))
  }

  pub fn iter(&self) -> SeekerStrategyIter<S> {
    std::iter::once((self.primary, SeekerMode::Targeting)).chain(self.secondaries.iter().copied())
  }
}

impl<S: Copy + Into<SeekerKind>> SeekerStrategy<S> {
  pub fn is_reasonable(&self) -> bool {
    let mut buffer = Vec::new();
    let mut iter = self.iter().peekable();
    let mut targeting_seekers = Vec::new();
    // Loop through every group of seekers (a targeting seeker and any attached validating seekers)
    while let Some((seeker, validating_seekers)) = next_seeker_layer(&mut iter, &mut buffer) {
      let previous_seeker = targeting_seekers.last().copied();
      let is_primary = previous_seeker.is_none();
      targeting_seekers.push(seeker);

      // Wake-Homing seekers cannot have backups because those backups would always be better as primaries
      if matches!(self.primary.into(), SeekerKind::WakeHoming) && !is_primary { return false };
      // Command recievers cannot be backups because they would be better as primaries
      if matches!(seeker, SeekerKind::Command) && !is_primary { return false };
      // Command recievers, Wake-Homing seekers, and Home-On-Jam cannot have validators
      if matches!(seeker, SeekerKind::Command | SeekerKind::WakeHoming | SeekerKind::HomeOnJam) && !validating_seekers.is_empty() { return false };
      // Home-On-Jam seekers must only be used as either primaries or backups for Active Radar or Semi-Active Radar seekers
      if matches!(seeker, SeekerKind::HomeOnJam) && !matches!(previous_seeker, Some(SeekerKind::ActiveRadar | SeekerKind::SemiActiveRadar) | None) { return false };
      // Anti-Radiation seekers cannot have validators, except for Wake Homing validators
      if matches!(seeker, SeekerKind::AntiRadiation) && !(validating_seekers.is_empty() || validating_seekers == [SeekerKind::WakeHoming]) { return false };
      // Electro-Optical seekers cannot have validators, except for Command reciever validators
      if matches!(seeker, SeekerKind::ElectroOptical) && !(validating_seekers.is_empty() || validating_seekers == [SeekerKind::Command]) { return false };
      // Active Radar, Semi-Active Radar, and Home-On-Jam are not allowed to be validators
      if validating_seekers.contains(&SeekerKind::ActiveRadar) { return false };
      if validating_seekers.contains(&SeekerKind::SemiActiveRadar) { return false };
      if validating_seekers.contains(&SeekerKind::HomeOnJam) { return false };
      // The targeting seeker should not have an attached validating seeker of the same type
      if validating_seekers.contains(&seeker) { return false };
      // Filter out redundant combinations (duplicate validating seekers)
      if has_redundancy(validating_seekers) { return false };
    };

    // Home-On-Jam all by itself is very bad
    if targeting_seekers == [SeekerKind::HomeOnJam] { return false };

    // Filter out redundant combinations (duplicate targeting seekers)
    if has_redundancy(&targeting_seekers) { return false };

    true
  }

  pub fn is_defeated_by(&self, countermeasures: CountermeasuresMask) -> bool {
    let jamming = countermeasures.mask_category(CountermeasureCategory::Jamming);

    let mut buffer = Vec::new();
    let mut iter = self.iter().peekable();
    // Loop through every group of seekers (a targeting seeker and any attached validating seekers)
    while let Some((seeker, validating_seekers)) = next_seeker_layer(&mut iter, &mut buffer) {
      if matches!(seeker, SeekerKind::HomeOnJam) {
        // If there is no radar jamming, the Home-On-Jam seeker does not see anything, fall back to the next seeker
        if countermeasures.radar_jamming { return false } else { continue };
      } else if let Some(defeating_countermeasures) = seeker.get_defeating_countermeasures(countermeasures) {
        let defeated_by_jamming = defeating_countermeasures.mask_category(CountermeasureCategory::Jamming).any();
        let defeated_by_concealment = defeating_countermeasures.mask_category(CountermeasureCategory::Concealment).any();

        // The seeker has been defeated by jamming, fall back to the next seeker
        if defeated_by_jamming { continue };

        // Validating seekers must invalidate all decoys, otherwise the seeker remains decoyed
        let mut decoys = defeating_countermeasures.mask_category(CountermeasureCategory::Decoy);
        for &validating_seeker in validating_seekers {
          // If the seeker is jammed, it cannot validate
          if validating_seeker.is_defeated_by(jamming) { continue };
          decoys &= validating_seeker.defeating_countermeasures_mask();
        };

        let defeated_by_decoys = decoys.any();

        // All decoys were invalidated, but the target is not seen by this seeker, fall back to the next seeker
        if !defeated_by_decoys && defeated_by_concealment { continue };

        // The missile is defeated by decoys
        return defeated_by_decoys;
      } else {
        // The missile is not defeated
        return false;
      };
    };

    // There are no more targeting seekers, the missile is defeated
    true
  }

  pub fn get_countermeasure_methods(&self) -> Box<[CountermeasuresMask]> {
    fn is_subset_of(sup: CountermeasuresMask, sub: CountermeasuresMask) -> bool {
      (sup != sub) && (sup & sub == sub)
    }

    let mut list = Vec::new();
    for countermeasures in CountermeasuresMask::values_ordered() {
      if list.iter().rev().any(|&sub| is_subset_of(countermeasures, sub)) { continue };
      if self.is_defeated_by(countermeasures) {
        list.push(countermeasures);
      };
    };

    list.into_boxed_slice()
  }
}

impl SeekerStrategy {
  pub fn min_cost(&self) -> f32 {
    self.iter().map(|(seeker, mode)| seeker.min_cost()[mode]).sum::<f32>()
  }

  pub fn max_cost(&self) -> f32 {
    self.iter().map(|(seeker, mode)| seeker.max_cost()[mode]).sum::<f32>()
  }

  pub fn to_full(&self) -> Vec<SeekerStrategyFull> {
    let primary = self.primary.iter_seeker_keys();
    let secondaries = self.secondaries.iter()
      .map(|&(seeker, mode)| seeker.iter_seeker_keys().map(move |s| (s, mode)))
      .multi_cartesian_product();
    primary.cartesian_product(secondaries)
      .map(|(primary, secondaries)| SeekerStrategy::new(primary, secondaries))
      .collect::<Vec<SeekerStrategyFull>>()
  }

  pub fn possibilities() -> impl Iterator<Item = Self> + Clone {
    Self::possibilities1().chain(Self::possibilities2()).chain(Self::possibilities3())
  }

  pub fn possibilities1() -> impl Iterator<Item = Self> + Clone {
    SeekerKind::values().map(|primary| SeekerStrategy { primary, secondaries: Box::new([]) })
  }

  pub fn possibilities2() -> impl Iterator<Item = Self> + Clone {
    let modes = [SeekerMode::Targeting, SeekerMode::Validation];
    SeekerKind::values()
      .cartesian_product(SeekerKind::values().cartesian_product(modes))
      .map(|(primary, secondary)| SeekerStrategy::new(primary, [secondary]))
  }

  pub fn possibilities3() -> impl Iterator<Item = Self> + Clone {
    let modes = [SeekerMode::Targeting, SeekerMode::Validation];
    SeekerKind::values()
      .cartesian_product(SeekerKind::values().cartesian_product(modes).clone())
      .cartesian_product(SeekerKind::values().cartesian_product(modes))
      .map(|((primary, secondary), tertiary)| SeekerStrategy::new(primary, [secondary, tertiary]))
  }
}

impl SeekerStrategyFull {
  pub fn cost(&self) -> f32 {
    self.iter().map(|(seeker, mode)| seeker.cost()[mode]).sum::<f32>()
  }
}

impl fmt::Display for SeekerStrategy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.primary, f)?;
    for &(seeker, mode) in self.secondaries.iter() {
      match mode {
        SeekerMode::Targeting => write!(f, "/{seeker}")?,
        SeekerMode::Validation => write!(f, "/[{seeker}]")?
      };
    };

    Ok(())
  }
}

pub type SeekerStrategyIter<'a, S = SeekerKind> = std::iter::Chain<
  std::iter::Once<(S, SeekerMode)>,
  std::iter::Copied<std::slice::Iter<'a, (S, SeekerMode)>>
>;

fn next_seeker_layer<'a, 'b, S: Copy + Into<SeekerKind>>(
  iter: &mut std::iter::Peekable<SeekerStrategyIter<'a, S>>,
  buffer: &'b mut Vec<SeekerKind>
) -> Option<(SeekerKind, &'b [SeekerKind])> {
  buffer.clear();
  let (seeker, mode) = iter.next()?;
  debug_assert_eq!(mode, SeekerMode::Targeting);
  while let Some((seeker, SeekerMode::Validation)) = iter.peek().copied() {
    buffer.push(seeker.into());
    iter.next();
  };

  Some((seeker.into(), buffer.as_slice()))
}

fn has_redundancy(seekers: &[SeekerKind]) -> bool {
  let mut unique = std::collections::HashSet::new();
  !seekers.iter().all(|&seeker| unique.insert(seeker)) ||
  (unique.contains(&SeekerKind::ActiveRadar) && unique.contains(&SeekerKind::SemiActiveRadar)) ||
  (unique.contains(&SeekerKind::AntiRadiation) && unique.contains(&SeekerKind::HomeOnJam))
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SeekerStrategyEntry {
  pub seeker_strategy: SeekerStrategy,
  pub countermeasure_methods: Box<[CountermeasuresMask]>
}

impl SeekerStrategyEntry {
  pub fn new(seeker_strategy: SeekerStrategy) -> Self {
    let countermeasure_methods = seeker_strategy.get_countermeasure_methods();
    SeekerStrategyEntry { seeker_strategy, countermeasure_methods }
  }

  pub fn get_defeat_probability(&self, probabilities: CountermeasureProbabilities) -> f32 {
    if self.countermeasure_methods.is_empty() { return 0.0 };
    let probabilities = self.countermeasure_methods.iter()
      .map(|mask| mask.iter_filtered().map(|cm| probabilities[cm]).product::<f32>())
      .collect::<Vec<f32>>();
    crate::utils::probability_any(&probabilities)
  }

  pub fn get_defeat_probability_default(&self) -> f32 {
    self.get_defeat_probability(COUNTERMEASURE_PROBABILITIES)
  }

  pub fn get_defeat_probability_default_no_ewar(&self) -> f32 {
    self.get_defeat_probability(COUNTERMEASURE_PROBABILITIES_NO_EWAR)
  }

  pub fn get_entries() -> Box<[Self]> {
    SeekerStrategy::possibilities()
      .filter(SeekerStrategy::is_reasonable)
      .map(SeekerStrategyEntry::new)
      .collect()
  }

  pub fn get_entries_cached() -> &'static [Self] {
    static LOCK: OnceLock<Box<[SeekerStrategyEntry]>> = OnceLock::new();
    LOCK.get_or_init(SeekerStrategyEntry::get_entries)
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Countermeasure {
  RadarJamming,
  CommsJamming,
  LaserDazzler,
  ChaffDecoy,
  FlareDecoy,
  ActiveDecoy,
  CutEngines,
  CutRadar
}

impl Countermeasure {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::RadarJamming => "Radar Jamming",
      Self::CommsJamming => "Comms Jamming",
      Self::LaserDazzler => "Laser Dazzler",
      Self::ChaffDecoy => "Chaff Decoy",
      Self::FlareDecoy => "Flare Decoy",
      Self::ActiveDecoy => "Active Decoy",
      Self::CutEngines => "Disable Engines",
      Self::CutRadar => "Disable Radar"
    }
  }

  pub const fn category(self) -> CountermeasureCategory {
    match self {
      Self::RadarJamming | Self::CommsJamming | Self::LaserDazzler => CountermeasureCategory::Jamming,
      Self::ChaffDecoy | Self::FlareDecoy | Self::ActiveDecoy => CountermeasureCategory::Decoy,
      Self::CutEngines | Self::CutRadar => CountermeasureCategory::Concealment
    }
  }
}

impl fmt::Display for Countermeasure {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CountermeasureCategory {
  /// Creates false target(s) which decieve or confuse the seeker.
  /// May be countered through validating seekers.
  Decoy,
  /// Prevents the seeker from operating entirely.
  /// May be countered through backup (targeting) seekers.
  Jamming,
  /// Mitigates or conceals signatures used by the seeker to detect the target.
  /// May be countered through backup (targeting) seekers.
  Concealment
}

impl CountermeasureCategory {
  pub const fn mask(self) -> CountermeasuresMask {
    match self {
      Self::Decoy => CountermeasuresMask::ONLY_DECOY,
      Self::Jamming => CountermeasuresMask::ONLY_JAMMING,
      Self::Concealment => CountermeasuresMask::ONLY_CONCEALMENT
    }
  }
}

/// Defines a mask/list of countermeasures that are currently employed.
pub type CountermeasuresMask = CountermeasureMatrix<bool>;

/// Defines the likelyhood of any given countermeasure's employment within the battlespace
/// for use by the generator in weighing a seeker strategy's resistance to said countermeasures.
pub type CountermeasureProbabilities = CountermeasureMatrix<f32>;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CountermeasureMatrix<T> {
  /// Radar jamming, such as from the 'Blanket' jammer.
  pub radar_jamming: T,
  /// Comms jamming, such as from the 'Hangup' jammer.
  pub comms_jamming: T,
  /// Electro-Optical seeker jamming, such as from the 'Blackjack' laser dazzler.
  pub laser_dazzler: T,
  /// Chaff decoys.
  pub chaff_decoy: T,
  /// Flare decoys.
  pub flare_decoy: T,
  /// Active decoys, only available to ANS.
  pub active_decoy: T,
  /// Ship has disabling/cutting its thrusters.
  /// This includes ships that are immobilized from damage.
  pub cut_engines: T,
  /// Ship has disabling all radar emissions.
  pub cut_radar: T
}

impl<T> CountermeasureMatrix<T> {
  pub const fn get(&self, cm: Countermeasure) -> &T {
    match cm {
      Countermeasure::RadarJamming => &self.radar_jamming,
      Countermeasure::CommsJamming => &self.comms_jamming,
      Countermeasure::LaserDazzler => &self.laser_dazzler,
      Countermeasure::ChaffDecoy => &self.chaff_decoy,
      Countermeasure::FlareDecoy => &self.flare_decoy,
      Countermeasure::ActiveDecoy => &self.active_decoy,
      Countermeasure::CutEngines => &self.cut_engines,
      Countermeasure::CutRadar => &self.cut_radar
    }
  }

  pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> CountermeasureMatrix<U> {
    CountermeasureMatrix::from_array(self.into_array().map(f))
  }

  pub fn map_with_tag<U, F: FnMut(Countermeasure, T) -> U>(self, mut f: F) -> CountermeasureMatrix<U> {
    CountermeasureMatrix {
      radar_jamming: f(Countermeasure::RadarJamming, self.radar_jamming),
      comms_jamming: f(Countermeasure::CommsJamming, self.comms_jamming),
      laser_dazzler: f(Countermeasure::LaserDazzler, self.laser_dazzler),
      chaff_decoy: f(Countermeasure::ChaffDecoy, self.chaff_decoy),
      flare_decoy: f(Countermeasure::FlareDecoy, self.flare_decoy),
      active_decoy: f(Countermeasure::ActiveDecoy, self.active_decoy),
      cut_engines: f(Countermeasure::CutEngines, self.cut_engines),
      cut_radar: f(Countermeasure::CutRadar, self.cut_radar)
    }
  }

  pub fn zip<U, V, F: FnMut(T, U) -> V>(self, other: CountermeasureMatrix<U>, mut f: F) -> CountermeasureMatrix<V> {
    CountermeasureMatrix {
      radar_jamming: f(self.radar_jamming, other.radar_jamming),
      comms_jamming: f(self.comms_jamming, other.comms_jamming),
      laser_dazzler: f(self.laser_dazzler, other.laser_dazzler),
      chaff_decoy: f(self.chaff_decoy, other.chaff_decoy),
      flare_decoy: f(self.flare_decoy, other.flare_decoy),
      active_decoy: f(self.active_decoy, other.active_decoy),
      cut_engines: f(self.cut_engines, other.cut_engines),
      cut_radar: f(self.cut_radar, other.cut_radar)
    }
  }

  #[inline]
  pub const fn from_array(array: [T; COUNTERMEASURE_MATRIX_LEN]) -> Self {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { union_cast!(array, CountermeasureMatrixCast, array, this) }
  }

  #[inline]
  pub const fn from_array_ref(array: &[T; COUNTERMEASURE_MATRIX_LEN]) -> &Self {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { crate::utils::cast_ref(array) }
  }

  #[inline]
  pub fn from_array_ref_mut(array: &mut [T; COUNTERMEASURE_MATRIX_LEN]) -> &mut Self {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { crate::utils::cast_ref_mut(array) }
  }

  #[inline]
  pub const fn into_array(self) -> [T; COUNTERMEASURE_MATRIX_LEN] {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { union_cast!(self, CountermeasureMatrixCast, this, array) }
  }

  #[inline]
  pub const fn as_array_ref(&self) -> &[T; COUNTERMEASURE_MATRIX_LEN] {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { crate::utils::cast_ref(self) }
  }

  #[inline]
  pub fn as_array_ref_mut(&mut self) -> &mut [T; COUNTERMEASURE_MATRIX_LEN] {
    // SAFETY: `CountermeasureMatrix` is repr(C), so has the same memory representation as `[T; 8]`
    unsafe { crate::utils::cast_ref_mut(self) }
  }
}

macro_rules! const_binop {
  ($a:expr, $b:expr, $op:tt) => ({
    let a = $a.into_array();
    let b = $b.into_array();
    let mut out = [false; COUNTERMEASURE_MATRIX_LEN];
    let mut i = 0;
    while i < COUNTERMEASURE_MATRIX_LEN {
      out[i] = a[i] & b[i];
      i += 1;
    };

    CountermeasuresMask::from_array(out)
  });
}

impl CountermeasuresMask {
  pub const ALL: Self = CountermeasureMatrix::from_array([true; COUNTERMEASURE_MATRIX_LEN]);
  pub const NONE: Self = CountermeasureMatrix::from_array([false; COUNTERMEASURE_MATRIX_LEN]);

  pub const ONLY_DECOY: Self = Self { chaff_decoy: true, flare_decoy: true, active_decoy: true, ..Self::NONE };
  pub const ONLY_JAMMING: Self = Self { radar_jamming: true, comms_jamming: true, laser_dazzler: true, ..Self::NONE };
  pub const ONLY_CONCEALMENT: Self = Self { cut_engines: true, cut_radar: true, ..Self::NONE };

  /// Countermeasures available to the Alliance faction (all except for Laser Dazzler).
  pub const ONLY_ALLIANCE_COUNTERMEASURES: Self = Self { laser_dazzler: false, ..Self::ALL };
  /// Countermeasures available to the Protectorate faction (all except for Active Decoy).
  pub const ONLY_PROTECTORATE_COUNTERMEASURES: Self = Self { active_decoy: false, ..Self::ALL };

  pub const fn from_faction(faction: Faction) -> Self {
    match faction {
      Faction::Alliance => Self::ONLY_ALLIANCE_COUNTERMEASURES,
      Faction::Protectorate => Self::ONLY_PROTECTORATE_COUNTERMEASURES
    }
  }

  pub const fn any(self) -> bool {
    !self.none()
  }

  pub const fn all(self) -> bool {
    matches!(self, Self::ALL)
  }

  pub const fn none(self) -> bool {
    matches!(self, Self::NONE)
  }

  pub const fn and(self, other: Self) -> Self {
    const_binop!(self, other, &)
  }

  pub const fn or(self, other: Self) -> Self {
    const_binop!(self, other, |)
  }

  pub const fn xor(self, other: Self) -> Self {
    const_binop!(self, other, ^)
  }

  pub const fn not(self) -> Self {
    let mut out = self.into_array();
    let mut i = 0;
    while i < COUNTERMEASURE_MATRIX_LEN {
      out[i] = !out[i];
      i += 1;
    };

    Self::from_array(out)
  }

  pub const fn count_trues(self) -> usize {
    let array = self.into_array();
    let mut count = 0;
    let mut i = 0;
    while i < COUNTERMEASURE_MATRIX_LEN {
      if array[i] { count += 1 };
      i += 1;
    };

    count
  }

  pub const fn count_falses(self) -> usize {
    let array = self.into_array();
    let mut count = 0;
    let mut i = 0;
    while i < COUNTERMEASURE_MATRIX_LEN {
      if !array[i] { count += 1 };
      i += 1;
    };

    count
  }

  #[inline]
  pub const fn from_num(num: u32) -> Self {
    Self::from_array(crate::utils::bool_array_from_num(num))
  }

  #[inline]
  pub const fn to_num(self) -> u32 {
    crate::utils::bool_array_to_num(self.into_array())
  }

  pub const fn mask_faction(self, faction: Faction) -> Self {
    self.and(Self::from_faction(faction))
  }

  pub const fn mask_faction_inv(self, faction: Faction) -> Self {
    self.and(Self::from_faction(faction).not())
  }

  pub const fn mask_category(self, category: CountermeasureCategory) -> Self {
    self.and(category.mask())
  }

  pub const fn mask_category_inv(self, category: CountermeasureCategory) -> Self {
    self.and(category.mask().not())
  }

  pub fn iter_filtered(self) -> impl Iterator<Item = Countermeasure> + DoubleEndedIterator + Clone {
    Countermeasure::values().filter(move |&cm| self[cm])
  }

  pub fn values() -> impl Iterator<Item = Self> + DoubleEndedIterator + Clone {
    const MAX: u32 = CountermeasuresMask::ALL.to_num();
    (0..=MAX).map(Self::from_num)
  }

  pub fn values_ordered() -> impl Iterator<Item = Self> + Clone {
    crate::utils::combination_iter::<{COUNTERMEASURE_MATRIX_LEN as u32}>()
      .map(|num| Self::from_num(num.get()))
  }
}

impl fmt::Display for CountermeasuresMask {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, cm) in self.iter_filtered().enumerate() {
      if i != 0 { f.write_str(" + ")? };
      f.write_str(cm.to_str())?;
    };

    Ok(())
  }
}

impl<T> Index<Countermeasure> for CountermeasureMatrix<T> {
  type Output = T;

  fn index(&self, cm: Countermeasure) -> &Self::Output {
    self.get(cm)
  }
}

impl<T> IntoIterator for CountermeasureMatrix<T> {
  type IntoIter = <[T; COUNTERMEASURE_MATRIX_LEN] as IntoIterator>::IntoIter;
  type Item = T;

  fn into_iter(self) -> Self::IntoIter {
    self.into_array().into_iter()
  }
}

impl<'a, T> IntoIterator for &'a CountermeasureMatrix<T> {
  type IntoIter = <&'a [T; COUNTERMEASURE_MATRIX_LEN] as IntoIterator>::IntoIter;
  type Item = &'a T;

  fn into_iter(self) -> Self::IntoIter {
    self.as_array_ref().into_iter()
  }
}

impl<'a, T> IntoIterator for &'a mut CountermeasureMatrix<T> {
  type IntoIter = <&'a mut [T; COUNTERMEASURE_MATRIX_LEN] as IntoIterator>::IntoIter;
  type Item = &'a mut T;

  fn into_iter(self) -> Self::IntoIter {
    self.as_array_ref_mut().into_iter()
  }
}

macro_rules! impl_bit_binop {
  ($Trait:ident, $function:ident, $TraitAssign:ident, $function_assign:ident) => {
    impl<T, Rhs> $Trait<CountermeasureMatrix<Rhs>> for CountermeasureMatrix<T> where T: $Trait<Rhs> {
      type Output = CountermeasureMatrix<T::Output>;

      fn $function(self, rhs: CountermeasureMatrix<Rhs>) -> Self::Output {
        self.zip(rhs, T::$function)
      }
    }

    impl<T, Rhs> $TraitAssign<CountermeasureMatrix<Rhs>> for CountermeasureMatrix<T> where T: $TraitAssign<Rhs> {
      fn $function_assign(&mut self, rhs: CountermeasureMatrix<Rhs>) {
        for (lhs, rhs) in self.into_iter().zip(rhs) {
          T::$function_assign(lhs, rhs);
        };
      }
    }
  };
}

impl_bit_binop!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_bit_binop!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_bit_binop!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl<T> Not for CountermeasureMatrix<T> where T: Not {
  type Output = CountermeasureMatrix<T::Output>;

  fn not(self) -> Self::Output {
    self.map(T::not)
  }
}

/// A matrix of base probabilities for estimating the employment of any given countermeasure,
/// with EWAR countermeasure probabilities set to 0%.
pub const COUNTERMEASURE_PROBABILITIES_NO_EWAR: CountermeasureProbabilities = {
  CountermeasureMatrix { radar_jamming: 0.0, comms_jamming: 0.0, ..COUNTERMEASURE_PROBABILITIES }
};

/// A matrix of base probabilities for estimating the employment of any given countermeasure.
pub const COUNTERMEASURE_PROBABILITIES: CountermeasureProbabilities = {
  CountermeasureMatrix {
    radar_jamming: 0.90,
    comms_jamming: 0.80,
    laser_dazzler: 0.05,
    chaff_decoy: 0.95,
    flare_decoy: 0.15,
    active_decoy: 0.50,
    cut_engines: 0.20,
    cut_radar: 0.05
  }
};

/// A matrix of base probabilities for estimating the employment of any given countermeasure by the Alliance.
pub const COUNTERMEASURE_PROBABILITIES_VS_ALLIANCE: CountermeasureProbabilities = {
  CountermeasureMatrix {
    laser_dazzler: 0.0,
    ..COUNTERMEASURE_PROBABILITIES
  }
};

/// A matrix of base probabilities for estimating the employment of any given countermeasure by the Protectorate.
pub const COUNTERMEASURE_PROBABILITIES_VS_PROTECTORATE: CountermeasureProbabilities = {
  CountermeasureMatrix {
    // only the ocello has access to comms jamming, so the chance of comms jamming is lower
    comms_jamming: COUNTERMEASURE_PROBABILITIES.comms_jamming / 2.0,
    active_decoy: 0.0,
    ..COUNTERMEASURE_PROBABILITIES
  }
};

const COUNTERMEASURE_MATRIX_LEN: usize = std::mem::size_of::<CountermeasureMatrix<u8>>();

union CountermeasureMatrixCast<T> {
  this: std::mem::ManuallyDrop<CountermeasureMatrix<T>>,
  array: std::mem::ManuallyDrop<[T; COUNTERMEASURE_MATRIX_LEN]>
}
