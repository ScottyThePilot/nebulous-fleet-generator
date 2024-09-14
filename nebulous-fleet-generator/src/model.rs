pub mod predicate;

use nebulous_data::data::components::{ComponentKey, ComponentVariant, SigType};
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::hulls::config::Variant;
use nebulous_data::data::munitions::{MunitionFamily, MunitionKey, WeaponRole};
use nebulous_data::data::{Faction, MissileSize};
use nebulous_data::format::{ComponentData, MissileTemplate, Ship};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::Extend;
use std::str::FromStr;



#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ShipName {
  #[serde(rename = "text")]
  Text(String),
  #[serde(rename = "key")]
  Key(String)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShipState {
  pub name: ShipName,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub author: Option<String>,
  pub role: String,
  pub hull_type: HullKey,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub hull_config: Option<[Variant; 3]>,
  pub cost_budget_total: usize,
  pub cost_budget_spare: usize,
  pub components: Box<[Option<ComponentState>]>,
  pub equipment_summary: EquipmentSummary
}

impl ShipState {
  pub fn from_ship(ship: &Ship, missile_templates: &[MissileTemplate]) -> Self {
    let mut component_map = HashMap::new();
    let mut equipment_summary = EquipmentSummary::default();

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

      let (load, identity_option) = match hull_socket.component_data {
        Some(ComponentData::BulkMagazineData { ref load }) => (Some(load), None),
        Some(ComponentData::CellLauncherData { ref missile_load }) => (Some(missile_load), None),
        Some(ComponentData::ResizableCellLauncherData { ref missile_load, .. }) => (Some(missile_load), None),
        Some(ComponentData::DeceptionComponentData { identity_option }) => (None, Some(identity_option)),
        None => (None, None)
      };

      let magazine_contents = load.map(|load| {
        let mut magazine_contents = BTreeMap::new();
        for magazine_save_data in load.iter() {
          if let Ok(munition_key) = magazine_save_data.munition_key.parse::<MunitionKey>() {
            *magazine_contents.entry(munition_key).or_default() += magazine_save_data.quantity;
          };
        };

        magazine_contents
      });

      component_map.insert(hull_socket.key, ComponentState {
        component_key: hull_socket.component_name,
        identity_option,
        magazine_contents
      });
    };

    let components = hull.sockets.iter()
      .map(|hull_socket| component_map.remove(&hull_socket.save_key))
      .collect::<Box<[Option<ComponentState>]>>();

    let hull_config = ship.hull_config.as_ref().zip(hull.config_template)
      .and_then(|(hull_config, config_template)| config_template.get_variants(hull_config));

    let costs = ship.calculate_costs(missile_templates);

    ShipState {
      name: ShipName::Text(ship.name.clone()),
      author: None,
      role: "unknown".to_string(),
      hull_type: ship.hull_type,
      hull_config,
      cost_budget_total: costs.total(),
      cost_budget_spare: costs.missiles,
      components,
      equipment_summary
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComponentState {
  pub component_key: ComponentKey,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub identity_option: Option<usize>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub magazine_contents: Option<BTreeMap<MunitionKey, usize>>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissileData {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Equipment {
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
pub struct EquipmentSummary {
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

impl EquipmentSummary {
  pub fn add_equipment(&mut self, equipment: Equipment) {
    match equipment {
      Equipment::Intelligence => self.has_intelligence = true,
      Equipment::Illuminator => self.has_illuminator = true,
      Equipment::DeceptionModule => self.has_deception_module = true,
      Equipment::MissileIdentification => self.has_missile_identification = true,
      Equipment::FireControl { sig_type } => {
        self.fire_control.insert(sig_type);
      },
      Equipment::Jamming { sig_type } => {
        self.jamming.insert(sig_type);
      },
      Equipment::Sensor { sig_type } => {
        self.sensors.insert(sig_type);
      },
      Equipment::Weapon { family } => {
        self.weapons.insert(family);
      }
    }
  }

  pub fn add_component_key(&mut self, component_key: ComponentKey) {
    if let ComponentKey::E15MasqueradeDeceptionModule = component_key {
      self.add_equipment(Equipment::DeceptionModule);
    } else if let ComponentKey::ES32ScryerMissileIDSystem = component_key {
      self.add_equipment(Equipment::MissileIdentification);
    } else if let ComponentKey::P60GrazerPDT = component_key {
      self.add_equipment(Equipment::Weapon {
        family: WeaponFamily::PointDefense(PointDefenseType::Beam)
      });
    } else {
      match component_key.component().variant {
        Some(ComponentVariant::FireControl { fire_control }) => {
          self.add_equipment(Equipment::FireControl { sig_type: fire_control.sig_type });
        },
        Some(ComponentVariant::Illuminator { .. }) => {
          self.add_equipment(Equipment::Illuminator);
        },
        Some(ComponentVariant::Intelligence { work_on_remote_tracks: true, .. }) => {
          self.add_equipment(Equipment::Intelligence);
        },
        Some(ComponentVariant::Jammer { sig_type, .. }) => {
          self.add_equipment(Equipment::Jamming { sig_type });
        },
        Some(ComponentVariant::Sensor { sig_type, can_lock, .. }) => {
          self.add_equipment(Equipment::Sensor { sig_type });
          if can_lock {
            self.add_equipment(Equipment::FireControl { sig_type });
          };
        },
        Some(ComponentVariant::SensorPassive) => {
          self.add_equipment(Equipment::Sensor { sig_type: SigType::Radar });
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

            self.add_equipment(Equipment::Weapon { family });
          };
        },
        Some(ComponentVariant::WeaponProjectile { munition_family: Some(munition_family), .. }) |
        Some(ComponentVariant::WeaponMissileLauncher { munition_family, .. }) |
        Some(ComponentVariant::WeaponMissileBank { munition_family, .. }) => {
          if let Some(family) = WeaponFamily::from_munition_family(munition_family) {
            self.add_equipment(Equipment::Weapon { family });
          };
        },
        Some(..) | None => ()
      };
    };
  }
}

impl Extend<Equipment> for EquipmentSummary {
  fn extend<T: IntoIterator<Item = Equipment>>(&mut self, iter: T) {
    for equipment in iter {
      self.add_equipment(equipment);
    };
  }
}

impl Extend<ComponentKey> for EquipmentSummary {
  fn extend<T: IntoIterator<Item = ComponentKey>>(&mut self, iter: T) {
    for component_key in iter {
      self.add_component_key(component_key);
    };
  }
}

impl FromIterator<Equipment> for EquipmentSummary {
  fn from_iter<T: IntoIterator<Item = Equipment>>(iter: T) -> Self {
    let mut summary = Self::default();
    summary.extend(iter);
    summary
  }
}

impl FromIterator<ComponentKey> for EquipmentSummary {
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
  Near = 1, Middle = 2, Far = 3
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
