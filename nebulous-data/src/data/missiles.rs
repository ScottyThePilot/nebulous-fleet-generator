use super::Faction;

use std::fmt;
use std::ops::Index;
use std::str::FromStr;



/// A seeker that can be chosen in game.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
}

impl fmt::Display for SeekerKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.seeker_kind(), f)
  }
}

/// Describes a seeker's method of operation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
      _ => Err(crate::data::InvalidKey)
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

#[derive(Debug, Clone, Copy)]
pub struct SeekerStrategy {
  pub primary: SeekerKind,
  pub secondary: Option<(SeekerKind, SeekerMode)>,
  /// A list of ways this seeker configuration can be decoyed.
  pub decoyed_by: &'static [&'static [Countermeasure]]
}

impl SeekerStrategy {
  pub const fn new_single(
    seeker: SeekerKind,
    decoyed_by: &'static [&'static [Countermeasure]]
  ) -> Self {
    SeekerStrategy { primary: seeker, secondary: None, decoyed_by }
  }

  pub const fn new_double(
    primary: SeekerKind, secondary: SeekerKind, mode: SeekerMode,
    decoyed_by: &'static [&'static [Countermeasure]]
  ) -> Self {
    SeekerStrategy { primary, secondary: Some((secondary, mode)), decoyed_by }
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
  /// this seeker configuration can be decoyed by fielded enemy countermeasures.
  pub fn decoy_probability(self, probabilities: CountermeasureProbabilities) -> f32 {
    let probabilities = self.decoyed_by.iter()
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
pub enum PayloadKey {
  HEImpact,
  HEKineticPenetrator,
  BlastFragmentation,
  BlastFragmentationEL
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissileBody {
  pub name: &'static str,
  pub save_key: &'static str,
  pub faction: Option<Faction>
}

impl fmt::Display for Maneuvers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineSettings {
  pub top_speed: f32,
  pub burn_duration: f32,
  pub maneuverability: f32
}

impl EngineSettings {
  pub const fn from_array(a: [f32; 3]) -> Self {
    EngineSettings { top_speed: a[0], burn_duration: a[1], maneuverability: a[2] }
  }

  pub const fn to_array(self) -> [f32; 3] {
    [self.top_speed, self.burn_duration, self.maneuverability]
  }

  pub fn normalize(self) -> Self {
    let a = self.top_speed.max(0.0);
    let b = self.burn_duration.max(0.0);
    let c = self.maneuverability.max(0.0);
    let sum = a + b + c;
    if sum <= 0.0 {
      EngineSettings {
        top_speed: 1.0 / 3.0,
        burn_duration: 1.0 / 3.0,
        maneuverability: 1.0 / 3.0
      }
    } else {
      EngineSettings {
        top_speed: a / sum,
        burn_duration: b / sum,
        maneuverability: c / sum
      }
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MissileBodyKey {
  SGM1Balestra,
  SGM2Tempest,
  SGMH2Cyclone,
  SGMH3Atlatl,
  SGT3Pilum,
  CM4Container,
  CMS4Container,
}

impl MissileBodyKey {
  pub const fn save_key(self) -> &'static str {
    self.missile_body().save_key
  }

  pub const fn missile_body(self) -> &'static MissileBody {
    use self::list::*;

    match self {
      Self::SGM1Balestra => &SGM1_BALESTRA,
      Self::SGM2Tempest => &SGM2_TEMPEST,
      Self::SGMH2Cyclone => &SGMH2_CYCLONE,
      Self::SGMH3Atlatl => &SGMH3_ATLATL,
      Self::SGT3Pilum => &SGT3_PILUM,
      Self::CM4Container => &CM4_CONTAINER,
      Self::CMS4Container => &CMS4_CONTAINER
    }
  }

  pub const VALUES: &'static [Self] = &[
    Self::SGM1Balestra,
    Self::SGM2Tempest,
    Self::SGMH2Cyclone,
    Self::SGMH3Atlatl,
    Self::SGT3Pilum,
    Self::CM4Container,
    Self::CMS4Container
  ];
}

impl FromStr for MissileBodyKey {
  type Err = super::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    MissileBodyKey::VALUES.iter().copied()
      .find(|hull_key| hull_key.save_key() == s)
      .ok_or(super::InvalidKey)
  }
}

impl fmt::Display for MissileBodyKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}

pub mod list {
  use super::*;

  pub const SGM1_BALESTRA: MissileBody = MissileBody {
    name: "SGM-1",
    save_key: "Stock/SGM-1 Body",
    faction: None
  };

  pub const SGM2_TEMPEST: MissileBody = MissileBody {
    name: "SGM-2",
    save_key: "Stock/SGM-2 Body",
    faction: None
  };

  pub const SGMH2_CYCLONE: MissileBody = MissileBody {
    name: "SGM-H-2",
    save_key: "Stock/SGM-H-2 Body",
    faction: Some(Faction::Alliance)
  };

  pub const SGMH3_ATLATL: MissileBody = MissileBody {
    name: "SGM-H-3",
    save_key: "Stock/SGM-H-3 Body",
    faction: Some(Faction::Alliance)
  };

  pub const SGT3_PILUM: MissileBody = MissileBody {
    name: "SGT-3",
    save_key: "Stock/SGT-3 Body",
    faction: None
  };

  pub const CM4_CONTAINER: MissileBody = MissileBody {
    name: "CM-4",
    save_key: "Stock/CM-4 Body",
    faction: Some(Faction::Protectorate)
  };

  pub const CMS4_CONTAINER: MissileBody = MissileBody {
    name: "CM-S-4",
    save_key: "Stock/CM-S-4 Body",
    faction: Some(Faction::Protectorate)
  };
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
      _ => Err(crate::data::InvalidKey)
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
}

impl fmt::Display for Countermeasure {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
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
