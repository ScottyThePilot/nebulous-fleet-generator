use crate::data::components::{ComponentKey, ComponentVariant, SigType};
use crate::data::hulls::HullKey;
use crate::data::hulls::config::Variant;
use crate::data::missiles::engines::EngineSettings;
use crate::data::missiles::{AuxiliaryKey, AvionicsKey, Maneuvers, WarheadKey};
use crate::data::missiles::seekers::{SeekerKey, SeekerKind, SeekerMode, SeekerStrategy};
use crate::data::missiles::bodies::{MissileBodyKey, MissileComponentsMask};
use crate::data::munitions::{MunitionFamily, MunitionKey, WeaponRole};
use crate::data::MissileSize;
use crate::format::*;
use crate::format::key::Key;

use itertools::Either;
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
  #[error("missing primary seeker")]
  MissingPrimarySeeker,
  #[error("missing avionics")]
  MissingAvionics,
  #[error("missing payload slot")]
  MissingPayloadSlot,
  #[error("too many components")]
  TooManyComponents,
  #[error("invalid engine(s)")]
  InvalidEngines,
  #[error("invalid missile component")]
  InvalidMissileComponent
}

#[derive(Debug, Clone, PartialEq)]
pub struct MissileLoadout {
  pub body_key: MissileBodyKey,
  pub seekers: SeekerLoadout,
  pub slots: Box<[MissileSlot]>,
  pub avionics: AvionicsLoadout,
  pub payload: (Option<MissileSlot>, zsize),
  pub engines: EngineLoadout
}

impl MissileLoadout {
  pub fn from_missile_template(missile_template: &MissileTemplate) -> Result<Self, MissileLoadoutError> {
    Self::new(missile_template.body_key, &missile_template.sockets)
  }

  pub fn new(body_key: MissileBodyKey, missile_sockets: &[MissileSocket]) -> Result<Self, MissileLoadoutError> {
    let mut seekers = Vec::new();
    let mut avionics = None;
    let mut slots = Vec::new();
    let mut engines = Vec::new();

    for &MissileSocket { size, installed_component } in missile_sockets {
      if let Some(installed_component) = installed_component {
        let installed_component = MissileLoadoutComponent::from_missile_component(installed_component)
          .ok_or(MissileLoadoutError::InvalidMissileComponent)?;

        match installed_component {
          MissileLoadoutComponent::Seeker(seeker_loadout) => seekers.push(seeker_loadout),
          MissileLoadoutComponent::Avionics(avionics_loadout) => avionics = Some(avionics_loadout),
          MissileLoadoutComponent::Auxiliary(auxiliary_key) => slots.push((Some(MissileSlot::Auxiliary(auxiliary_key)), size)),
          MissileLoadoutComponent::Warhead(warhead_key) => slots.push((Some(MissileSlot::Warhead(warhead_key)), size)),
          MissileLoadoutComponent::Engine(engine_settings) => engines.push((engine_settings, size))
        };
      } else {
        slots.push((None, size));
      };
    };

    let avionics = avionics.ok_or(MissileLoadoutError::MissingAvionics)?;
    let seekers = SeekerLoadout::from_seeker_list(seekers)
      .ok_or(MissileLoadoutError::MissingPrimarySeeker)?;
    let engines = EngineLoadout::from_engine_list(engines)
      .ok_or(MissileLoadoutError::InvalidEngines)?;
    let payload = slots.pop()
      .ok_or(MissileLoadoutError::MissingPayloadSlot)?;
    let slots = slots.into_iter().flat_map(|slot| slot.0)
      .collect::<Box<[MissileSlot]>>();

    Ok(MissileLoadout {
      body_key,
      seekers,
      slots,
      avionics,
      payload,
      engines
    })
  }

  pub fn to_missile_template(&self, additional: MissileTemplateAdditional) -> Result<MissileTemplate, MissileLoadoutError> {
    let variant = self.body_key.missile_body().variant;
    let socket_masks = variant.missile_components_masks();
    let mut sockets = vec![(None, zsize!(1)); variant.len().get()];

    let mut sockets_view = SocketsView {
      sockets: &mut sockets,
      socket_masks: &socket_masks
    };

    for seeker_secondary in self.seekers.iter() {
      sockets_view.put_front(seeker_secondary.into_missile_component(), zsize!(1));
    };

    //for (engine_settings, size) in self.engines.iter() {
    //  let (socket, socket_size, mask) = sockets_view.split_last()
    //    .ok_or(MissileLoadoutError::TooManyComponents)?;
    //  *socket = Some(engine_settings.into_missile_component());
    //  *socket_size = size;
    //};

    todo!()
  }
}

struct SocketsView<'a> {
  sockets: &'a mut [(Option<MissileComponent>, zsize)],
  socket_masks: &'a [MissileComponentsMask]
}

impl<'a> SocketsView<'a> {
  fn split_first<'s: 'a>(&'s mut self) -> Option<(&'a mut Option<MissileComponent>, &'a mut zsize, MissileComponentsMask)> {
    let ((socket_missile_component, socket_size), new_sockets) = self.sockets.split_first_mut()?;
    let (socket_mask, new_socket_masks) = self.socket_masks.split_first()?;
    self.sockets = new_sockets;
    self.socket_masks = new_socket_masks;
    Some((socket_missile_component, socket_size, *socket_mask))
  }

  fn split_last<'s>(&'s mut self) -> Option<(&'a mut Option<MissileComponent>, &'a mut zsize, MissileComponentsMask)> {
    let ((socket_missile_component, socket_size), new_sockets) = self.sockets.split_last_mut()?;
    let (socket_mask, new_socket_masks) = self.socket_masks.split_last()?;
    self.sockets = new_sockets;
    self.socket_masks = new_socket_masks;
    Some((socket_missile_component, socket_size, *socket_mask))
  }

  fn put_front(&mut self, missile_component: MissileComponent, size: zsize) -> Option<()> {
    self.split_first().map(move |(socket, socket_size, _)| {
      *socket = Some(missile_component);
      *socket_size = size;
    })
  }

  fn put_back(&mut self, missile_component: MissileComponent, size: zsize) -> Option<()> {
    self.split_last().map(|(socket, socket_size, _)| {
      *socket = Some(missile_component);
      *socket_size = size;
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MissileSlot {
  Auxiliary(AuxiliaryKey),
  Warhead(WarheadKey)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeekerLoadout {
  pub primary: SeekerConfigured,
  pub secondaries: Box<[SeekerSecondary]>
}

impl SeekerLoadout {
  pub fn from_seeker_list(list: impl IntoIterator<Item = SeekerSecondary>) -> Option<Self> {
    let mut list = list.into_iter();
    if let Some(SeekerSecondary::Targeting { seeker }) = list.next() {
      Some(SeekerLoadout { primary: seeker, secondaries: list.collect() })
    } else {
      None
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = SeekerSecondary> + '_ {
    std::iter::once(SeekerSecondary::Targeting { seeker: self.primary })
      .chain(self.secondaries.iter().copied())
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeekerSecondary {
  Targeting {
    seeker: SeekerConfigured
  },
  Validating {
    seeker: SeekerConfigured,
    reject_unvalidated: bool
  }
}

impl SeekerSecondary {
  pub const fn new(seeker: SeekerConfigured, mode: SeekerMode, reject_unvalidated: bool) -> Self {
    match mode {
      SeekerMode::Targeting => SeekerSecondary::Targeting { seeker },
      SeekerMode::Validation => SeekerSecondary::Validating { seeker, reject_unvalidated }
    }
  }

  pub const fn into_parts(self) -> (SeekerConfigured, SeekerMode, bool) {
    match self {
      Self::Targeting { seeker } => {
        (seeker, SeekerMode::Targeting, false)
      },
      Self::Validating { seeker, reject_unvalidated } => {
        (seeker, SeekerMode::Validation, reject_unvalidated)
      }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvionicsLoadout {
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

impl AvionicsLoadout {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineLoadout {
  Conventional {
    engine_settings: EngineSettings,
    engine_size: zsize
  },
  Hybrid {
    cruise_engine_settings: EngineSettings,
    sprint_engine_settings: EngineSettings,
    sprint_engine_length: zsize
  }
}

impl EngineLoadout {
  pub fn from_engine_list(list: Vec<(EngineSettings, zsize)>) -> Option<Self> {
    match <[_; 2]>::try_from(list) {
      Ok([(sprint_engine_settings, sprint_engine_length), (cruise_engine_settings, ..)]) => {
        Some(EngineLoadout::Hybrid { cruise_engine_settings, sprint_engine_settings, sprint_engine_length })
      },
      Err(list) => match <[_; 1]>::try_from(list) {
        Ok([(engine_settings, engine_size)]) => {
          Some(EngineLoadout::Conventional { engine_settings, engine_size })
        },
        Err(..) => None
      }
    }
  }

  pub fn iter(self) -> impl Iterator<Item = (EngineSettings, zsize)> {
    match self {
      Self::Conventional { engine_settings, engine_size } => {
        Either::Left([
          (engine_settings, engine_size)
        ].into_iter())
      },
      Self::Hybrid { cruise_engine_settings, sprint_engine_settings, sprint_engine_length } => {
        Either::Right([
          (cruise_engine_settings, zsize!(1)),
          (sprint_engine_settings, sprint_engine_length)
        ].into_iter())
      }
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum MissileLoadoutComponent {
  Seeker(SeekerSecondary),
  Avionics(AvionicsLoadout),
  Auxiliary(AuxiliaryKey),
  Warhead(WarheadKey),
  Engine(EngineSettings)
}

impl MissileLoadoutComponent {
  fn from_missile_component(missile_component: MissileComponent) -> Option<Self> {
    let MissileComponent { component_key, settings } = missile_component;
    let component_key = component_key.map(MissileLoadoutComponentKey::from_missile_component_key);

    Some(match (component_key, settings) {
      (Some(MissileLoadoutComponentKey::CommandReceiver), Some(MissileComponentSettings::CommandSeekerSettings { mode })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::Command, mode, false))
      },
      (Some(MissileLoadoutComponentKey::FixedActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::FixedActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::SteerableActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::SteerableActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::SteerableExtendedActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::SteerableExtendedActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::FixedSemiActiveRadarSeeker), Some(MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::FixedSemiActiveRadar { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::FixedAntiRadiationSeeker), Some(MissileComponentSettings::PassiveARHSeekerSettings { mode, reject_unvalidated, home_on_jam })) => {
        let seeker_configured = if home_on_jam { SeekerConfigured::FixedHomeOnJam } else { SeekerConfigured::FixedAntiRadiation };
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(seeker_configured, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::ElectroOpticalSeeker), Some(MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::ElectroOptical { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::WakeHomingSeeker), Some(MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets })) => {
        MissileLoadoutComponent::Seeker(SeekerSecondary::new(SeekerConfigured::WakeHoming { detect_pd_targets }, mode, reject_unvalidated))
      },
      (Some(MissileLoadoutComponentKey::Auxiliary(auxiliary_key)), None) => {
        MissileLoadoutComponent::Auxiliary(auxiliary_key)
      },
      (Some(MissileLoadoutComponentKey::Avionics(AvionicsKey::DirectGuidance)), Some(MissileComponentSettings::DirectGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control })) => {
        MissileLoadoutComponent::Avionics(AvionicsLoadout::DirectGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control })
      },
      (Some(MissileLoadoutComponentKey::Avionics(AvionicsKey::CruiseGuidance)), Some(MissileComponentSettings::CruiseGuidanceSettings { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine })) => {
        MissileLoadoutComponent::Avionics(AvionicsLoadout::CruiseGuidance { hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine })
      },
      (Some(MissileLoadoutComponentKey::Warhead(warhead_key)), None) => {
        MissileLoadoutComponent::Warhead(warhead_key)
      },
      (None, Some(MissileComponentSettings::MissileEngineSettings { balance_values })) => {
        MissileLoadoutComponent::Engine(balance_values)
      },
      _ => return None
    })
  }
}
