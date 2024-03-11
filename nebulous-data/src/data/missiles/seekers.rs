use std::fmt;
use std::num::NonZeroUsize as zsize;
use std::ops::Index;
use std::str::FromStr;

use bytemuck::Contiguous;
use itertools::{Itertools, Either};



/// A seeker that can be chosen in game.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
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
  pub const fn cost(self) -> (f32, f32) {
    match self {
      Self::Command => (3.50, 3.00),
      Self::FixedActiveRadar => (1.00, 0.25),
      Self::SteerableActiveRadar => (1.50, 0.50),
      Self::SteerableExtendedActiveRadar => (3.00, 1.00),
      Self::FixedSemiActiveRadar => (0.00, 0.50),
      Self::FixedAntiRadiation => (2.00, 2.00),
      Self::FixedHomeOnJam => (0.50, 0.50,),
      Self::ElectroOptical => (8.00, 5.00),
      Self::WakeHoming => (0.25, 0.50)
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

  #[inline]
  pub const fn values() -> crate::utils::ContiguousEnumValues<Self> {
    crate::utils::ContiguousEnumValues::new()
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

  const fn countermeasures(self) -> &'static [Countermeasure] {
    match self {
      Self::Command => &[
        Countermeasure::CommsJamming
      ],
      Self::ActiveRadar | Self::SemiActiveRadar => &[
        Countermeasure::RadarJamming,
        Countermeasure::ChaffDecoy,
        Countermeasure::ActiveDecoy
      ],
      Self::AntiRadiation => &[
        Countermeasure::ActiveDecoy,
        Countermeasure::CutRadar
      ],
      Self::HomeOnJam => &[],
      Self::ElectroOptical => &[
        Countermeasure::LaserDazzler
      ],
      Self::WakeHoming => &[
        Countermeasure::FlareDecoy,
        Countermeasure::CutEngines
      ]
    }
  }

  #[inline]
  pub const fn values() -> crate::utils::ContiguousEnumValues<Self> {
    crate::utils::ContiguousEnumValues::new()
  }
}

impl fmt::Display for SeekerKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeekerLayout {
  pub primary: SeekerKey,
  pub secondary: Option<(SeekerKey, SeekerMode)>
}

impl SeekerLayout {
  pub fn guidance_quality(self) -> f32 {
    let primary = self.primary.base_guidance_quality() + 2.5;
    if let Some((secondary, mode)) = self.secondary {
      let secondary = secondary.base_guidance_quality();
      match mode {
        SeekerMode::Targeting => primary + secondary / 2.0,
        SeekerMode::Validation => (primary + primary.max(secondary + 2.5)) / 2.0
      }
    } else {
      primary
    }
  }

  pub fn cost(self) -> f32 {
    let primary_cost = self.primary.cost().0;
    let secondary_cost = match self.secondary {
      Some((secondary, SeekerMode::Targeting)) => secondary.cost().0,
      Some((secondary, SeekerMode::Validation)) => secondary.cost().1,
      None => 0.0
    };

    primary_cost + secondary_cost
  }
}

impl fmt::Display for SeekerLayout {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.secondary {
      Some((secondary, SeekerMode::Targeting)) => write!(f, "{}/{}", self.primary, secondary),
      Some((secondary, SeekerMode::Validation)) => write!(f, "{}/[{}]", self.primary, secondary),
      None => write!(f, "{}", self.primary)
    }
  }
}

#[derive(Debug, Clone)]
pub struct SeekerStrategy2 {
  pub primary: SeekerKind,
  pub secondaries: Box<[(SeekerKind, SeekerMode)]>
}

impl SeekerStrategy2 {
  pub const fn len(&self) -> zsize {
    zsize!(self.secondaries.len().wrapping_add(1))
  }

  pub fn is_reasonable(&self) -> bool {
    fn has_redundancy(seekers: &[SeekerKind]) -> bool {
      let mut unique = std::collections::HashSet::new();
      !seekers.iter().all(|&seeker| unique.insert(seeker)) ||
      (unique.contains(&SeekerKind::ActiveRadar) && unique.contains(&SeekerKind::SemiActiveRadar))
    }

    let mut i = 0;
    let mut buffer = Vec::new();
    let mut iter = self.iter().peekable();
    let mut targeting_seekers = Vec::new();
    while let Some((seeker, validating_seekers)) = next_seeker_layer(&mut iter, &mut buffer) {
      targeting_seekers.push(seeker);
      let is_primary = i == 0;

      // Home-On-Jam seekers cannot be primary seekers, because Anti-Radiation would always be better
      if matches!(seeker, SeekerKind::HomeOnJam) && is_primary { return false };
      // Command recievers cannot be backups because they would be better as primaries
      if matches!(seeker, SeekerKind::Command) && !is_primary { return false };
      // Wake-Homing seekers cannot have backups because those backups would always be better as primaries
      if matches!(self.primary, SeekerKind::WakeHoming) && !is_primary { return false };
      // Command recievers and Wake-Homing seekers cannot have validators
      if matches!(seeker, SeekerKind::Command | SeekerKind::WakeHoming) && !validating_seekers.is_empty() { return false };
      // Electro-Optical cannot have validators, except for Command reciever validators
      if matches!(seeker, SeekerKind::ElectroOptical) && !(validating_seekers.is_empty() || validating_seekers.contains(&SeekerKind::Command)) { return false };
      // Active Radar and Semi-Active radar are not allowed to be validators
      if validating_seekers.contains(&SeekerKind::ActiveRadar) || validating_seekers.contains(&SeekerKind::SemiActiveRadar) { return false };
      // Filter out redundant combinations (duplicate validating seekers)
      if has_redundancy(validating_seekers) { return false };

      i += 1;
    };

    // Filter out redundant combinations (duplicate targeting seekers)
    if has_redundancy(&targeting_seekers) { return false };

    false
  }

  pub fn iter(&self) -> SeekerStrategyIter {
    std::iter::once((self.primary, SeekerMode::Targeting)).chain(self.secondaries.iter().copied())
  }

  pub fn possibilities1() -> impl Iterator<Item = Self> {
    SeekerKind::values().map(|primary| SeekerStrategy2 { primary, secondaries: Box::new([]) })
  }

  pub fn possibilities2() -> impl Iterator<Item = Self> {
    let modes = [SeekerMode::Targeting, SeekerMode::Validation];
    SeekerKind::values()
      .cartesian_product(SeekerKind::values().cartesian_product(modes))
      .map(|(primary, secondary)| SeekerStrategy2 { primary, secondaries: Box::new([secondary]) })
  }
}

impl fmt::Display for SeekerStrategy2 {
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

pub type SeekerStrategyIter<'a> = std::iter::Chain<
  std::iter::Once<(SeekerKind, SeekerMode)>,
  std::iter::Copied<std::slice::Iter<'a, (SeekerKind, SeekerMode)>>
>;

fn next_seeker_layer<'a, 'b>(
  iter: &mut std::iter::Peekable<SeekerStrategyIter<'a>>,
  buffer: &'b mut Vec<SeekerKind>
) -> Option<(SeekerKind, &'b [SeekerKind])> {
  buffer.clear();
  let (seeker, mode) = iter.next()?;
  debug_assert_eq!(mode, SeekerMode::Targeting);
  while let Some((seeker, SeekerMode::Validation)) = iter.peek().copied() {
    buffer.push(seeker);
    iter.next();
  };

  Some((seeker, buffer.as_slice()))
}

#[derive(Debug, Clone, Copy)]
pub struct SeekerStrategy {
  pub primary: SeekerKind,
  pub secondary: Option<(SeekerKind, SeekerMode)>,
  /// A list of ways this seeker configuration can be defeated.
  pub defeated_by: &'static [&'static [Countermeasure]]
}

impl SeekerStrategy {
  pub const fn new_single(
    seeker: SeekerKind,
    defeated_by: &'static [&'static [Countermeasure]]
  ) -> Self {
    SeekerStrategy { primary: seeker, secondary: None, defeated_by }
  }

  pub const fn new_double(
    primary: SeekerKind, secondary: SeekerKind, mode: SeekerMode,
    defeated_by: &'static [&'static [Countermeasure]]
  ) -> Self {
    SeekerStrategy { primary, secondary: Some((secondary, mode)), defeated_by }
  }

  pub fn layouts(self) -> Vec<SeekerLayout> {
    if let Some((secondary, mode)) = self.secondary {
      self.primary.seeker_keys().iter()
        .flat_map(move |&primary| {
          secondary.seeker_keys().iter().map(move |&secondary| {
            SeekerLayout { primary, secondary: Some((secondary, mode)) }
          })
        })
        .collect()
    } else {
      self.primary.seeker_keys().iter()
        .map(move |&primary| SeekerLayout { primary, secondary: None })
        .collect()
    }
  }

  /// The probability that, based on the provided [`CountermeasureProbabilities`],
  /// this seeker configuration can be defeated by countermeasures fielded by a hypothetical opponent.
  pub fn defeat_probability(self, probabilities: CountermeasureProbabilities) -> f32 {
    let probabilities = self.defeated_by.iter()
      .map(|&d| d.iter().map(|&cm| probabilities[cm]).product::<f32>())
      .collect::<Vec<f32>>();
    crate::utils::probability_any(&probabilities)
  }

  /// Defines a list of 'reasonable' seeker configurations that the generator may pick from.
  /// Some configurations are intentionally excluded from this list:
  /// - `CMD` is never allowed to have validators because `CMD` is already the most exact (in target discrimination).
  /// - `EO` is not allowed to have validators (except for `CMD` validator) for similar reasons.
  /// - `CMD` is never allowed to be a backup seeker because it would *always* be better served as the primary seeker.
  /// - `WAKE` is not allowed to have backups or validators because that seeker would *always* be better served as the primary.
  /// - `ACT(RADAR)` and `SAH(RADAR)` are not allowed to be validators because they are broad and will rarely actually filter things out.
  /// - `HOJ` is not allowed to be a primary or a validator because such use cases are either too niche or inferior to `ARAD` in its place.
  /// - Redundant combinations like `ACT(RADAR)/SAH(RADAR)` combinations are not allowed for obvious reasons.
  ///
  /// Other combinations are probably generally bad ideas or have too niche of applications,
  /// but are likely to be filtered out by the generator due to cost considerations.
  ///
  /// Additionally, this all works under the assumption that Reject Unvalidated Targets is never used.
  pub const VALUES: &'static [Self] = {
    use Countermeasure::*;
    use SeekerMode::{Targeting as BKP, Validation as VAL};
    use SeekerKind::{
      Command as CMD,
      ActiveRadar as ACT, SemiActiveRadar as SAH,
      AntiRadiation as ARAD, HomeOnJam as HOJ,
      ElectroOptical as EO, WakeHoming as WAKE
    };

    &[
      Self::new_single(CMD, &[
        &[CommsJamming]
      ]),
      Self::new_double(CMD, ACT, BKP, &[
        &[CommsJamming, RadarJamming],
        &[CommsJamming, ChaffDecoy],
        &[CommsJamming, ActiveDecoy]
      ]),
      Self::new_double(CMD, SAH, BKP, &[
        &[CommsJamming, RadarJamming],
        &[CommsJamming, ChaffDecoy],
        &[CommsJamming, ActiveDecoy]
      ]),
      Self::new_double(CMD, ARAD, BKP, &[
        &[CommsJamming, CutRadar],
        &[CommsJamming, ActiveDecoy]
      ]),
      Self::new_double(CMD, EO, BKP, &[
        &[CommsJamming, LaserDazzler]
      ]),
      Self::new_double(CMD, WAKE, BKP, &[
        &[CommsJamming, FlareDecoy]
      ]),
      Self::new_single(ACT, &[
        &[RadarJamming],
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(ACT, CMD, VAL, &[
        &[RadarJamming],
        &[ChaffDecoy, CommsJamming],
        &[ActiveDecoy, CommsJamming]
      ]),
      Self::new_double(ACT, ARAD, BKP, &[
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(ACT, ARAD, VAL, &[
        &[RadarJamming],
        &[ActiveDecoy]
      ]),
      Self::new_double(ACT, HOJ, BKP, &[
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(ACT, EO, BKP, &[
        &[RadarJamming, LaserDazzler],
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(ACT, EO, VAL, &[
        &[RadarJamming],
        &[ChaffDecoy, LaserDazzler],
        &[ActiveDecoy, LaserDazzler]
      ]),
      Self::new_double(ACT, WAKE, BKP, &[
        &[RadarJamming, FlareDecoy],
        &[RadarJamming, CutEngines],
        &[ChaffDecoy, FlareDecoy],
        &[ChaffDecoy, CutEngines],
        &[ActiveDecoy, FlareDecoy],
        &[ActiveDecoy, CutEngines]
      ]),
      Self::new_double(ACT, WAKE, VAL, &[
        &[RadarJamming]
      ]),
      Self::new_single(SAH, &[
        &[RadarJamming],
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(SAH, CMD, VAL, &[
        &[RadarJamming, CommsJamming],
        &[ChaffDecoy, CommsJamming],
        &[ActiveDecoy, CommsJamming]
      ]),
      Self::new_double(SAH, ARAD, BKP, &[
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(SAH, ARAD, VAL, &[
        &[RadarJamming],
        &[ActiveDecoy]
      ]),
      Self::new_double(SAH, HOJ, BKP, &[
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(SAH, EO, BKP, &[
        &[RadarJamming, LaserDazzler],
        &[ChaffDecoy],
        &[ActiveDecoy]
      ]),
      Self::new_double(SAH, EO, VAL, &[
        &[RadarJamming],
        &[ChaffDecoy, LaserDazzler],
        &[ActiveDecoy, LaserDazzler]
      ]),
      Self::new_double(SAH, WAKE, BKP, &[
        &[RadarJamming, FlareDecoy],
        &[RadarJamming, CutEngines],
        &[ChaffDecoy, FlareDecoy],
        &[ChaffDecoy, CutEngines],
        &[ActiveDecoy, FlareDecoy],
        &[ActiveDecoy, CutEngines]
      ]),
      Self::new_double(SAH, WAKE, VAL, &[
        &[RadarJamming]
      ]),
      Self::new_single(ARAD, &[
        &[ActiveDecoy],
        &[CutRadar]
      ]),
      Self::new_double(ARAD, CMD, VAL, &[
        &[ActiveDecoy, CommsJamming],
        &[CutRadar, CommsJamming]
      ]),
      Self::new_double(ARAD, ACT, BKP, &[
        &[ActiveDecoy],
        &[CutRadar, ChaffDecoy]
      ]),
      Self::new_double(ARAD, SAH, BKP, &[
        &[ActiveDecoy],
        &[CutRadar, ChaffDecoy]
      ]),
      Self::new_double(ARAD, EO, BKP, &[
        &[ActiveDecoy]
      ]),
      Self::new_double(ARAD, EO, VAL, &[
        &[ActiveDecoy, LaserDazzler],
        &[CutRadar, LaserDazzler]
      ]),
      Self::new_double(ARAD, WAKE, BKP, &[
        &[ActiveDecoy],
        &[CutRadar, CutEngines]
      ]),
      Self::new_double(ARAD, WAKE, VAL, &[
        &[CutRadar, CutEngines]
      ]),
      Self::new_single(EO, &[
        &[LaserDazzler]
      ]),
      Self::new_double(EO, CMD, VAL, &[
        &[LaserDazzler]
      ]),
      Self::new_double(EO, ACT, BKP, &[
        &[LaserDazzler, RadarJamming],
        &[LaserDazzler, ChaffDecoy],
        &[LaserDazzler, ActiveDecoy]
      ]),
      Self::new_double(EO, SAH, BKP, &[
        &[LaserDazzler, RadarJamming],
        &[LaserDazzler, ChaffDecoy],
        &[LaserDazzler, ActiveDecoy]
      ]),
      Self::new_double(EO, ARAD, BKP, &[
        &[LaserDazzler, ActiveDecoy],
        &[LaserDazzler, CutRadar]
      ]),
      Self::new_double(EO, WAKE, BKP, &[
        &[LaserDazzler, FlareDecoy],
        &[LaserDazzler, CutEngines]
      ]),
      Self::new_single(WAKE, &[
        &[FlareDecoy],
        &[CutEngines]
      ])
    ]
  };
}

impl fmt::Display for SeekerStrategy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.secondary {
      Some((secondary, SeekerMode::Targeting)) => write!(f, "{}/{}", self.primary, secondary),
      Some((secondary, SeekerMode::Validation)) => write!(f, "{}/[{}]", self.primary, secondary),
      None => write!(f, "{}", self.primary)
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
      Self::CutEngines | Self::CutRadar => todo!()
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

/// Defines the likelyhood of any given countermeasure's employment within the battlespace
/// for use by the generator in weighing a seeker strategy's resistance to said countermeasures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CountermeasureProbabilities {
  /// RADAR jamming, such as from the 'Blanket' jammer.
  pub radar_jamming: f32,
  /// COMMS jamming, such as from the 'Hangup' jammer.
  pub comms_jamming: f32,
  /// Electro-Optical seeker jamming, such as from the 'Blackjack' laser dazzler.
  pub laser_dazzler: f32,
  /// Chaff decoys.
  pub chaff_decoy: f32,
  /// Flare decoys.
  pub flare_decoy: f32,
  /// Active decoys, only available to ANS.
  pub active_decoy: f32,
  /// Ship has disabled/cut its thrusters.
  /// This includes ships that are immobilized from damage.
  pub cut_engines: f32,
  /// Ship has disabled all radar emissions.
  pub cut_radar: f32
}

impl Index<Countermeasure> for CountermeasureProbabilities {
  type Output = f32;

  fn index(&self, cm: Countermeasure) -> &Self::Output {
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
}

impl Default for CountermeasureProbabilities {
  fn default() -> Self {
    CountermeasureProbabilities {
      radar_jamming: 0.90,
      comms_jamming: 0.80,
      laser_dazzler: 0.05,
      chaff_decoy: 0.95,
      flare_decoy: 0.15,
      active_decoy: 0.50,
      cut_engines: 0.20,
      cut_radar: 0.05
    }
  }
}
