pub mod predicate;

use self::predicate::ShipPredicate;

use nebulous_data::data::components::{ComponentKey, ComponentVariant, SigType};
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::hulls::config::Variant;
use nebulous_data::data::missiles::{AuxiliaryKey, AvionicsKey, Maneuvers, WarheadKey};
use nebulous_data::data::missiles::seekers::{SeekerKind, SeekerStrategy};
use nebulous_data::data::missiles::bodies::MissileBodyKey;
use nebulous_data::data::munitions::{MunitionFamily, MunitionKey, WeaponRole};
use nebulous_data::data::MissileSize;
use nebulous_data::format::{ComponentData, Color, MunitionOrMissileKey, MissileTemplate, MissileSocket, Ship};
use nebulous_data::loadout::{
  AvionicsConfigured, MissileLoadout, MissileLoadoutError, MissileTemplateAdditional, MissileTemplateSummary,
  ShipAdditional, ShipLoadout, ShipLoadoutError
};
use nebulous_data::uuid::Builder as UuidBuilder;
use rand::Rng;
use rand::seq::SliceRandom;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::Extend;
use std::str::FromStr;



#[derive(Debug, Error)]
pub enum ModelError {
  #[error("ship loadout error: {0}")]
  ShipLoadout(#[from] ShipLoadoutError),
  #[error("missile_loadout error: {0}")]
  MissileLoadout(#[from] MissileLoadoutError),
  #[error("missile has no seekers")]
  MissileHasNoSeekers,
  #[error("missile has no avionics")]
  MissileHasNoAvionics
}

const fn default_one() -> usize { 1 }

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FleetStrategy {
  pub selections: Vec<FleetStrategySelection>
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FleetStrategySelection {
  #[serde(default = "default_one")]
  pub weight_initial: usize,
  #[serde(default = "default_one")]
  pub weight_additional: usize,
  #[serde(default)]
  pub predicates: FleetStrategyPredicates
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FleetStrategyPredicates {
  #[serde(skip_serializing, default)]
  pub reject: Option<ShipPredicate>,
  #[serde(skip_serializing, default)]
  pub require: Option<ShipPredicate>,
  #[serde(skip_serializing, default)]
  pub prioritize: Option<ShipPredicate>
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShipState {
  #[serde(with = "crate::utils::serde_one_or_many")]
  pub name: Vec<String>,
  pub author: Option<String>,
  pub tags: HashSet<String>,
  pub cost_budget_total: usize,
  pub cost_budget_spare: usize,
  pub equipment_summary: ShipEquipmentSummary,
  #[serde(rename = "socket_data")]
  #[serde(with = "crate::utils::serde_base64_cbor")]
  pub loadout: ShipLoadout
}

impl ShipState {
  pub fn from_ship(ship: &Ship, missile_templates: &[MissileTemplate]) -> Result<Self, ShipLoadoutError> {
    let mut equipment_summary = ShipEquipmentSummary::default();

    let hull = ship.hull_type.hull();
    for hull_socket in ship.socket_map.iter() {
      let Some(hull_socket_definition) = hull.get_socket(hull_socket.key) else { continue };
      let component = hull_socket.component_name.component();

      equipment_summary.add_component_key(hull_socket.component_name);

      if let Some(ComponentVariant::WeaponMissileBank { munition_family, cells, .. }) = component.variant {
        if let Some(missile_type) = MissileType::from_munition_family(munition_family) {
          if let Some(count) = cells.get_count(hull_socket_definition.size, component.size) {
            *equipment_summary.missile_cells.entry(missile_type).or_default() += count;
          };
        };
      };
    };

    let costs = ship.calculate_costs(missile_templates);
    let loadout = ShipLoadout::from_ship(ship)?;

    Ok(ShipState {
      name: vec![ship.name.clone()],
      author: None,
      tags: HashSet::new(),
      cost_budget_total: costs.total(),
      cost_budget_spare: costs.missiles,
      equipment_summary,
      loadout
    })
  }

  pub fn to_ship<R: Rng + ?Sized>(&self, rng: &mut R) -> Ship {
    self.loadout.to_ship(ShipAdditional {
      key: UuidBuilder::from_random_bytes(rng.gen()).into_uuid(),
      name: self.name.choose(rng).cloned()
        .unwrap_or_else(|| "Ship".to_owned()),
      // TODO: properly calculate cost
      cost: 0,
      callsign: None,
      number: rng.gen_range(0..10000),
      weapon_groups: Vec::new(),
      initial_formation: None,
      missile_types: Vec::new()
    }, rng)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SocketState {
  pub component_key: ComponentKey,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub identity_option: Option<usize>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub magazine_contents: Option<BTreeMap<MunitionKey, usize>>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MissileData {
  pub designation: String,
  pub nickname: String,
  pub author: Option<String>,
  pub tags: HashSet<String>,
  pub base_color: Color,
  pub stripe_color: Color,
  pub equipment_summary: MissileEquipmentSummary,
  #[serde(rename = "socket_data")]
  #[serde(with = "crate::utils::serde_base64_cbor")]
  pub loadout: MissileLoadout
}

impl MissileData {
  pub fn from_missile_template(missile_template: &MissileTemplate) -> Result<Self, ModelError> {
    Ok(MissileData {
      designation: missile_template.designation.clone(),
      nickname: missile_template.nickname.clone(),
      author: None,
      tags: HashSet::new(),
      base_color: missile_template.base_color,
      stripe_color: missile_template.stripe_color,
      equipment_summary: MissileEquipmentSummary::from_missile_template_summary(
        missile_template.body_key, missile_template.get_summary()
      )?,
      loadout: MissileLoadout::from_missile_template(missile_template)?
    })
  }

  pub fn to_missile_template<R: Rng + ?Sized>(&self, rng: &mut R) -> MissileTemplate {
    self.loadout.to_missile_template(MissileTemplateAdditional {
      designation: self.designation.clone(),
      nickname: self.nickname.clone(),
      description: String::new(),
      long_description: String::new(),
      // TODO: properly calculate cost
      cost: 0,
      template_key: UuidBuilder::from_random_bytes(rng.gen()).into_uuid(),
      base_color: self.base_color,
      stripe_color: self.stripe_color
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MissileEquipmentSummary {
  pub body_key: MissileBodyKey,
  pub seekers: SeekerStrategy,
  pub auxiliary_components: Vec<AuxiliaryKey>,
  pub avionics: AvionicsConfigured,
  pub warhead: Option<WarheadKey>
}

impl MissileEquipmentSummary {
  pub fn from_missile_template_summary(body_key: MissileBodyKey, missile_template_summary: MissileTemplateSummary) -> Result<Self, ModelError> {
    let seekers = missile_template_summary.seekers.ok_or(ModelError::MissileHasNoSeekers)?.to_basic();
    let auxiliary_components = missile_template_summary.auxiliary_components;
    let avionics = missile_template_summary.avionics.ok_or(ModelError::MissileHasNoAvionics)?;
    let warhead = match <[_; 1]>::try_from(missile_template_summary.warheads) {
      Ok([(warhead_key, _size)]) => Some(warhead_key),
      Err(..) => None
    };

    Ok(MissileEquipmentSummary {
      body_key,
      seekers,
      auxiliary_components,
      avionics,
      warhead
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileKind {
  Standard,
  Torpedo,
  Hybrid
}

impl MissileKind {
  pub const fn from_missile_body_key(missile_body_key: MissileBodyKey) -> Self {
    match missile_body_key {
      MissileBodyKey::SGM1Balestra => MissileKind::Standard,
      MissileBodyKey::SGM2Tempest => MissileKind::Standard,
      MissileBodyKey::SGMH2Cyclone => MissileKind::Hybrid,
      MissileBodyKey::SGMH3Atlatl => MissileKind::Hybrid,
      MissileBodyKey::SGT3Pilum => MissileKind::Torpedo,
      MissileBodyKey::CM4Container => MissileKind::Torpedo,
      MissileBodyKey::CMS4Container => MissileKind::Torpedo
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarheadKind {
  /// Blast Fragmentation and Blast Fragmentation EL
  AntiMissile,
  /// HE Impact
  AntiShip,
  /// HE Kinetic Penetrator
  AntiArmor
}

impl WarheadKind {
  pub const fn from_warhead_key(warhead_key: WarheadKey) -> Self {
    match warhead_key {
      WarheadKey::HEImpact => WarheadKind::AntiShip,
      WarheadKey::HEKineticPenetrator => WarheadKind::AntiArmor,
      WarheadKey::BlastFragmentation => WarheadKind::AntiMissile,
      WarheadKey::BlastFragmentationEL => WarheadKind::AntiMissile
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipEquipment {
  Intelligence,
  Illuminator,
  DeceptionModule,
  MissileIdentification,
  FireControl {
    sig_type: SigType,
  },
  Jamming {
    sig_type: SigType
  },
  Sensor {
    sig_type: SigType
  },
  Weapon {
    family: WeaponFamily
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct ShipEquipmentSummary {
  pub has_intelligence: bool,
  pub has_illuminator: bool,
  pub has_deception_module: bool,
  pub has_missile_identification: bool,
  pub fire_control: HashSet<SigType>,
  pub jamming: HashSet<SigType>,
  pub sensors: HashSet<SigType>,
  pub weapons: HashSet<WeaponFamily>,
  pub missile_cells: HashMap<MissileType, usize>
}

impl ShipEquipmentSummary {
  pub fn add_ship_equipment(&mut self, equipment: ShipEquipment) {
    match equipment {
      ShipEquipment::Intelligence => self.has_intelligence = true,
      ShipEquipment::Illuminator => self.has_illuminator = true,
      ShipEquipment::DeceptionModule => self.has_deception_module = true,
      ShipEquipment::MissileIdentification => self.has_missile_identification = true,
      ShipEquipment::FireControl { sig_type } => {
        self.fire_control.insert(sig_type);
      },
      ShipEquipment::Jamming { sig_type } => {
        self.jamming.insert(sig_type);
      },
      ShipEquipment::Sensor { sig_type } => {
        self.sensors.insert(sig_type);
      },
      ShipEquipment::Weapon { family } => {
        self.weapons.insert(family);
      }
    }
  }

  pub fn add_component_key(&mut self, component_key: ComponentKey) {
    if let ComponentKey::E15MasqueradeDeceptionModule = component_key {
      self.add_ship_equipment(ShipEquipment::DeceptionModule);
    } else if let ComponentKey::ES32ScryerMissileIDSystem = component_key {
      self.add_ship_equipment(ShipEquipment::MissileIdentification);
    } else if let ComponentKey::P60GrazerPDT = component_key {
      self.add_ship_equipment(ShipEquipment::Weapon {
        family: WeaponFamily::PointDefense(PointDefenseType::Beam)
      });
    } else {
      match component_key.component().variant {
        Some(ComponentVariant::FireControl { fire_control }) => {
          self.add_ship_equipment(ShipEquipment::FireControl { sig_type: fire_control.sig_type });
        },
        Some(ComponentVariant::Illuminator { .. }) => {
          self.add_ship_equipment(ShipEquipment::Illuminator);
        },
        Some(ComponentVariant::Intelligence { work_on_remote_tracks: true, .. }) => {
          self.add_ship_equipment(ShipEquipment::Intelligence);
        },
        Some(ComponentVariant::Jammer { sig_type, .. }) => {
          self.add_ship_equipment(ShipEquipment::Jamming { sig_type });
        },
        Some(ComponentVariant::Sensor { sig_type, can_lock, .. }) => {
          self.add_ship_equipment(ShipEquipment::Sensor { sig_type });
          if can_lock {
            self.add_ship_equipment(ShipEquipment::FireControl { sig_type });
          };
        },
        Some(ComponentVariant::SensorPassive) => {
          self.add_ship_equipment(ShipEquipment::Sensor { sig_type: SigType::Radar });
        },
        Some(ComponentVariant::WeaponBeam { role, .. }) => {
          if let Some(usage) = WeaponUsage::from_weapon_role(role) {
            let distance_realm = match component_key {
              ComponentKey::Mk90AuroraPDT => const { DistanceRealm::from_range(3000) },
              ComponentKey::Mk600BeamCannon => const { DistanceRealm::from_range(6000) },
              ComponentKey::Mk610BeamTurret => const { DistanceRealm::from_range(5000) },
              ComponentKey::P60GrazerPDT => const { DistanceRealm::from_range(1500) },
              _ => DistanceRealm::Near
            };

            let family = match usage {
              WeaponUsage::Offensive => WeaponFamily::EnergyBeam(distance_realm),
              WeaponUsage::Defensive => WeaponFamily::PointDefense(PointDefenseType::Beam)
            };

            self.add_ship_equipment(ShipEquipment::Weapon { family });
          };
        },
        Some(ComponentVariant::WeaponProjectile { munition_family: Some(munition_family), .. }) |
        Some(ComponentVariant::WeaponMissileLauncher { munition_family, .. }) |
        Some(ComponentVariant::WeaponMissileBank { munition_family, .. }) => {
          if let Some(family) = WeaponFamily::from_munition_family(munition_family) {
            self.add_ship_equipment(ShipEquipment::Weapon { family });
          };
        },
        Some(..) | None => ()
      };
    };
  }
}

impl Extend<ShipEquipment> for ShipEquipmentSummary {
  fn extend<T: IntoIterator<Item = ShipEquipment>>(&mut self, iter: T) {
    for equipment in iter {
      self.add_ship_equipment(equipment);
    };
  }
}

impl Extend<ComponentKey> for ShipEquipmentSummary {
  fn extend<T: IntoIterator<Item = ComponentKey>>(&mut self, iter: T) {
    for component_key in iter {
      self.add_component_key(component_key);
    };
  }
}

impl FromIterator<ShipEquipment> for ShipEquipmentSummary {
  fn from_iter<T: IntoIterator<Item = ShipEquipment>>(iter: T) -> Self {
    let mut summary = Self::default();
    summary.extend(iter);
    summary
  }
}

impl FromIterator<ComponentKey> for ShipEquipmentSummary {
  fn from_iter<T: IntoIterator<Item = ComponentKey>>(iter: T) -> Self {
    let mut summary = Self::default();
    summary.extend(iter);
    summary
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponUsage {
  Offensive,
  Defensive
}

impl WeaponUsage {
  pub const fn from_weapon_role(weapon_role: WeaponRole) -> Option<Self> {
    match weapon_role {
      WeaponRole::Offensive => Some(Self::Offensive),
      WeaponRole::Defensive => Some(Self::Defensive),
      WeaponRole::DualPurpose => Some(Self::Offensive),
      WeaponRole::Utility | WeaponRole::Decoy => None
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponFamily {
  EnergyBeam(DistanceRealm),
  EnergyPlasma(DistanceRealm),
  EnergyRailgun(DistanceRealm),
  Ballistic(DistanceRealm),
  PointDefense(PointDefenseType),
  StandardMissile(MissileSize),
  ContainerMissile,
  LoiteringMine,
  UnguidedRocket
}

impl WeaponFamily {
  pub fn from_munition_family(munition_family: MunitionFamily) -> Option<Self> {
    munition_family.max_range().map(|max_range| {
      let distance_realm = DistanceRealm::from_range(max_range as usize);
      match munition_family {
        MunitionFamily::BallisticMagnetic15mm => WeaponFamily::PointDefense(PointDefenseType::Railgun),
        MunitionFamily::BallisticChemical20mm => WeaponFamily::PointDefense(PointDefenseType::Defender),
        MunitionFamily::BallisticChemical50mmFlak => WeaponFamily::PointDefense(PointDefenseType::Flak),
        MunitionFamily::BallisticChemical100mm => WeaponFamily::Ballistic(distance_realm),
        MunitionFamily::BallisticChemical120mm => WeaponFamily::Ballistic(distance_realm),
        MunitionFamily::BallisticChemical250mm => WeaponFamily::Ballistic(distance_realm),
        MunitionFamily::BallisticMagnetic300mmRailgun => WeaponFamily::EnergyRailgun(distance_realm),
        MunitionFamily::BallisticMagnetic400mmPlasma => WeaponFamily::EnergyPlasma(distance_realm),
        MunitionFamily::BallisticChemical450mm => WeaponFamily::Ballistic(distance_realm),
        MunitionFamily::BallisticMagnetic500mmMassDriver => WeaponFamily::EnergyRailgun(distance_realm),
        MunitionFamily::BallisticChemical600mm => WeaponFamily::Ballistic(distance_realm),
        MunitionFamily::StandardMissileSize1 => WeaponFamily::StandardMissile(MissileSize::Size1),
        MunitionFamily::StandardMissileSize2 => WeaponFamily::StandardMissile(MissileSize::Size2),
        MunitionFamily::StandardMissileSize3 => WeaponFamily::StandardMissile(MissileSize::Size3),
        MunitionFamily::ContainerMissile => WeaponFamily::ContainerMissile,
        MunitionFamily::LoiteringMine => WeaponFamily::LoiteringMine,
        MunitionFamily::UnguidedRocket => WeaponFamily::UnguidedRocket
      }
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileType {
  StandardMissile(MissileSize),
  ContainerMissile,
  LoiteringMine,
  UnguidedRocket
}

impl MissileType {
  pub fn from_munition_family(munition_family: MunitionFamily) -> Option<Self> {
    match munition_family {
      MunitionFamily::StandardMissileSize1 => Some(MissileType::StandardMissile(MissileSize::Size1)),
      MunitionFamily::StandardMissileSize2 => Some(MissileType::StandardMissile(MissileSize::Size2)),
      MunitionFamily::StandardMissileSize3 => Some(MissileType::StandardMissile(MissileSize::Size3)),
      MunitionFamily::ContainerMissile => Some(MissileType::ContainerMissile),
      MunitionFamily::LoiteringMine => Some(MissileType::LoiteringMine),
      MunitionFamily::UnguidedRocket => Some(MissileType::UnguidedRocket),
      _ => None
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceRealm {
  /// Between 0m and 7km.
  Near = 1,
  /// Between 7km and 12km.
  Middle = 2,
  /// Beyond 12km.
  Far = 3
}

impl DistanceRealm {
  pub const fn from_range(range: usize) -> Self {
    match range {
      0..7000 => Self::Near,
      7000..12000 => Self::Middle,
      12000.. => Self::Far
    }
  }
}

impl FromStr for DistanceRealm {
  type Err = ParseDistanceRealmError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "near" => Ok(Self::Near),
      "middle" | "mid" => Ok(Self::Middle),
      "far" => Ok(Self::Far),
      _ => Err(ParseDistanceRealmError)
    }
  }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Default)]
#[error("failed to parse point defense type")]
pub struct ParseDistanceRealmError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PointDefenseType {
  // Mk90 Aurora
  // P60 Grazer
  Beam,
  // Mk20 Defender
  // P11 Pavise
  Defender,
  // Mk25 Rebound
  // Mk29 Stonewall
  // P20 Bastion
  Flak,
  // Mk95 Sarissa
  Railgun
}

impl FromStr for PointDefenseType {
  type Err = ParsePointDefenseTypeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "beam" | "laser" => Ok(Self::Beam),
      "defender" | "pavise" | "ciws" => Ok(Self::Defender),
      "flak" | "flakgun" | "flak gun" => Ok(Self::Flak),
      "rail" | "railgun" | "rail gun" => Ok(Self::Railgun),
      _ => Err(ParsePointDefenseTypeError)
    }
  }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Default)]
#[error("failed to parse point defense type")]
pub struct ParsePointDefenseTypeError;
