use super::hulls::HullKey;
use super::munitions::{MunitionFamily, WeaponRole};
use super::{Buff, Faction, MissileSize};
use crate::utils::Size;

use bytemuck::Contiguous;

use std::fmt;
use std::num::NonZeroUsize as zsize;
use std::str::FromStr;



#[derive(Debug, Clone, Copy)]
pub struct Component {
  pub name: &'static str,
  pub save_key: &'static str,
  pub kind: ComponentKind,
  pub variant: Option<ComponentVariant>,
  pub faction: Option<Faction>,
  pub compounding_multiplier: Option<f32>,
  pub first_instance_free: bool,
  pub point_cost: usize,
  pub mass: f32,
  pub size: Size,
  pub power: isize,
  /// Crew is tileable when the variant is `Berthing`.
  pub crew: isize,
  pub max_health: f32,
  pub reinforced: bool,
  pub buffs: &'static [(Buff, f32)]
}

impl Component {
  /// Whether or not this component is legal to use on the given hull.
  pub const fn is_usable_on(self, hull: HullKey) -> bool {
    match hull {
      HullKey::OcelloCommandCruiser => true,
      hull => self.is_usable_by(hull.faction())
    }
  }

  /// Whether or not this component is legal to use by the given faction *in general*.
  pub const fn is_usable_by(self, faction: Faction) -> bool {
    match self.faction {
      Some(Faction::Alliance) => matches!(faction, Faction::Alliance),
      Some(Faction::Protectorate) => matches!(faction, Faction::Protectorate),
      None => true
    }
  }

  /// Whether this component tiles. This does not include missile launcher tiling.
  pub const fn can_tile(self) -> bool {
    matches!(self.variant, Some(ComponentVariant::Berthing | ComponentVariant::Magazine { .. }))
  }

  /// Whether or not this component can fit inside a socket of the given size.
  pub const fn can_fit_in(self, socket_size: Size) -> bool {
    can_fit_in(socket_size, self.size)
  }

  /// The number of times this component could fit inside a socket of the given size.
  ///
  /// If this component cannot fit inside a socket of the given size, this returns `0`.
  /// Otherwise, tiling components may return `1` or more, while non-tiling components will only return `1`.
  ///
  /// Missile launchers are always considered to be non-tiling to this function.
  pub const fn tiling_quantity(self, socket_size: Size) -> usize {
    if self.can_tile() {
      if self.can_fit_in(socket_size) { 1 } else { 0 }
    } else {
      tiling_quantity(socket_size, self.size)
    }
  }

  /// Calculates how undersized a given component is (in volume)
  /// were this component to be placed inside a socket of the given size.
  /// This includes missile launcher tiling.
  ///
  /// Returns `None` if this component cannot fit inside of it.
  pub fn undersize_volume_within(self, socket_size: Size) -> Option<usize> {
    if self.can_fit_in(socket_size) {
      let tiled_size = socket_size.div(self.size).mul(self.size);
      let tiling_undersize = socket_size.volume() - tiled_size.volume();
      let nominal_undersize = Size::sub(socket_size, self.size).volume();
      match self.variant {
        Some(ComponentVariant::Berthing) => Some(tiling_undersize),
        Some(ComponentVariant::Magazine { .. }) => Some(tiling_undersize),
        Some(ComponentVariant::WeaponMissileBank { cells, .. }) => match cells {
          MissileLauncherCells::Constant { .. } => Some(nominal_undersize),
          MissileLauncherCells::Tiling { .. } => Some(tiling_undersize),
          MissileLauncherCells::Function { groups, .. } => groups(socket_size).map(|_| 0)
        },
        Some(..) | None => Some(nominal_undersize)
      }
    } else {
      None
    }
  }

  /// The number of crew consumed or contributed by this component.
  pub const fn crew(self, socket_size: Size) -> isize {
    let component = self;
    if let Some(ComponentVariant::Berthing) = component.variant {
      component.crew * tiling_quantity(socket_size, self.size) as isize
    } else {
      component.crew
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentKind {
  Mount, Compartment, Module
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentVariant {
  /// An antenna.
  Antenna {
    transmit_power: f32
  },
  /// A berthing.
  Berthing,
  /// A command compartment, such as a CIC.
  Command {
    work_on_remote_tracks: bool,
    intel_effort: f32,
    intel_accuracy: f32
  },
  /// A damage-control compartment.
  DamageControl {
    teams: usize,
    repair_speed: f32,
    movement_speed: f32,
    restores: usize
  },
  /// A fire-control radar.
  FireControl { fire_control: FireControl },
  /// An illuminator for guiding semi-active missiles.
  Illuminator {
    max_range: f32,
    battleshort_available: bool,
    burst_duration: f32,
    cooldown_time: f32,
    cone_fov: f32
  },
  /// An intelligence compartment.
  /// This differs from command compartments in that they cannot control the ship.
  Intelligence {
    work_on_remote_tracks: bool,
    intel_effort: f32,
    intel_accuracy: f32
  },
  /// An electronic warfare jammer.
  Jammer {
    sig_type: SigType,
    max_range: f32,
    battleshort_available: bool,
    burst_duration: f32,
    cooldown_time: f32,
    // If none, this is an omnidirectional jammer.
    cone_fov: Option<f32>
  },
  /// A magazine.
  Magazine {
    available_volume: usize
  },
  /// An electronic warfare sensor component, such as a search radar.
  Sensor {
    sig_type: SigType,
    max_range: f32,
    can_lock: bool,
    can_burnthrough: bool,
    /// If none, this is an omnidirectional sensor.
    cone_fov: Option<f32>
  },
  /// A passive sensor.
  /// The Pinard is the only component of this type that exists.
  SensorPassive,
  /// A weapon that can fire without consuming ammunition.
  /// These can be fixed or turreted.
  WeaponBeam {
    is_fixed: bool,
    is_energy: bool,
    integrated_fire_control: Option<FireControl>,
    optical_backup: bool,
    role: WeaponRole,
    battleshort_available: bool,
    burst_duration: f32,
    cooldown_time: f32
  },
  /// A missile launcher that is reloadable from external magazines.
  WeaponMissileLauncher {
    integrated_fire_control: Option<FireControl>,
    optical_backup: bool,
    role: WeaponRole,
    munition_family: MunitionFamily,
    missile_size: MissileSize,
    load_time: f32
  },
  /// A missile launcher that uses a non-reloadable, integrated magazine bank.
  WeaponMissileBank {
    is_fixed: bool,
    integrated_fire_control: Option<FireControl>,
    optical_backup: bool,
    role: WeaponRole,
    munition_family: MunitionFamily,
    missile_size: MissileSize,
    /// Cells can be tileable.
    cells: MissileLauncherCells
  },
  /// A weapon that fires projectiles and requires ammunition to operate.
  /// These can be fixed or turreted.
  WeaponProjectile {
    is_fixed: bool,
    is_energy: bool,
    integrated_fire_control: Option<FireControl>,
    optical_backup: bool,
    role: WeaponRole,
    munition_family: MunitionFamily,
    reload_time: f32,
    autoloader: Option<Autoloader>
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Autoloader {
  pub capacity: zsize,
  pub recycle_time: f32
}

/// Returns fire rate in rounds per second. Multiply by 60 to get rounds per minute.
pub fn fire_rate(reload_time: f32, autoloader: Option<Autoloader>) -> f32 {
  f32::recip(if let Some(autoloader) = autoloader {
    let capacity = autoloader.capacity.get();
    ((capacity - 1) as f32 * autoloader.recycle_time + reload_time) / capacity as f32
  } else {
    reload_time
  })
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SigType {
  ElectroOptical,
  Radar,
  Comms
}

#[derive(Debug, Clone, Copy)]
pub struct FireControl {
  pub sig_type: SigType,
  pub max_range: f32
}

#[derive(Debug, Clone, Copy)]
pub enum MissileLauncherCells {
  /// Missile launcher has a fixed quantity of cells.
  Constant {
    count: usize
  },
  /// Missile launcher will tile in the x and z directions based on its base size.
  Tiling {
    count_per_group: usize,
    /// Whether each tiled group of cells must contain the same missile type.
    separated_groups: bool
  },
  /// Missile launcher's size is determined by a function.
  Function {
    count_per_group: usize,
    groups: fn(Size) -> Option<[usize; 2]>
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
pub enum ComponentKey {
  ActivelyCooledAmplifiers,
  AdaptiveRadarReceiver,
  AmmunitionElevators,
  AnalysisAnnex,
  AuxiliarySteering,
  BW1500Drive,
  BW1500RDrive,
  BW2000Drive,
  BW800Drive,
  BW800RDrive,
  BasicCIC,
  BattleDressingStation,
  Berthing,
  BoostedReactor,
  BulkMagazine,
  BulwarkHuntress,
  C30Cannon,
  C53Cannon,
  C56Cannon,
  C65Cannon,
  C81PlasmaCannon,
  C90Cannon,
  CHI7700Drive,
  CHI777YardDrive,
  CHI9100LongHaulDrive,
  CLS3Launcher,
  CR10Antenna,
  CR70Antenna,
  CitadelCIC,
  CitadelMagazine,
  CivilianReactor,
  ContainerBankLauncher,
  ContainerStackLauncher,
  DamageControlCentral,
  DamageControlComplex,
  E15MasqueradeDeceptionModule,
  E20LighthouseIlluminator,
  E55SpotlightIlluminator,
  E57FloodlightIlluminator,
  E70InterruptionJammer,
  E71HangupJammer,
  E90BlanketJammer,
  ES22PinardElectronicSupportModule,
  ES32ScryerMissileIDSystem,
  EnergyRegulator,
  FM200Drive,
  FM200RDrive,
  FM230WhiplashDrive,
  FM240DragonflyDrive,
  FM280RaiderDrive,
  FM30XProwlerDrive,
  FM500Drive,
  FM500RDrive,
  FM530WhiplashDrive,
  FM540DragonflyDrive,
  FM580RaiderDrive,
  FR3300MicroReactor,
  FR4800Reactor,
  FocusedParticleAccelerator,
  GunPlottingCenter,
  IntelligenceCenter,
  IthacaBridgemaster,
  J15BellbirdJammer,
  J360LyrebirdJammer,
  JuryRiggedReactor,
  L50BlackjackLaserDazzler,
  LargeDCLocker,
  LargeDCStorage,
  LauncherDelugeSystem,
  LightCivilianReactor,
  ML9MineLauncher,
  MLS2Launcher,
  MLS3Launcher,
  MagazineSprinklers,
  MissileParallelInterface,
  MissileProgrammingBus,
  MissileProgrammingBusArray,
  Mk20DefenderPDT,
  Mk25ReboundPDT,
  Mk29StonewallPDT,
  Mk550Railgun,
  Mk600BeamCannon,
  Mk610BeamTurret,
  Mk61Cannon,
  Mk62Cannon,
  Mk64Cannon,
  Mk65Cannon,
  Mk66Cannon,
  Mk68Cannon,
  Mk81Railgun,
  Mk90AuroraPDT,
  Mk95SarissaPDT,
  MountGyros,
  P11PavisePDT,
  P20BastionPDT,
  P60GrazerPDT,
  PlantControlCenter,
  R400BloodhoundLRTRadar,
  R550EarlyWarningRadar,
  RF101BullseyeRadar,
  RF44PinpointRadar,
  RL18Launcher,
  RL36Launcher,
  RM50ParallaxRadar,
  RS35FrontlineRadar,
  RS41SpyglassRadar,
  RapidCycleCradle,
  RapidDCLocker,
  RedundantReactorFailsafes,
  ReinforcedCIC,
  ReinforcedDCLocker,
  ReinforcedMagazine,
  ReinforcedThrusterNozzles,
  SignatureScrambler,
  SmallDCLocker,
  SmallEnergyRegulator,
  SmallReactorBooster,
  SmallWorkshop,
  StrikePlanningCenter,
  StrobeCorrelator,
  SundriveRacingPro,
  SupplementaryRadioAmplifiers,
  T20Cannon,
  T30Cannon,
  T81PlasmaCannon,
  TE45MassDriver,
  TLS3Launcher,
  TrackCorrelator,
  VLS123Launcher,
  VLS146Launcher,
  VLS2Launcher,
  VLS3Launcher
}

impl ComponentKey {
  pub const fn save_key(self) -> &'static str {
    self.component().save_key
  }

  pub const fn component(self) -> &'static Component {
    use self::list::*;

    match self {
      Self::ActivelyCooledAmplifiers => &ACTIVELY_COOLED_AMPLIFIERS,
      Self::AdaptiveRadarReceiver => &ADAPTIVE_RADAR_RECEIVER,
      Self::AmmunitionElevators => &AMMUNITION_ELEVATORS,
      Self::AnalysisAnnex => &ANALYSIS_ANNEX,
      Self::AuxiliarySteering => &AUXILIARY_STEERING,
      Self::BW1500Drive => &BW1500_DRIVE,
      Self::BW1500RDrive => &BW1500R_DRIVE,
      Self::BW2000Drive => &BW2000_DRIVE,
      Self::BW800Drive => &BW800_DRIVE,
      Self::BW800RDrive => &BW800R_DRIVE,
      Self::BasicCIC => &BASIC_CIC,
      Self::BattleDressingStation => &BATTLE_DRESSING_STATION,
      Self::Berthing => &BERTHING,
      Self::BoostedReactor => &BOOSTED_REACTOR,
      Self::BulkMagazine => &BULK_MAGAZINE,
      Self::BulwarkHuntress => &BULWARK_HUNTRESS,
      Self::C30Cannon => &C30_CANNON,
      Self::C53Cannon => &C53_CANNON,
      Self::C56Cannon => &C56_CANNON,
      Self::C65Cannon => &C65_CANNON,
      Self::C81PlasmaCannon => &C81_PLASMA_CANNON,
      Self::C90Cannon => &C90_CANNON,
      Self::CHI7700Drive => &CHI7700_DRIVE,
      Self::CHI777YardDrive => &CHI777_YARD_DRIVE,
      Self::CHI9100LongHaulDrive => &CHI9100_LONG_HAUL_DRIVE,
      Self::CLS3Launcher => &CLS3_LAUNCHER,
      Self::CR10Antenna => &CR10_ANTENNA,
      Self::CR70Antenna => &CR70_ANTENNA,
      Self::CitadelCIC => &CITADEL_CIC,
      Self::CitadelMagazine => &CITADEL_MAGAZINE,
      Self::CivilianReactor => &CIVILIAN_REACTOR,
      Self::ContainerBankLauncher => &CONTAINER_BANK_LAUNCHER,
      Self::ContainerStackLauncher => &CONTAINER_STACK_LAUNCHER,
      Self::DamageControlCentral => &DAMAGE_CONTROL_CENTRAL,
      Self::DamageControlComplex => &DAMAGE_CONTROL_COMPLEX,
      Self::E15MasqueradeDeceptionModule => &E15_MASQUERADE_DECEPTION_MODULE,
      Self::E20LighthouseIlluminator => &E20_LIGHTHOUSE_ILLUMINATOR,
      Self::E55SpotlightIlluminator => &E55_SPOTLIGHT_ILLUMINATOR,
      Self::E57FloodlightIlluminator => &E57_FLOODLIGHT_ILLUMINATOR,
      Self::E70InterruptionJammer => &E70_INTERRUPTION_JAMMER,
      Self::E71HangupJammer => &E71_HANGUP_JAMMER,
      Self::E90BlanketJammer => &E90_BLANKET_JAMMER,
      Self::ES22PinardElectronicSupportModule => &ES22_PINARD_ELECTRONIC_SUPPORT_MODULE,
      Self::ES32ScryerMissileIDSystem => &ES32_SCRYER_MISSILE_ID_SYSTEM,
      Self::EnergyRegulator => &ENERGY_REGULATOR,
      Self::FM200Drive => &FM200_DRIVE,
      Self::FM200RDrive => &FM200R_DRIVE,
      Self::FM230WhiplashDrive => &FM230_WHIPLASH_DRIVE,
      Self::FM240DragonflyDrive => &FM240_DRAGONFLY_DRIVE,
      Self::FM280RaiderDrive => &FM280_RAIDER_DRIVE,
      Self::FM30XProwlerDrive => &FM30X_PROWLER_DRIVE,
      Self::FM500Drive => &FM500_DRIVE,
      Self::FM500RDrive => &FM500R_DRIVE,
      Self::FM530WhiplashDrive => &FM530_WHIPLASH_DRIVE,
      Self::FM540DragonflyDrive => &FM540_DRAGONFLY_DRIVE,
      Self::FM580RaiderDrive => &FM580_RAIDER_DRIVE,
      Self::FR3300MicroReactor => &FR3300_MICRO_REACTOR,
      Self::FR4800Reactor => &FR4800_REACTOR,
      Self::FocusedParticleAccelerator => &FOCUSED_PARTICLE_ACCELERATOR,
      Self::GunPlottingCenter => &GUN_PLOTTING_CENTER,
      Self::IntelligenceCenter => &INTELLIGENCE_CENTER,
      Self::IthacaBridgemaster => &ITHACA_BRIDGEMASTER,
      Self::J15BellbirdJammer => &J15_BELLBIRD_JAMMER,
      Self::J360LyrebirdJammer => &J360_LYREBIRD_JAMMER,
      Self::JuryRiggedReactor => &JURYRIGGED_REACTOR,
      Self::L50BlackjackLaserDazzler => &L50_BLACKJACK_LASER_DAZZLER,
      Self::LargeDCLocker => &LARGE_DC_LOCKER,
      Self::LargeDCStorage => &LARGE_DC_STORAGE,
      Self::LauncherDelugeSystem => &LAUNCHER_DELUGE_SYSTEM,
      Self::LightCivilianReactor => &LIGHT_CIVILIAN_REACTOR,
      Self::ML9MineLauncher => &ML9_MINE_LAUNCHER,
      Self::MLS2Launcher => &MLS2_LAUNCHER,
      Self::MLS3Launcher => &MLS3_LAUNCHER,
      Self::MagazineSprinklers => &MAGAZINE_SPRINKLERS,
      Self::MissileParallelInterface => &MISSILE_PARALLEL_INTERFACE,
      Self::MissileProgrammingBus => &MISSILE_PROGRAMMING_BUS,
      Self::MissileProgrammingBusArray => &MISSILE_PROGRAMMING_BUS_ARRAY,
      Self::Mk20DefenderPDT => &MK20_DEFENDER_PDT,
      Self::Mk25ReboundPDT => &MK25_REBOUND_PDT,
      Self::Mk29StonewallPDT => &MK29_STONEWALL_PDT,
      Self::Mk550Railgun => &MK550_RAILGUN,
      Self::Mk600BeamCannon => &MK600_BEAM_CANNON,
      Self::Mk610BeamTurret => &MK610_BEAM_TURRET,
      Self::Mk61Cannon => &MK61_CANNON,
      Self::Mk62Cannon => &MK62_CANNON,
      Self::Mk64Cannon => &MK64_CANNON,
      Self::Mk65Cannon => &MK65_CANNON,
      Self::Mk66Cannon => &MK66_CANNON,
      Self::Mk68Cannon => &MK68_CANNON,
      Self::Mk81Railgun => &MK81_RAILGUN,
      Self::Mk90AuroraPDT => &MK90_AURORA_PDT,
      Self::Mk95SarissaPDT => &MK95_SARISSA_PDT,
      Self::MountGyros => &MOUNT_GYROS,
      Self::P11PavisePDT => &P11_PAVISE_PDT,
      Self::P20BastionPDT => &P20_BASTION_PDT,
      Self::P60GrazerPDT => &P60_GRAZER_PDT,
      Self::PlantControlCenter => &PLANT_CONTROL_CENTER,
      Self::R400BloodhoundLRTRadar => &R400_BLOODHOUND_LRT_RADAR,
      Self::R550EarlyWarningRadar => &R550_EARLY_WARNING_RADAR,
      Self::RF101BullseyeRadar => &RF101_BULLSEYE_RADAR,
      Self::RF44PinpointRadar => &RF44_PINPOINT_RADAR,
      Self::RL18Launcher => &RL18_LAUNCHER,
      Self::RL36Launcher => &RL36_LAUNCHER,
      Self::RM50ParallaxRadar => &RM50_PARALLAX_RADAR,
      Self::RS35FrontlineRadar => &RS35_FRONTLINE_RADAR,
      Self::RS41SpyglassRadar => &RS41_SPYGLASS_RADAR,
      Self::RapidCycleCradle => &RAPIDCYCLE_CRADLE,
      Self::RapidDCLocker => &RAPID_DC_LOCKER,
      Self::RedundantReactorFailsafes => &REDUNDANT_REACTOR_FAILSAFES,
      Self::ReinforcedCIC => &REINFORCED_CIC,
      Self::ReinforcedDCLocker => &REINFORCED_DC_LOCKER,
      Self::ReinforcedMagazine => &REINFORCED_MAGAZINE,
      Self::ReinforcedThrusterNozzles => &REINFORCED_THRUSTER_NOZZLES,
      Self::SignatureScrambler => &SIGNATURE_SCRAMBLER,
      Self::SmallDCLocker => &SMALL_DC_LOCKER,
      Self::SmallEnergyRegulator => &SMALL_ENERGY_REGULATOR,
      Self::SmallReactorBooster => &SMALL_REACTOR_BOOSTER,
      Self::SmallWorkshop => &SMALL_WORKSHOP,
      Self::StrikePlanningCenter => &STRIKE_PLANNING_CENTER,
      Self::StrobeCorrelator => &STROBE_CORRELATOR,
      Self::SundriveRacingPro => &SUNDRIVE_RACING_PRO,
      Self::SupplementaryRadioAmplifiers => &SUPPLEMENTARY_RADIO_AMPLIFIERS,
      Self::T20Cannon => &T20_CANNON,
      Self::T30Cannon => &T30_CANNON,
      Self::T81PlasmaCannon => &T81_PLASMA_CANNON,
      Self::TE45MassDriver => &TE45_MASS_DRIVER,
      Self::TLS3Launcher => &TLS3_LAUNCHER,
      Self::TrackCorrelator => &TRACK_CORRELATOR,
      Self::VLS123Launcher => &VLS1_23_LAUNCHER,
      Self::VLS146Launcher => &VLS1_46_LAUNCHER,
      Self::VLS2Launcher => &VLS2_LAUNCHER,
      Self::VLS3Launcher => &VLS3_LAUNCHER
    }
  }

  #[inline]
  pub const fn values() -> crate::utils::ContiguousEnumValues<Self> {
    crate::utils::ContiguousEnumValues::new()
  }
}

impl FromStr for ComponentKey {
  type Err = super::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    ComponentKey::values()
      .find(|component_key| component_key.save_key() == s)
      .ok_or(super::InvalidKey::Component)
  }
}

impl fmt::Display for ComponentKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}

pub const fn tiling_quantity(container: Size, item: Size) -> usize {
  Size::div(container, item).volume()
}

pub const fn can_fit_in(container: Size, item: Size) -> bool {
  container.x >= item.x && container.y >= item.y && container.z >= item.z
}



pub mod list {
  use super::*;

  pub const ACTIVELY_COOLED_AMPLIFIERS: Component = Component {
    name: "Actively Cooled Amplifiers",
    save_key: "Stock/Actively Cooled Amplifiers",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -175,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::BurstDurationEmitters, 0.25),
      (Buff::CooldownTimeEmitters, -0.2),
      (Buff::OverheatDamageChanceEmitters, -0.1)
    ]
  };

  pub const ADAPTIVE_RADAR_RECEIVER: Component = Component {
    name: "Adaptive Radar Receiver",
    save_key: "Stock/Adaptive Radar Receiver",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -200,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::NoiseFiltering, -0.7),
      (Buff::PositionalError, 0.05),
      (Buff::Sensitivity, 0.25),
      (Buff::VelocityError, 0.025)
    ]
  };

  pub const AMMUNITION_ELEVATORS: Component = Component {
    name: "Ammunition Elevators",
    save_key: "Stock/Ammunition Elevators",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 30.0,
    size: Size::new(2, 2, 2),
    power: -150,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::LauncherReloadTime, -0.15), (Buff::ReloadTime, -0.15)
    ]
  };

  pub const ANALYSIS_ANNEX: Component = Component {
    name: "Analysis Annex",
    save_key: "Stock/Analysis Annex",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 10.0,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -20,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::IntelligenceAccuracy, -0.3),
      (Buff::IntelligenceEffort, 0.3)
    ]
  };

  pub const AUXILIARY_STEERING: Component = Component {
    name: "Auxiliary Steering",
    save_key: "Stock/Auxiliary Steering",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Command {
      work_on_remote_tracks: false,
      intel_effort: 0.0,
      intel_accuracy: 0.25
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 10.0,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -5,
    max_health: 300.0,
    reinforced: true,
    buffs: &[]
  };

  pub const BW1500_DRIVE: Component = Component {
    name: "BW1500 Drive",
    save_key: "Stock/BW1500 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 35.0,
    size: Size::new(12, 12, 12),
    power: 300,
    crew: -10,
    max_health: 700.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BW1500R_DRIVE: Component = Component {
    name: "BW1500-R Drive",
    save_key: "Stock/BW1500-R Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(12, 12, 12),
    power: 300,
    crew: -10,
    max_health: 1400.0,
    reinforced: true,
    buffs: &[]
  };

  pub const BW2000_DRIVE: Component = Component {
    name: "BW2000 Drive",
    save_key: "Stock/BW2000 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 35.0,
    size: Size::new(12, 12, 12),
    power: 2000,
    crew: -10,
    max_health: 700.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, -0.15),
      (Buff::LinearThrust, -0.15),
      (Buff::TopSpeed, -0.1)
    ]
  };

  pub const BW800_DRIVE: Component = Component {
    name: "BW800 Drive",
    save_key: "Stock/BW800 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 300,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BW800R_DRIVE: Component = Component {
    name: "BW800-R Drive",
    save_key: "Stock/BW800-R Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 300,
    crew: -10,
    max_health: 400.0,
    reinforced: true,
    buffs: &[]
  };

  pub const BASIC_CIC: Component = Component {
    name: "Basic CIC",
    save_key: "Stock/Basic CIC",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Command {
      work_on_remote_tracks: false,
      intel_effort: 1.0,
      intel_accuracy: 0.3
    }),
    faction: None,
    compounding_multiplier: Some(1.0),
    first_instance_free: false,
    point_cost: 10,
    mass: 15.0,
    size: Size::new(4, 1, 6),
    power: 0,
    crew: -20,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BATTLE_DRESSING_STATION: Component = Component {
    name: "Battle Dressing Station",
    save_key: "Stock/Battle Dressing Station",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 10.0,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -8,
    max_health: 125.0,
    reinforced: false,
    buffs: &[
      (Buff::CrewVulnerability, -0.3)
    ]
  };

  pub const BERTHING: Component = Component {
    name: "Berthing",
    save_key: "Stock/Berthing",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Berthing),
    faction: None,
    compounding_multiplier: Some(0.0),
    first_instance_free: true,
    point_cost: 1,
    mass: 0.5,
    size: Size::new(1, 1, 1),
    power: 0,
    crew: 3,
    max_health: 50.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BOOSTED_REACTOR: Component = Component {
    name: "Boosted Reactor",
    save_key: "Stock/Boosted Reactor",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 40.0,
    size: Size::new(5, 5, 5),
    power: 5000,
    crew: -10,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BULK_MAGAZINE: Component = Component {
    name: "Bulk Magazine",
    save_key: "Stock/Bulk Magazine",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Magazine {
      available_volume: 15
    }),
    faction: None,
    compounding_multiplier: Some(0.0),
    first_instance_free: true,
    point_cost: 1,
    mass: 1.0,
    size: Size::new(1, 1, 1),
    power: 0,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const BULWARK_HUNTRESS: Component = Component {
    name: "Bulwark Huntress",
    save_key: "Stock/Bulwark Huntress",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 10000.0,
      can_lock: false,
      can_burnthrough: true,
      cone_fov: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -2000,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C30_CANNON: Component = Component {
    name: "C30 Cannon",
    save_key: "Stock/C30 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical100mm,
      reload_time: 30.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(24),
        recycle_time: 1.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 15.0,
    size: Size::new(2, 5, 2),
    power: -100,
    crew: -10,
    max_health: 350.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C53_CANNON: Component = Component {
    name: "C53 Cannon",
    save_key: "Stock/C53 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical250mm,
      reload_time: 70.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(8),
        recycle_time: 2.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 3.0,
    size: Size::new(2, 4, 2),
    power: -100,
    crew: -15,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C56_CANNON: Component = Component {
    name: "C56 Cannon",
    save_key: "Stock/C54 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical250mm,
      reload_time: 70.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(15),
        recycle_time: 2.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 3.0,
    size: Size::new(4, 6, 4),
    power: -100,
    crew: -15,
    max_health: 350.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C65_CANNON: Component = Component {
    name: "C65 Cannon",
    save_key: "Stock/C65 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical450mm,
      reload_time: 90.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(8),
        recycle_time: 4.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 35.0,
    size: Size::new(6, 10, 6),
    power: -300,
    crew: -25,
    max_health: 500.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C81_PLASMA_CANNON: Component = Component {
    name: "C81 Plasma Cannon",
    save_key: "Stock/C81 Plasma Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticMagnetic400mmPlasma,
      reload_time: 12.0,
      autoloader: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 80.0,
    size: Size::new(4, 6, 4),
    power: -1200,
    crew: -25,
    max_health: 500.0,
    reinforced: false,
    buffs: &[]
  };

  pub const C90_CANNON: Component = Component {
    name: "C90 Cannon",
    save_key: "Stock/C90 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical600mm,
      reload_time: 18.0,
      autoloader: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 50.0,
    size: Size::new(6, 12, 6),
    power: -200,
    crew: -25,
    max_health: 500.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CHI7700_DRIVE: Component = Component {
    name: "CHI-7700 Drive",
    save_key: "Stock/CHI-7700 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(12, 12, 12),
    power: 300,
    crew: -10,
    max_health: 700.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, 0.35),
      (Buff::TopSpeed, -0.15),
      (Buff::TurnRate, 0.4)
    ]
  };

  pub const CHI777_YARD_DRIVE: Component = Component {
    name: "CHI-777 Yard Drive",
    save_key: "Stock/CHI-777 Yard Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 300,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, 0.4),
      (Buff::LinearThrust, 0.1),
      (Buff::TopSpeed, -0.1),
      (Buff::TurnRate, 0.45)
    ]
  };

  pub const CHI9100_LONG_HAUL_DRIVE: Component = Component {
    name: "CHI-9100 Long Haul Drive",
    save_key: "Stock/CHI-9100 Long Haul Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(12, 12, 12),
    power: 300,
    crew: -10,
    max_health: 700.0,
    reinforced: false,
    buffs: &[
      (Buff::FlankDamageProbability, 0.5),
      (Buff::LinearThrust, 0.2),
      (Buff::TopSpeed, 0.2)
    ]
  };

  pub const CLS3_LAUNCHER: Component = Component {
    name: "CLS-3 Launcher",
    save_key: "Stock/CLS-3 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size3,
      cells: MissileLauncherCells::Function {
        count_per_group: 2,
        groups: |size| match size {
          // CLS-3 is not available for 3x4x3 mounts on the Raines or Keystone,
          // but *is* available for 3x4x3 mounts on the Axford and Solomon, not sure why.
          Size { x: 3, y, z: 5 } if y >= 4 => Some([1, 3]), // 6
          Size { x: 6, y, z: 6 } if y >= 4 => Some([2, 4]), // 16
          Size { x: 8, y, z: 8 } if y >= 4 => Some([2, 5]), // 20
          _ => None
        }
      }
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 20.0,
    size: Size::new(3, 4, 3),
    power: -200,
    crew: 0,
    max_health: 175.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CR10_ANTENNA: Component = Component {
    name: "CR10 Antenna",
    save_key: "Stock/CR10 Antenna",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Antenna {
      transmit_power: 500.0
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 5,
    mass: 4.0,
    size: Size::new(2, 1, 2),
    power: -250,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CR70_ANTENNA: Component = Component {
    name: "CR70 Antenna",
    save_key: "Stock/CR70 Antenna",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Antenna {
      transmit_power: 1250.0
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 7.0,
    size: Size::new(2, 1, 2),
    power: -600,
    crew: 0,
    max_health: 250.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CITADEL_CIC: Component = Component {
    name: "Citadel CIC",
    save_key: "Stock/Citadel CIC",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Command {
      work_on_remote_tracks: false,
      intel_effort: 4.0,
      intel_accuracy: 0.25
    }),
    faction: None,
    compounding_multiplier: Some(1.0),
    first_instance_free: false,
    point_cost: 75,
    mass: 70.0,
    size: Size::new(6, 1, 8),
    power: 0,
    crew: -40,
    max_health: 400.0,
    reinforced: true,
    buffs: &[]
  };

  pub const CITADEL_MAGAZINE: Component = Component {
    name: "Citadel Magazine",
    save_key: "Stock/Citadel Magazine",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Magazine {
      available_volume: 10
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: Some(0.0),
    first_instance_free: true,
    point_cost: 8,
    mass: 6.0,
    size: Size::new(1, 3, 1),
    power: -10,
    crew: 0,
    max_health: 300.0,
    reinforced: true,
    buffs: &[]
  };

  pub const CIVILIAN_REACTOR: Component = Component {
    name: "Civilian Reactor",
    save_key: "Stock/Civilian Reactor",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 40.0,
    size: Size::new(5, 5, 5),
    power: 3650,
    crew: -10,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CONTAINER_BANK_LAUNCHER: Component = Component {
    name: "Container Bank Launcher",
    save_key: "Stock/Container Bank Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::ContainerMissile,
      missile_size: MissileSize::Size3,
      cells: MissileLauncherCells::Tiling {
        count_per_group: 24,
        separated_groups: true
      }
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 5,
    mass: 50.0,
    size: Size::new(20, 5, 10),
    power: -200,
    crew: 0,
    max_health: 500.0,
    reinforced: false,
    buffs: &[]
  };

  pub const CONTAINER_STACK_LAUNCHER: Component = Component {
    name: "Container Stack Launcher",
    save_key: "Stock/Container Stack Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::ContainerMissile,
      missile_size: MissileSize::Size3,
      cells: MissileLauncherCells::Constant { count: 2 }
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 5,
    mass: 3.0,
    size: Size::new(6, 1, 6),
    power: -50,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const DAMAGE_CONTROL_CENTRAL: Component = Component {
    name: "Damage Control Central",
    save_key: "Stock/Damage Control Central",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: -10,
    max_health: 200.0,
    reinforced: false,
    buffs: &[
      (Buff::RepairSpeed, 0.1), (Buff::RepairTeamMoveSpeed, 0.5)
    ]
  };

  pub const DAMAGE_CONTROL_COMPLEX: Component = Component {
    name: "Damage Control Complex",
    save_key: "Stock/Damage Control Complex",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 3,
      repair_speed: 0.2,
      movement_speed: 0.5,
      restores: 2
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: Some(3.0),
    first_instance_free: false,
    point_cost: 80,
    mass: 20.0,
    size: Size::new(6, 3, 6),
    power: 0,
    crew: -50,
    max_health: 600.0,
    reinforced: true,
    buffs: &[
      (Buff::MaxRepair, 0.05),
      (Buff::RepairSpeed, 0.1),
      (Buff::RepairTeamMoveSpeed, 0.3)
    ]
  };

  pub const E15_MASQUERADE_DECEPTION_MODULE: Component = Component {
    name: "E15 'Masquerade' Deception Module",
    save_key: "Stock/E15 'Masquerade' Signature Booster",
    kind: ComponentKind::Mount,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 5.0,
    size: Size::new(3, 1, 3),
    power: -600,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const E20_LIGHTHOUSE_ILLUMINATOR: Component = Component {
    name: "E20 'Lighthouse' Illuminator",
    save_key: "Stock/E20 'Lighthouse' Illuminator",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Illuminator {
      max_range: 10000.0,
      battleshort_available: true,
      burst_duration: 60.0,
      cooldown_time: 45.0,
      cone_fov: 2.5
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -600,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::RadarSignature, 0.1)
    ]
  };

  pub const E55_SPOTLIGHT_ILLUMINATOR: Component = Component {
    name: "E55 'Spotlight' Illuminator",
    save_key: "Stock/E55 'Spotlight' Illuminator",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Illuminator {
      max_range: 12000.0,
      battleshort_available: true,
      burst_duration: 90.0,
      cooldown_time: 60.0,
      cone_fov: 2.5
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -750,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::RadarSignature, 0.1)
    ]
  };

  pub const E57_FLOODLIGHT_ILLUMINATOR: Component = Component {
    name: "E57 'Floodlight' Illuminator",
    save_key: "Stock/E57 'Floodlight' Illuminator",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Illuminator {
      max_range: 10000.0,
      battleshort_available: true,
      burst_duration: 90.0,
      cooldown_time: 60.0,
      cone_fov: 20.0
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -1500,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::RadarSignature, 0.1)
    ]
  };

  pub const E70_INTERRUPTION_JAMMER: Component = Component {
    name: "E70 'Interruption' Jammer",
    save_key: "Stock/E70 'Interruption' Jammer",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Comms,
      max_range: 4000.0,
      battleshort_available: true,
      burst_duration: 180.0,
      cooldown_time: 60.0,
      cone_fov: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 55,
    mass: 5.0,
    size: Size::new(3, 1, 3),
    power: -600,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const E71_HANGUP_JAMMER: Component = Component {
    name: "E71 'Hangup' Jammer",
    save_key: "Stock/E71 'Hangup' Jammer",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Comms,
      max_range: 20000.0,
      battleshort_available: true,
      burst_duration: 120.0,
      cooldown_time: 60.0,
      cone_fov: Some(10.0)
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -500,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[]
  };

  pub const E90_BLANKET_JAMMER: Component = Component {
    name: "E90 'Blanket' Jammer",
    save_key: "Stock/E90 'Blanket' Jammer",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Radar,
      max_range: 10000.0,
      battleshort_available: true,
      burst_duration: 90.0,
      cooldown_time: 60.0,
      cone_fov: Some(20.0)
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 50,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -1000,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::RadarSignature, 0.1)
    ]
  };

  pub const ES22_PINARD_ELECTRONIC_SUPPORT_MODULE: Component = Component {
    name: "ES22 'Pinard' Electronic Support Module",
    save_key: "Stock/ES22 'Pinard' Electronic Support Module",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::SensorPassive),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 3.0,
    size: Size::new(3, 1, 3),
    power: -200,
    crew: 0,
    max_health: 50.0,
    reinforced: false,
    buffs: &[]
  };

  pub const ES32_SCRYER_MISSILE_ID_SYSTEM: Component = Component {
    name: "ES32 'Scryer' Missile ID System",
    save_key: "Stock/ES32 'Scryer' Missile ID System",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Intelligence {
      work_on_remote_tracks: false,
      intel_effort: 10.0,
      intel_accuracy: 0.0
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -300,
    crew: 0,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const ENERGY_REGULATOR: Component = Component {
    name: "Energy Regulator",
    save_key: "Stock/Energy Regulator",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 30.0,
    size: Size::new(3, 3, 3),
    power: -650,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::CooldownTimeBeam, -0.2),
      (Buff::CooldownTimeEnergy, -0.2),
      (Buff::ReloadTimeEnergy, -0.2)
    ]
  };

  pub const FM200_DRIVE: Component = Component {
    name: "FM200 Drive",
    save_key: "Stock/FM200 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 500,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[]
  };

  pub const FM200R_DRIVE: Component = Component {
    name: "FM200R Drive",
    save_key: "Stock/FM200R Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 500,
    crew: -10,
    max_health: 500.0,
    reinforced: true,
    buffs: &[]
  };

  pub const FM230_WHIPLASH_DRIVE: Component = Component {
    name: "FM230 'Whiplash' Drive",
    save_key: "Stock/FM230 'Whiplash' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 650,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, -0.2),
      (Buff::FlankDamageProbability, -0.15),
      (Buff::TopSpeed, 0.2),
      (Buff::TurnRate, -0.15)
    ]
  };

  pub const FM240_DRAGONFLY_DRIVE: Component = Component {
    name: "FM240 'Dragonfly' Drive",
    save_key: "Stock/FM240 'Dragonfly' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 500,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, 0.3),
      (Buff::TopSpeed, -0.075),
      (Buff::TurnRate, 0.4)
    ]
  };

  pub const FM280_RAIDER_DRIVE: Component = Component {
    name: "FM280 'Raider' Drive",
    save_key: "Stock/FM280 'Raider' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 0,
    crew: -10,
    max_health: 250.0,
    reinforced: false,
    buffs: &[
      (Buff::FlankDamageProbability, 0.5),
      (Buff::LinearThrust, 0.3),
      (Buff::RadarSignature, 0.2)
    ]
  };

  pub const FM30X_PROWLER_DRIVE: Component = Component {
    name: "FM30X 'Prowler' Drive",
    save_key: "Stock/FM30X 'Prowler' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 350,
    crew: -10,
    max_health: 200.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, -0.1),
      (Buff::LinearThrust, -0.25),
      (Buff::RadarSignature, -0.25),
      (Buff::TopSpeed, -0.15)
    ]
  };

  pub const FM500_DRIVE: Component = Component {
    name: "FM500 Drive",
    save_key: "Stock/FM500 Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 80.0,
    size: Size::new(10, 8, 10),
    power: 1000,
    crew: -10,
    max_health: 1000.0,
    reinforced: false,
    buffs: &[]
  };

  pub const FM500R_DRIVE: Component = Component {
    name: "FM500R Drive",
    save_key: "Stock/FM500R Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 80.0,
    size: Size::new(10, 8, 10),
    power: 1000,
    crew: -10,
    max_health: 1500.0,
    reinforced: true,
    buffs: &[]
  };

  pub const FM530_WHIPLASH_DRIVE: Component = Component {
    name: "FM530 'Whiplash' Drive",
    save_key: "Stock/FM530 'Whiplash' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 80.0,
    size: Size::new(10, 8, 10),
    power: 1500,
    crew: -10,
    max_health: 1000.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, -0.2),
      (Buff::FlankDamageProbability, -0.15),
      (Buff::TopSpeed, 0.15),
      (Buff::TurnRate, -0.2)
    ]
  };

  pub const FM540_DRAGONFLY_DRIVE: Component = Component {
    name: "FM540 'Dragonfly' Drive",
    save_key: "Stock/FM540 'Dragonfly' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 80.0,
    size: Size::new(10, 8, 10),
    power: 1000,
    crew: -10,
    max_health: 1000.0,
    reinforced: false,
    buffs: &[
      (Buff::AngularThrust, 0.4),
      (Buff::TopSpeed, -0.1),
      (Buff::TurnRate, 0.45)
    ]
  };

  pub const FM580_RAIDER_DRIVE: Component = Component {
    name: "FM580 'Raider' Drive",
    save_key: "Stock/FM580 'Raider' Drive",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 80.0,
    size: Size::new(10, 8, 10),
    power: 500,
    crew: -10,
    max_health: 1000.0,
    reinforced: false,
    buffs: &[
      (Buff::FlankDamageProbability, 0.5),
      (Buff::LinearThrust, 0.3),
      (Buff::RadarSignature, 0.25)
    ]
  };

  pub const FR3300_MICRO_REACTOR: Component = Component {
    name: "FR3300 Micro Reactor",
    save_key: "Stock/FR3300 Micro Reactor",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 25.0,
    size: Size::new(3, 3, 3),
    power: 2250,
    crew: -5,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const FR4800_REACTOR: Component = Component {
    name: "FR4800 Reactor",
    save_key: "Stock/FR4800 Reactor",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 40.0,
    size: Size::new(5, 5, 5),
    power: 4200,
    crew: -10,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const FOCUSED_PARTICLE_ACCELERATOR: Component = Component {
    name: "Focused Particle Accelerator",
    save_key: "Stock/Focused Particle Accelerator",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 45.0,
    size: Size::new(3, 3, 3),
    power: -800,
    crew: 0,
    max_health: 150.0,
    reinforced: false,
    buffs: &[
      (Buff::BurstDurationBeam, -0.05),
      (Buff::DamageMultiplierBeam, 0.25)
    ]
  };

  pub const GUN_PLOTTING_CENTER: Component = Component {
    name: "Gun Plotting Center",
    save_key: "Stock/Gun Plotting Center",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: -30,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::Spread, -0.3)
    ]
  };

  pub const INTELLIGENCE_CENTER: Component = Component {
    name: "Intelligence Center",
    save_key: "Stock/Intelligence Center",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Intelligence {
      work_on_remote_tracks: true,
      intel_effort: 15.0,
      intel_accuracy: 0.15
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 90,
    mass: 25.0,
    size: Size::new(6, 1, 8),
    power: 0,
    crew: -45,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const ITHACA_BRIDGEMASTER: Component = Component {
    name: "Ithaca Bridgemaster",
    save_key: "Stock/Ithaca Bridgemaster",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 8500.0,
      can_lock: false,
      can_burnthrough: false,
      cone_fov: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -1200,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const J15_BELLBIRD_JAMMER: Component = Component {
    name: "J15 'Bellbird' Jammer",
    save_key: "Stock/J15 Jammer",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Radar,
      max_range: 10000.0,
      battleshort_available: true,
      burst_duration: 45.0,
      cooldown_time: 60.0,
      cone_fov: Some(20.0)
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 50,
    mass: 2.0,
    size: Size::new(3, 1, 3),
    power: -850,
    crew: 0,
    max_health: 150.0,
    reinforced: false,
    buffs: &[
      (Buff::RadarSignature, 0.1)
    ]
  };

  pub const J360_LYREBIRD_JAMMER: Component = Component {
    name: "J360 'Lyrebird' Jammer",
    save_key: "Stock/J360 Jammer",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Radar,
      max_range: 4000.0,
      battleshort_available: true,
      burst_duration: 180.0,
      cooldown_time: 60.0,
      cone_fov: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 75,
    mass: 10.0,
    size: Size::new(3, 1, 3),
    power: -500,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const JURYRIGGED_REACTOR: Component = Component {
    name: "Jury-Rigged Reactor",
    save_key: "Stock/Jury-Rigged Reactor",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 40.0,
    size: Size::new(6, 3, 6),
    power: 500,
    crew: -30,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const L50_BLACKJACK_LASER_DAZZLER: Component = Component {
    name: "L50 'Blackjack' Laser Dazzler",
    save_key: "Stock/L50 Laser Dazzler",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Jammer {
      sig_type: SigType::Radar,
      max_range: 8000.0,
      battleshort_available: true,
      burst_duration: 90.0,
      cooldown_time: 60.0,
      cone_fov: Some(30.0)
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -750,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[]
  };

  pub const LARGE_DC_LOCKER: Component = Component {
    name: "Large DC Locker",
    save_key: "Stock/Large DC Locker",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 3,
      repair_speed: 0.2,
      movement_speed: 0.5,
      restores: 2
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 3.0,
    size: Size::new(3, 1, 6),
    power: 0,
    crew: -15,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const LARGE_DC_STORAGE: Component = Component {
    name: "Large DC Storage",
    save_key: "Stock/Large DC Storage",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 1,
      repair_speed: 0.2,
      movement_speed: 0.5,
      restores: 4
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 7.0,
    size: Size::new(6, 3, 6),
    power: 0,
    crew: -5,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const LAUNCHER_DELUGE_SYSTEM: Component = Component {
    name: "Launcher Deluge System",
    save_key: "Stock/Launcher Deluge System",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 30.0,
    size: Size::new(2, 2, 2),
    power: -100,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::CatastrophicEventProbCellLauncher, -0.5)
    ]
  };

  pub const LIGHT_CIVILIAN_REACTOR: Component = Component {
    name: "Light Civilian Reactor",
    save_key: "Stock/Light Civilian Reactor",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 25.0,
    size: Size::new(3, 3, 3),
    power: 1850,
    crew: -5,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const ML9_MINE_LAUNCHER: Component = Component {
    name: "ML-9 Mine Launcher",
    save_key: "Stock/ML-9 Mine Launcher",
    kind: ComponentKind::Mount,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: Some(2.0),
    first_instance_free: false,
    point_cost: 10,
    mass: 7.5,
    size: Size::new(3, 2, 3),
    power: -100,
    crew: -10,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MLS2_LAUNCHER: Component = Component {
    name: "MLS-2 Launcher",
    save_key: "Stock/MLS-2 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileLauncher {
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size2,
      load_time: 45.0
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 7.5,
    size: Size::new(2, 2, 5),
    power: -50,
    crew: -10,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MLS3_LAUNCHER: Component = Component {
    name: "MLS-3 Launcher",
    save_key: "Stock/MLS-3 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileLauncher {
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size3,
      load_time: 60.0
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 15.0,
    size: Size::new(2, 2, 5),
    power: -50,
    crew: -12,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MAGAZINE_SPRINKLERS: Component = Component {
    name: "Magazine Sprinklers",
    save_key: "Stock/Magazine Sprinklers",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 30.0,
    size: Size::new(2, 2, 2),
    power: -100,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::CatastrophicEventProbMagazine, -0.5)
    ]
  };

  pub const MISSILE_PARALLEL_INTERFACE: Component = Component {
    name: "Missile Parallel Interface",
    save_key: "Stock/Missile Parallel Interface",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -150,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::MissileProgrammingSpeed, 0.35)
    ]
  };

  pub const MISSILE_PROGRAMMING_BUS: Component = Component {
    name: "Missile Programming Bus",
    save_key: "Stock/Missile Programming Bus",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -200,
    crew: 0,
    max_health: 80.0,
    reinforced: false,
    buffs: &[
      (Buff::MissileProgrammingChannels, 1.0)
    ]
  };

  pub const MISSILE_PROGRAMMING_BUS_ARRAY: Component = Component {
    name: "Missile Programming Bus Array",
    save_key: "Stock/Missile Programming Bus Array",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 50,
    mass: 5.0,
    size: Size::new(3, 3, 3),
    power: -350,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::MissileProgrammingChannels, 2.0)
    ]
  };

  pub const MK20_DEFENDER_PDT: Component = Component {
    name: "Mk20 'Defender' PDT",
    save_key: "Stock/Mk20 'Defender' PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: Some(FireControl {
        sig_type: SigType::Radar,
        max_range: 2500.0
      }),
      optical_backup: false,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticChemical20mm,
      reload_time: 60.0 / 2400.0,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 3.0,
    size: Size::new(2, 1, 2),
    power: -175,
    crew: -3,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK25_REBOUND_PDT: Component = Component {
    name: "Mk25 'Rebound' PDT",
    save_key: "Stock/Mk25 'Rebound' PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticChemical50mmFlak,
      reload_time: 3.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(15),
        recycle_time: 0.3
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 3.0,
    size: Size::new(2, 1, 2),
    power: -20,
    crew: -3,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK29_STONEWALL_PDT: Component = Component {
    name: "Mk29 'Stonewall' PDT",
    save_key: "Stock/Mk29 'Stonewall' PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticChemical50mmFlak,
      reload_time: 1.5,
      autoloader: Some(Autoloader {
        capacity: zsize!(16),
        recycle_time: 0.15
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: -40,
    crew: -5,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK550_RAILGUN: Component = Component {
    name: "Mk550 Railgun",
    save_key: "Stock/Mk550 Mass Driver",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticMagnetic300mmRailgun,
      reload_time: 15.0,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 75,
    mass: 100.0,
    size: Size::new(4, 12, 4),
    power: -2000,
    crew: -15,
    max_health: 450.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK600_BEAM_CANNON: Component = Component {
    name: "Mk600 Beam Cannon",
    save_key: "Stock/Mk600 Beam Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponBeam {
      is_fixed: true,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      battleshort_available: true,
      burst_duration: 7.5,
      cooldown_time: 30.0
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 75,
    mass: 100.0,
    size: Size::new(4, 12, 4),
    power: -3000,
    crew: -15,
    max_health: 450.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK610_BEAM_TURRET: Component = Component {
    name: "Mk610 Beam Turret",
    save_key: "Stock/Mk610 Beam Turret",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponBeam {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      battleshort_available: true,
      burst_duration: 7.5,
      cooldown_time: 45.0
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 80,
    mass: 50.0,
    size: Size::new(8, 5, 8),
    power: -3000,
    crew: -20,
    max_health: 400.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK61_CANNON: Component = Component {
    name: "Mk61 Cannon",
    save_key: "Stock/Mk61 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::BallisticChemical120mm,
      reload_time: 5.5,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 3.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: -4,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK62_CANNON: Component = Component {
    name: "Mk62 Cannon",
    save_key: "Stock/Mk62 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::BallisticChemical120mm,
      reload_time: 4.0,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 6.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: -8,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK64_CANNON: Component = Component {
    name: "Mk64 Cannon",
    save_key: "Stock/Mk64 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::BallisticChemical250mm,
      reload_time: 10.0,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 3.0,
    size: Size::new(3, 1, 5),
    power: -125,
    crew: -15,
    max_health: 225.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK65_CANNON: Component = Component {
    name: "Mk65 Cannon",
    save_key: "Stock/Mk65 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical250mm,
      reload_time: 13.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(3),
        recycle_time: 0.5
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 12.0,
    size: Size::new(6, 3, 6),
    power: -180,
    crew: -25,
    max_health: 450.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK66_CANNON: Component = Component {
    name: "Mk66 Cannon",
    save_key: "Stock/Mk66 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical450mm,
      reload_time: 20.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(2),
        recycle_time: 0.5
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 25.0,
    size: Size::new(6, 3, 6),
    power: -200,
    crew: -25,
    max_health: 550.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK68_CANNON: Component = Component {
    name: "Mk68 Cannon",
    save_key: "Stock/Mk68 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticChemical450mm,
      reload_time: 20.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(3),
        recycle_time: 0.5
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 75,
    mass: 40.0,
    size: Size::new(8, 5, 8),
    power: -225,
    crew: -40,
    max_health: 650.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK81_RAILGUN: Component = Component {
    name: "Mk81 Railgun",
    save_key: "Stock/Mk81 Railgun",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticMagnetic300mmRailgun,
      reload_time: 30.0,
      autoloader: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 50,
    mass: 20.0,
    size: Size::new(6, 3, 6),
    power: -1500,
    crew: -15,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK90_AURORA_PDT: Component = Component {
    name: "Mk90 'Aurora' PDT",
    save_key: "Stock/Mk90 'Aurora' PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponBeam {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: Some(FireControl {
        sig_type: SigType::Radar,
        max_range: 3250.0
      }),
      optical_backup: false,
      role: WeaponRole::Defensive,
      battleshort_available: true,
      burst_duration: 1.8,
      cooldown_time: 3.4
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 55,
    mass: 3.0,
    size: Size::new(2, 1, 2),
    power: -1250,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MK95_SARISSA_PDT: Component = Component {
    name: "Mk95 'Sarissa' PDT",
    save_key: "Stock/Mk95 'Sarissa' PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: Some(FireControl {
        sig_type: SigType::Radar,
        max_range: 8000.0
      }),
      optical_backup: true,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticMagnetic15mm,
      reload_time: 6.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(3),
        recycle_time: 0.33
      })
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 65,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: -1250,
    crew: -4,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const MOUNT_GYROS: Component = Component {
    name: "Mount Gyros",
    save_key: "Stock/Mount Gyros",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -150,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::CasemateElevationRate, 0.15),
      (Buff::CasemateTraverseRate, 0.15),
      (Buff::ElevationRate, 0.2),
      (Buff::Spread, -0.075),
      (Buff::TraverseRate, 0.25)
    ]
  };

  pub const P11_PAVISE_PDT: Component = Component {
    name: "P11 'Pavise' PDT",
    save_key: "Stock/P11 PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: Some(FireControl {
        sig_type: SigType::Radar,
        max_range: 2500.0
      }),
      optical_backup: false,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticChemical20mm,
      reload_time: 60.0 / 4800.0,
      autoloader: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 4.0,
    size: Size::new(2, 1, 2),
    power: -150,
    crew: -3,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const P20_BASTION_PDT: Component = Component {
    name: "P20 'Bastion' PDT",
    save_key: "Stock/P20 Flak PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::BallisticChemical50mmFlak,
      reload_time: 0.33,
      autoloader: Some(Autoloader {
        capacity: zsize!(100),
        recycle_time: 0.33
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 7.0,
    size: Size::new(2, 1, 2),
    power: -40,
    crew: -5,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const P60_GRAZER_PDT: Component = Component {
    name: "P60 'Grazer' PDT",
    save_key: "Stock/P60 Laser PDT",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: Some(FireControl {
        sig_type: SigType::ElectroOptical,
        max_range: 3000.0
      }),
      optical_backup: false,
      role: WeaponRole::Defensive,
      munition_family: MunitionFamily::Infinite,
      reload_time: 7.0,
      autoloader: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 35,
    mass: 3.0,
    size: Size::new(2, 1, 2),
    power: -1500,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const PLANT_CONTROL_CENTER: Component = Component {
    name: "Plant Control Center",
    save_key: "Stock/Plant Control Center",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -15,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::PowerplantEfficiency, 0.2)
    ]
  };

  pub const R400_BLOODHOUND_LRT_RADAR: Component = Component {
    name: "R400 'Bloodhound' LRT Radar",
    save_key: "Stock/R400 Radar",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 14000.0,
      can_lock: false,
      can_burnthrough: false,
      cone_fov: Some(-1.0)
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 0.0, // lol
    size: Size::new(2, 5, 2),
    power: -3000,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const R550_EARLY_WARNING_RADAR: Component = Component {
    name: "R550 Early Warning Radar",
    save_key: "Stock/R550 Early Warning Radar",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 18000.0,
      can_lock: false,
      can_burnthrough: false,
      cone_fov: Some(-1.0)
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 65,
    mass: 10.0,
    size: Size::new(3, 1, 3),
    power: -4500,
    crew: 0,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RF101_BULLSEYE_RADAR: Component = Component {
    name: "RF101 'Bullseye' Radar",
    save_key: "Stock/RF101 'Bullseye' Radar",
    kind: ComponentKind::Mount,
    variant: None,
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 40,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -300,
    crew: 0,
    max_health: 70.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RF44_PINPOINT_RADAR: Component = Component {
    name: "RF44 'Pinpoint' Radar",
    save_key: "Stock/RF44 'Pinpoint' Radar",
    kind: ComponentKind::Mount,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 2.0,
    size: Size::new(2, 1, 2),
    power: -250,
    crew: 0,
    max_health: 70.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RL18_LAUNCHER: Component = Component {
    name: "RL18 Launcher",
    save_key: "Stock/RL18 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: false,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::UnguidedRocket,
      missile_size: MissileSize::Size1,
      cells: MissileLauncherCells::Constant { count: 18 }
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 8.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RL36_LAUNCHER: Component = Component {
    name: "RL36 Launcher",
    save_key: "Stock/RL36 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: false,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::UnguidedRocket,
      missile_size: MissileSize::Size1,
      cells: MissileLauncherCells::Constant { count: 36 }
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 13.0,
    size: Size::new(6, 3, 6),
    power: -75,
    crew: 0,
    max_health: 175.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RM50_PARALLAX_RADAR: Component = Component {
    name: "RM50 'Parallax' Radar",
    save_key: "Stock/RM50 'Parallax' Radar",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 9500.0,
      can_lock: true,
      can_burnthrough: true,
      cone_fov: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 7.5,
    size: Size::new(2, 2, 2),
    power: -3600,
    crew: 0,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RS35_FRONTLINE_RADAR: Component = Component {
    name: "RS35 'Frontline' Radar",
    save_key: "Stock/RS35 'Frontline' Radar",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 8000.0,
      can_lock: false,
      can_burnthrough: true,
      cone_fov: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -2000,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RS41_SPYGLASS_RADAR: Component = Component {
    name: "RS41 'Spyglass' Radar",
    save_key: "Stock/RS41 'Spyglass' Radar",
    kind: ComponentKind::Module,
    variant: Some(ComponentVariant::Sensor {
      sig_type: SigType::Radar,
      max_range: 11500.0,
      can_lock: false,
      can_burnthrough: false,
      cone_fov: None
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -4000,
    crew: 0,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const RAPIDCYCLE_CRADLE: Component = Component {
    name: "Rapid-Cycle Cradle",
    save_key: "Stock/Rapid-Cycle Cradle",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 30.0,
    size: Size::new(3, 3, 3),
    power: -150,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::RecycleTime, -0.25), (Buff::RecycleTimeEnergy, -0.25)
    ]
  };

  pub const RAPID_DC_LOCKER: Component = Component {
    name: "Rapid DC Locker",
    save_key: "Stock/Rapid DC Locker",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 1,
      repair_speed: 0.3,
      movement_speed: 1.0,
      restores: 0
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 1.5,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -5,
    max_health: 70.0,
    reinforced: false,
    buffs: &[]
  };

  pub const REDUNDANT_REACTOR_FAILSAFES: Component = Component {
    name: "Redundant Reactor Failsafes",
    save_key: "Stock/Redundant Reactor Failsafes",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 5.0,
    size: Size::new(2, 2, 2),
    power: -100,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::CatastrophicEventProbReactor, -0.33)
    ]
  };

  pub const REINFORCED_CIC: Component = Component {
    name: "Reinforced CIC",
    save_key: "Stock/Reinforced CIC",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Command {
      work_on_remote_tracks: false,
      intel_effort: 1.0,
      intel_accuracy: 0.3
    }),
    faction: None,
    compounding_multiplier: Some(1.0),
    first_instance_free: false,
    point_cost: 25,
    mass: 35.0,
    size: Size::new(4, 1, 6),
    power: 0,
    crew: -20,
    max_health: 200.0,
    reinforced: true,
    buffs: &[]
  };

  pub const REINFORCED_DC_LOCKER: Component = Component {
    name: "Reinforced DC Locker",
    save_key: "Stock/Reinforced DC Locker",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 2,
      repair_speed: 0.2,
      movement_speed: 0.5,
      restores: 1
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 1.5,
    size: Size::new(3, 1, 6),
    power: 0,
    crew: -10,
    max_health: 200.0,
    reinforced: true,
    buffs: &[]
  };

  pub const REINFORCED_MAGAZINE: Component = Component {
    name: "Reinforced Magazine",
    save_key: "Stock/Reinforced Magazine",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::Magazine {
      available_volume: 10
    }),
    faction: None,
    compounding_multiplier: Some(0.0),
    first_instance_free: true,
    point_cost: 2,
    mass: 2.0,
    size: Size::new(1, 1, 1),
    power: -10,
    crew: 0,
    max_health: 150.0,
    reinforced: true,
    buffs: &[]
  };

  pub const REINFORCED_THRUSTER_NOZZLES: Component = Component {
    name: "Reinforced Thruster Nozzles",
    save_key: "Stock/Reinforced Thruster Nozzles",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: 0,
    crew: 0,
    max_health: 50.0,
    reinforced: false,
    buffs: &[
      (Buff::FlankDamageProbability, -0.2)
    ]
  };

  pub const SIGNATURE_SCRAMBLER: Component = Component {
    name: "Signature Scrambler",
    save_key: "Stock/Signature Scrambler",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 35,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -450,
    crew: 0,
    max_health: 50.0,
    reinforced: false,
    buffs: &[
      (Buff::IdentificationDifficulty, 1.0),
      (Buff::Sensitivity, -0.1)
    ]
  };

  pub const SMALL_DC_LOCKER: Component = Component {
    name: "Small DC Locker",
    save_key: "Stock/Small DC Locker",
    kind: ComponentKind::Compartment,
    variant: Some(ComponentVariant::DamageControl {
      teams: 2,
      repair_speed: 0.2,
      movement_speed: 0.5,
      restores: 1
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 1.5,
    size: Size::new(3, 1, 3),
    power: 0,
    crew: -10,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const SMALL_ENERGY_REGULATOR: Component = Component {
    name: "Small Energy Regulator",
    save_key: "Stock/Small Energy Regulator",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 20.0,
    size: Size::new(2, 2, 2),
    power: -400,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[
      (Buff::CooldownTimeBeam, -0.12),
      (Buff::CooldownTimeEnergy, -0.12),
      (Buff::ReloadTimeEnergy, -0.12)
    ]
  };

  pub const SMALL_REACTOR_BOOSTER: Component = Component {
    name: "Small Reactor Booster",
    save_key: "Stock/Small Reactor Booster",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: 0,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::PowerplantEfficiency, 0.1),
      (Buff::RadarSignature, 0.05)
    ]
  };

  pub const SMALL_WORKSHOP: Component = Component {
    name: "Small Workshop",
    save_key: "Stock/Small Workshop",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 7.0,
    size: Size::new(3, 1, 3),
    power: -50,
    crew: -10,
    max_health: 150.0,
    reinforced: false,
    buffs: &[
      (Buff::RepairSpeed, 0.3)
    ]
  };

  pub const STRIKE_PLANNING_CENTER: Component = Component {
    name: "Strike Planning Center",
    save_key: "Stock/Strike Planning Center",
    kind: ComponentKind::Compartment,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 70,
    mass: 10.0,
    size: Size::new(6, 1, 8),
    power: 0,
    crew: -20,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::MissileProgrammingSpeed, 0.6)
    ]
  };

  pub const STROBE_CORRELATOR: Component = Component {
    name: "Strobe Correlator",
    save_key: "Stock/Strobe Correlator",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -150,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::JammingLobAccuracy, -0.3)
    ]
  };

  pub const SUNDRIVE_RACING_PRO: Component = Component {
    name: "Sundrive Racing Pro",
    save_key: "Stock/Sundrive Racing Pro",
    kind: ComponentKind::Module,
    variant: None,
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 30,
    mass: 35.0,
    size: Size::new(5, 3, 5),
    power: 200,
    crew: -10,
    max_health: 100.0,
    reinforced: false,
    buffs: &[
      (Buff::FlankDamageProbability, -0.2), (Buff::TopSpeed, 0.25)
    ]
  };

  pub const SUPPLEMENTARY_RADIO_AMPLIFIERS: Component = Component {
    name: "Supplementary Radio Amplifiers",
    save_key: "Stock/Supplementary Radio Amplifiers",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -100,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::TransmitPower, 0.25)
    ]
  };

  pub const T20_CANNON: Component = Component {
    name: "T20 Cannon",
    save_key: "Stock/T20 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::BallisticChemical100mm,
      reload_time: 15.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(4),
        recycle_time: 1.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 15,
    mass: 3.0,
    size: Size::new(3, 1, 3),
    power: -30,
    crew: -4,
    max_health: 150.0,
    reinforced: false,
    buffs: &[]
  };

  pub const T30_CANNON: Component = Component {
    name: "T30 Cannon",
    save_key: "Stock/T30 Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: false,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::BallisticChemical100mm,
      reload_time: 30.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(16),
        recycle_time: 1.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 50.0,
    size: Size::new(6, 3, 6),
    power: -120,
    crew: -10,
    max_health: 200.0,
    reinforced: false,
    buffs: &[]
  };

  pub const T81_PLASMA_CANNON: Component = Component {
    name: "T81 Plasma Cannon",
    save_key: "Stock/T81 Plasma Cannon",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: false,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticMagnetic400mmPlasma,
      reload_time: 40.0,
      autoloader: Some(Autoloader {
        capacity: zsize!(8),
        recycle_time: 5.0
      })
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 45,
    mass: 80.0,
    size: Size::new(6, 3, 6),
    power: -1500,
    crew: -15,
    max_health: 300.0,
    reinforced: false,
    buffs: &[]
  };

  pub const TE45_MASS_DRIVER: Component = Component {
    name: "TE45 Mass Driver",
    save_key: "Stock/TE45 Mass Driver",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponProjectile {
      is_fixed: true,
      is_energy: true,
      integrated_fire_control: None,
      optical_backup: true,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::BallisticMagnetic500mmMassDriver,
      reload_time: 25.0,
      autoloader: None
    }),
    faction: Some(Faction::Protectorate),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 100,
    mass: 100.0,
    size: Size::new(4, 10, 4),
    power: -4250,
    crew: -15,
    max_health: 450.0,
    reinforced: false,
    buffs: &[]
  };

  pub const TLS3_LAUNCHER: Component = Component {
    name: "TLS-3 Launcher",
    save_key: "Stock/Torpedo Turret",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: false,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size3,
      cells: MissileLauncherCells::Constant { count: 6 }
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 25.0,
    size: Size::new(3, 1, 5),
    power: -250,
    crew: 0,
    max_health: 350.0,
    reinforced: false,
    buffs: &[]
  };

  pub const TRACK_CORRELATOR: Component = Component {
    name: "Track Correlator",
    save_key: "Stock/Track Correlator",
    kind: ComponentKind::Module,
    variant: None,
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 25,
    mass: 4.0,
    size: Size::new(2, 2, 2),
    power: -150,
    crew: 0,
    max_health: 40.0,
    reinforced: false,
    buffs: &[
      (Buff::PositionalError, -0.2), (Buff::VelocityError, -0.2)
    ]
  };

  pub const VLS1_23_LAUNCHER: Component = Component {
    name: "VLS-1-23 Launcher",
    save_key: "Stock/VLS-1-23 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size1,
      cells: MissileLauncherCells::Constant { count: 23 }
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 5,
    mass: 3.0,
    size: Size::new(2, 2, 2),
    power: -50,
    crew: 0,
    max_health: 75.0,
    reinforced: false,
    buffs: &[]
  };

  pub const VLS1_46_LAUNCHER: Component = Component {
    name: "VLS-1-46 Launcher",
    save_key: "Stock/VLS-1-46 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::DualPurpose,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size1,
      cells: MissileLauncherCells::Constant { count: 46 }
    }),
    faction: None,
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 5,
    mass: 10.0,
    size: Size::new(3, 2, 3),
    power: -50,
    crew: 0,
    max_health: 125.0,
    reinforced: false,
    buffs: &[]
  };

  pub const VLS2_LAUNCHER: Component = Component {
    name: "VLS-2 Launcher",
    save_key: "Stock/VLS-2 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size2,
      cells: MissileLauncherCells::Function {
        count_per_group: 8,
        groups: |size| match size {
          Size { x: 3, y, z: 3 } if y >= 4 => Some([1, 2]), // 16
          Size { x: 3, y, z: 5 } if y >= 4 => Some([1, 3]), // 24
          Size { x: 6, y, z: 6 } if y >= 4 => Some([2, 4]), // 64
          Size { x: 8, y, z: 8 } if y >= 4 => Some([2, 5]), // 80
          _ => None
        }
      }
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 10,
    mass: 5.0,
    size: Size::new(3, 4, 3),
    power: -200,
    crew: 0,
    max_health: 100.0,
    reinforced: false,
    buffs: &[]
  };

  pub const VLS3_LAUNCHER: Component = Component {
    name: "VLS-3 Launcher",
    save_key: "Stock/VLS-3 Launcher",
    kind: ComponentKind::Mount,
    variant: Some(ComponentVariant::WeaponMissileBank {
      is_fixed: true,
      integrated_fire_control: None,
      optical_backup: false,
      role: WeaponRole::Offensive,
      munition_family: MunitionFamily::StandardMissile,
      missile_size: MissileSize::Size3,
      cells: MissileLauncherCells::Function {
        count_per_group: 2,
        groups: |size| match size {
          Size { x: 3, y, z: 3 } if y >= 4 => Some([1, 2]), // 4
          Size { x: 3, y, z: 5 } if y >= 4 => Some([1, 3]), // 6
          Size { x: 6, y, z: 6 } if y >= 4 => Some([2, 4]), // 16
          Size { x: 8, y, z: 8 } if y >= 4 => Some([2, 5]), // 20
          _ => None
        }
      }
    }),
    faction: Some(Faction::Alliance),
    compounding_multiplier: None,
    first_instance_free: false,
    point_cost: 20,
    mass: 20.0,
    size: Size::new(3, 4, 3),
    power: -200,
    crew: 0,
    max_health: 175.0,
    reinforced: false,
    buffs: &[]
  };
}
