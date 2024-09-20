use crate::data::components::ComponentKey;
use crate::data::hulls::HullKey;
use crate::data::hulls::config::Variant;
use crate::data::missiles::engines::EngineSettings;
use crate::data::missiles::{AuxiliaryKey, AvionicsKey, Maneuvers, WarheadKey};
use crate::data::missiles::seekers::{SeekerKey, SeekerMode, SeekerStrategy, SeekerStrategyFull};
use crate::data::missiles::bodies::MissileBodyKey;
use crate::format::*;
use crate::format::key::Key;

use indexmap::IndexMap;

use std::collections::HashMap;
use std::num::NonZeroUsize as zsize;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLoadout {
  pub hull_type: HullKey,
  pub hull_config: Option<[Variant; 3]>,
  pub sockets: Box<[Option<ShipLoadoutSocket>]>
}

impl ShipLoadout {
  pub fn from_ship(ship: &Ship) -> Option<Self> {
    let hull = ship.hull_type.hull();
    let hull_config = ship.hull_config.as_ref().zip(hull.config_template)
      .and_then(|(hull_config, config_template)| config_template.get_variants(hull_config));

    let mut component_map = ship.socket_map.iter()
      .map(|hull_socket| (hull_socket.key, ShipLoadoutSocket::from_hull_socket(hull_socket)))
      .collect::<HashMap<Key, ShipLoadoutSocket>>();
    let sockets = hull.sockets.iter()
      .map(|hull_socket| component_map.remove(&hull_socket.save_key))
      .collect::<Box<[Option<ShipLoadoutSocket>]>>();

    Some(ShipLoadout {
      hull_type: ship.hull_type,
      hull_config,
      sockets
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLoadoutSocket {
  component_key: ComponentKey,
  variant: Option<ShipLoadoutSocketVariant>
}

impl ShipLoadoutSocket {
  pub fn from_hull_socket(hull_socket: &HullSocket) -> Self {
    let component_key = hull_socket.component_name;
    let variant = hull_socket.component_data.as_ref()
      .map(ShipLoadoutSocketVariant::from_component_data);
    ShipLoadoutSocket { component_key, variant }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShipLoadoutSocketVariant {
  DeceptionComponent {
    identity_option: usize
  },
  MagazineComponent {
    magazine_contents: IndexMap<MunitionOrMissileKey, usize>
  }
}

impl ShipLoadoutSocketVariant {
  pub fn from_component_data(component_data: &ComponentData) -> Self {
    match *component_data {
      ComponentData::BulkMagazineData { ref load } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(load) }
      },
      ComponentData::CellLauncherData { ref missile_load } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(missile_load) }
      },
      ComponentData::ResizableCellLauncherData { ref missile_load, .. } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(missile_load) }
      },
      ComponentData::DeceptionComponentData { identity_option } => {
        Self::DeceptionComponent { identity_option }
      }
    }
  }
}

fn get_magazine_contents(load: &[MagazineSaveData]) -> IndexMap<MunitionOrMissileKey, usize> {
  let mut magazine_contents = IndexMap::new();
  for &MagazineSaveData { ref munition_key, quantity, .. } in load.iter() {
    *magazine_contents.entry(munition_key.clone()).or_default() += quantity;
  };

  magazine_contents
}

#[derive(Debug, Clone, PartialEq)]
pub struct MissileTemplateAdditional {
  pub designation: String,
  pub nickname: String,
  pub description: String,
  pub long_description: String,
  pub cost: usize,
  pub template_key: Uuid,
  pub base_color: Color,
  pub stripe_color: Color
}

#[derive(Debug, Error, Clone, Copy)]
pub enum MissileLoadoutError {
  #[error("invalid missile component")]
  InvalidMissileComponent
}

#[derive(Debug, Clone, PartialEq)]
pub struct MissileLoadout {
  pub body_key: MissileBodyKey,
  pub sockets: Box<[MissileLoadoutSocket]>
}

impl MissileLoadout {
  pub fn from_missile_template(missile_template: &MissileTemplate) -> Result<Self, MissileLoadoutError> {
    let sockets = missile_template.sockets.iter().copied()
      .map(MissileLoadoutSocket::from_missile_socket)
      .collect::<Result<Box<[_]>, _>>()?;
    Ok(MissileLoadout { body_key: missile_template.body_key, sockets })
  }

  pub fn iter_seekers(&self) -> impl Iterator<Item = (SeekerKey, SeekerMode)> + '_ {
    self.sockets.iter().filter_map(|socket| {
      if let Some(MissileLoadoutComponent::Seeker(seeker)) = socket.component {
        let (seeker_configured, mode, _) = seeker.into_parts();
        Some((seeker_configured.into_seeker_key(), mode))
      } else {
        None
      }
    })
  }

  pub fn get_seeker_strategy_full(&self) -> Option<SeekerStrategyFull> {
    SeekerStrategy::try_from_iter(self.iter_seekers())
  }

  pub fn get_seeker_strategy_basic(&self) -> Option<SeekerStrategy> {
    SeekerStrategy::try_from_iter({
      self.iter_seekers().map(|(seeker, mode)| (seeker.seeker_kind(), mode))
    })
  }

  pub fn len(&self) -> zsize {
    zsize!(self.sockets.iter().map(|socket| socket.size.get()).sum::<usize>())
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissileLoadoutSocket {
  pub component: Option<MissileLoadoutComponent>,
  pub size: zsize
}

impl MissileLoadoutSocket {
  pub fn from_missile_socket(missile_socket: MissileSocket) -> Result<Self, MissileLoadoutError> {
    missile_socket.installed_component.map(MissileLoadoutComponent::from_missile_component)
      .transpose().map(|component| MissileLoadoutSocket { component, size: missile_socket.size })
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MissileLoadoutComponent {
  Seeker(SeekerWithMode),
  Avionics(AvionicsConfigured),
  Auxiliary(AuxiliaryKey),
  Warhead(WarheadKey),
  Engine(EngineSettings)
}

impl MissileLoadoutComponent {
  pub fn from_missile_component(missile_component: MissileComponent) -> Result<Self, MissileLoadoutError> {
    let MissileComponent { component_key, settings } = missile_component;
    let component_key = component_key.map(MissileLoadoutComponentKey::from_missile_component_key);

    Ok(match (component_key, settings) {
      (Some(MissileLoadoutComponentKey::CommandReceiver), Some(MissileComponentSettings::CommandSeekerSettings { mode })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::Command, mode, false))
      },
      (Some(MissileLoadoutComponentKey::FixedActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::FixedActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::SteerableActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::SteerableActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::SteerableExtendedActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::SteerableExtendedActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::FixedSemiActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::FixedSemiActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::FixedAntiRadiationSeeker), Some(MissileComponentSettings::PassiveARHSeekerSettings { mode, reject_unvalidated, home_on_jam })) => {
        let seeker_configured = if home_on_jam { SeekerConfigured::FixedHomeOnJam } else { SeekerConfigured::FixedAntiRadiation };
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(seeker_configured, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::ElectroOpticalSeeker), Some(MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::ElectroOptical { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::WakeHomingSeeker), Some(MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerWithMode::new(SeekerConfigured::WakeHoming { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::Auxiliary(auxiliary_key)), None) => {
        MissileLoadoutComponent::Auxiliary(auxiliary_key)
      },
      (Some(MissileLoadoutComponentKey::Avionics(AvionicsKey::DirectGuidance)), Some(MissileComponentSettings::DirectGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control })) => {
        MissileLoadoutComponent::Avionics(AvionicsConfigured::DirectGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control })
      },
      (Some(MissileLoadoutComponentKey::Avionics(AvionicsKey::CruiseGuidance)), Some(MissileComponentSettings::CruiseGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine })) => {
        MissileLoadoutComponent::Avionics(AvionicsConfigured::CruiseGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine })
      },
      (Some(MissileLoadoutComponentKey::Warhead(warhead_key)), None) => {
        MissileLoadoutComponent::Warhead(warhead_key)
      },
      (None, Some(MissileComponentSettings::MissileEngineSettings { balance_values })) => {
        MissileLoadoutComponent::Engine(balance_values)
      },
      _ => return Err(MissileLoadoutError::InvalidMissileComponent)
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeekerWithMode {
  Targeting {
    seeker: SeekerConfigured
  },
  Validation {
    seeker: SeekerConfigured,
    reject_unvalidated: bool
  }
}

impl SeekerWithMode {
  pub const fn new(seeker: SeekerConfigured, mode: SeekerMode, reject_unvalidated: bool) -> Self {
    match mode {
      SeekerMode::Targeting => SeekerWithMode::Targeting { seeker },
      SeekerMode::Validation => SeekerWithMode::Validation { seeker, reject_unvalidated }
    }
  }

  pub const fn into_parts(self) -> (SeekerConfigured, SeekerMode, bool) {
    match self {
      Self::Targeting { seeker } => {
        (seeker, SeekerMode::Targeting, false)
      },
      Self::Validation { seeker, reject_unvalidated } => {
        (seeker, SeekerMode::Validation, reject_unvalidated)
      }
    }
  }

  pub const fn seeker(self) -> SeekerConfigured {
    match self {
      Self::Targeting { seeker } | Self::Validation { seeker, .. } => seeker
    }
  }

  pub const fn mode(self) -> SeekerMode {
    match self {
      Self::Targeting { .. } => SeekerMode::Targeting,
      Self::Validation { .. } => SeekerMode::Validation
    }
  }

  pub const fn into_missile_component(self) -> MissileComponent {
    let (seeker, mode, reject_unvalidated) = self.into_parts();
    let (component_key, settings) = match seeker {
      SeekerConfigured::Command => {
        (MissileComponentKey::CommandReceiver, MissileComponentSettings::CommandSeekerSettings { mode })
      },
      SeekerConfigured::FixedActiveRadar { detect_pd_targets } => {
        (MissileComponentKey::FixedActiveRadarSeeker, MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      },
      SeekerConfigured::SteerableActiveRadar { detect_pd_targets } => {
        (MissileComponentKey::SteerableActiveRadarSeeker, MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      },
      SeekerConfigured::SteerableExtendedActiveRadar { detect_pd_targets } => {
        (MissileComponentKey::SteerableExtendedActiveRadarSeeker, MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      },
      SeekerConfigured::FixedSemiActiveRadar { detect_pd_targets } => {
        (MissileComponentKey::FixedSemiActiveRadarSeeker, MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      },
      SeekerConfigured::FixedAntiRadiation => {
        (MissileComponentKey::FixedAntiRadiationSeeker, MissileComponentSettings::PassiveARHSeekerSettings { mode, reject_unvalidated, home_on_jam: false })
      },
      SeekerConfigured::FixedHomeOnJam => {
        (MissileComponentKey::FixedAntiRadiationSeeker, MissileComponentSettings::PassiveARHSeekerSettings { mode, reject_unvalidated, home_on_jam: true })
      },
      SeekerConfigured::ElectroOptical { detect_pd_targets } => {
        (MissileComponentKey::ElectroOpticalSeeker, MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      },
      SeekerConfigured::WakeHoming { detect_pd_targets } => {
        (MissileComponentKey::WakeHomingSeeker, MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })
      }
    };

    MissileComponent {
      component_key: Some(component_key),
      settings: Some(settings)
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeekerConfigured {
  Command,
  FixedActiveRadar {
    detect_pd_targets: bool
  },
  SteerableActiveRadar {
    detect_pd_targets: bool
  },
  SteerableExtendedActiveRadar {
    detect_pd_targets: bool
  },
  FixedSemiActiveRadar {
    detect_pd_targets: bool
  },
  FixedAntiRadiation,
  FixedHomeOnJam,
  ElectroOptical {
    detect_pd_targets: bool
  },
  WakeHoming {
    detect_pd_targets: bool
  }
}

impl SeekerConfigured {
  pub const fn into_seeker_key(self) -> SeekerKey {
    match self {
      Self::Command => SeekerKey::Command,
      Self::FixedActiveRadar { .. } => SeekerKey::FixedActiveRadar,
      Self::SteerableActiveRadar { .. } => SeekerKey::SteerableActiveRadar,
      Self::SteerableExtendedActiveRadar { .. } => SeekerKey::SteerableExtendedActiveRadar,
      Self::FixedSemiActiveRadar { .. } => SeekerKey::FixedSemiActiveRadar,
      Self::FixedAntiRadiation => SeekerKey::FixedAntiRadiation,
      Self::FixedHomeOnJam => SeekerKey::FixedHomeOnJam,
      Self::ElectroOptical { .. } => SeekerKey::ElectroOptical,
      Self::WakeHoming { .. } => SeekerKey::WakeHoming,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvionicsConfigured {
  DirectGuidance {
    hot_launch: bool,
    self_destruct_on_lost: bool,
    maneuvers: Maneuvers,
    defensive_doctrine: Option<DefensiveDoctrine>,
    approach_angle_control: bool
  },
  CruiseGuidance {
    hot_launch: bool,
    self_destruct_on_lost: bool,
    maneuvers: Maneuvers,
    defensive_doctrine: Option<DefensiveDoctrine>
  }
}

impl AvionicsConfigured {
  pub const fn into_missile_component(self) -> MissileComponent {
    let (component_key, settings) = match self {
      Self::DirectGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control } => {
        (MissileComponentKey::DirectGuidance, MissileComponentSettings::DirectGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control })
      },
      Self::CruiseGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine } => {
        (MissileComponentKey::CruiseGuidance, MissileComponentSettings::CruiseGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine })
      }
    };

    MissileComponent {
      component_key: Some(component_key),
      settings: Some(settings)
    }
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MissileLoadoutComponentKey {
  CommandReceiver,
  FixedActiveRadarSeeker,
  SteerableActiveRadarSeeker,
  SteerableExtendedActiveRadarSeeker,
  FixedSemiActiveRadarSeeker,
  FixedAntiRadiationSeeker,
  ElectroOpticalSeeker,
  WakeHomingSeeker,
  Auxiliary(AuxiliaryKey),
  Avionics(AvionicsKey),
  Warhead(WarheadKey)
}

impl MissileLoadoutComponentKey {
  const fn from_missile_component_key(missile_component_key: MissileComponentKey) -> Self {
    match missile_component_key {
      MissileComponentKey::CommandReceiver => Self::CommandReceiver,
      MissileComponentKey::FixedActiveRadarSeeker => Self::FixedActiveRadarSeeker,
      MissileComponentKey::SteerableActiveRadarSeeker => Self::SteerableActiveRadarSeeker,
      MissileComponentKey::SteerableExtendedActiveRadarSeeker => Self::SteerableExtendedActiveRadarSeeker,
      MissileComponentKey::FixedSemiActiveRadarSeeker => Self::FixedSemiActiveRadarSeeker,
      MissileComponentKey::FixedAntiRadiationSeeker => Self::FixedAntiRadiationSeeker,
      MissileComponentKey::ElectroOpticalSeeker => Self::ElectroOpticalSeeker,
      MissileComponentKey::WakeHomingSeeker => Self::WakeHomingSeeker,
      MissileComponentKey::ColdGasBottle => Self::Auxiliary(AuxiliaryKey::ColdGasBottle),
      MissileComponentKey::DecoyLauncher => Self::Auxiliary(AuxiliaryKey::DecoyLauncher),
      MissileComponentKey::ClusterDecoyLauncher => Self::Auxiliary(AuxiliaryKey::ClusterDecoyLauncher),
      MissileComponentKey::FastStartupModule => Self::Auxiliary(AuxiliaryKey::FastStartupModule),
      MissileComponentKey::HardenedSkin => Self::Auxiliary(AuxiliaryKey::HardenedSkin),
      MissileComponentKey::RadarAbsorbentCoating => Self::Auxiliary(AuxiliaryKey::RadarAbsorbentCoating),
      MissileComponentKey::SelfScreeningJammer => Self::Auxiliary(AuxiliaryKey::SelfScreeningJammer),
      MissileComponentKey::BoostedSelfScreeningJammer => Self::Auxiliary(AuxiliaryKey::BoostedSelfScreeningJammer),
      MissileComponentKey::DirectGuidance => Self::Avionics(AvionicsKey::DirectGuidance),
      MissileComponentKey::CruiseGuidance => Self::Avionics(AvionicsKey::CruiseGuidance),
      MissileComponentKey::HEImpact => Self::Warhead(WarheadKey::HEImpact),
      MissileComponentKey::HEKineticPenetrator => Self::Warhead(WarheadKey::HEKineticPenetrator),
      MissileComponentKey::BlastFragmentation => Self::Warhead(WarheadKey::BlastFragmentation),
      MissileComponentKey::BlastFragmentationEL => Self::Warhead(WarheadKey::BlastFragmentationEL)
    }
  }
}
