pub mod key;

use crate::data::{Faction, MissileSize};
use crate::data::components::ComponentKey;
use crate::data::hulls::config::Variant;
use crate::data::hulls::HullKey;
use crate::data::missiles::{AuxiliaryKey, AvionicsKey, Maneuvers, WarheadKey};
use crate::data::missiles::bodies::MissileBodyKey;
use crate::data::missiles::seekers::{SeekerMode, SeekerKey, SeekerStrategy};
use crate::data::missiles::engines::EngineSettings;
use crate::data::munitions::MunitionKey;
use crate::utils::Size;
use self::key::Key;

use bytemuck::Contiguous;
#[cfg(feature = "rand")]
use rand::Rng;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use xml::{DeserializeElement, DeserializeNodes, SerializeElement, SerializeNodes, Element, Nodes, Attributes};

#[doc(no_inline)]
pub use uuid::Uuid;
#[doc(no_inline)]
pub use xml::{read_nodes, write_nodes};

use std::convert::Infallible;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroUsize as zsize;
use std::ops::{Add, AddAssign, Index};
use std::str::FromStr;



pub const CURRENT_FLEET_VERISON: usize = 3;

macro_rules! chain_iter {
  ($expr0:expr $(,)?) => ($expr0.into_iter());
  ($expr0:expr, $($expr:expr),* $(,)?) => ($expr0.into_iter()$(.chain($expr))*);
}

#[derive(Debug, Error)]
pub enum FormatError {
  #[error(transparent)]
  XmlError(#[from] xml::Error),
  #[error("unknown hull config type {0:?}")]
  UnknownHullConfigType(Box<str>),
  #[error("unknown hull component data type {0:?}")]
  UnknownComponentDataType(Box<str>),
  #[error("unknown missile settings type {0:?}")]
  UnknownMissileSettingsType(Box<str>)
}

impl From<xml::DeserializeErrorWrapper<FormatError>> for FormatError {
  fn from(value: xml::DeserializeErrorWrapper<FormatError>) -> Self {
    value.convert()
  }
}

impl From<xml::DeserializeErrorWrapper<xml::Error>> for FormatError {
  fn from(value: xml::DeserializeErrorWrapper<xml::Error>) -> Self {
    FormatError::XmlError(value.convert())
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Root<T> {
  pub element: T
}

impl<T> DeserializeNodes for Root<T> where T: DeserializeElement<Error = FormatError> {
  type Error = FormatError;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    nodes.try_into_one_element()
      .map_err(FormatError::from)
      .and_then(T::deserialize_element)
      .map(|element| Root { element })
  }
}

impl<T> SerializeNodes for Root<T> where T: SerializeElement<Error = Infallible> {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    let mut element = self.element.serialize_element()?;
    let mut attributes = Vec::from(std::mem::take(&mut element.attributes).list);
    attributes.push(("xmlns:xsd".into(), "http://www.w3.org/2001/XMLSchema".into()));
    attributes.push(("xmlns:xsi".into(), "http://www.w3.org/2001/XMLSchema-instance".into()));
    element.attributes = Attributes::from(attributes);
    Ok(Nodes::new_one(element))
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Fleet {
  pub name: String,
  pub total_points: usize,
  pub faction_key: Faction,
  pub description: Option<String>,
  pub ships: Vec<Ship>,
  pub missile_types: Vec<MissileTemplate>
}

impl Fleet {
  #[cfg(feature = "rand")]
  pub fn double<R: Rng + ?Sized>(&mut self, rng: &mut R) {
    self.name.push_str(" (2x)");
    self.total_points *= 2;
    for i in 0..self.ships.len() {
      self.ships.push(self.ships[i].dupe(rng));
    };
  }

  pub fn calculate_costs(&self, missile_templates: &[MissileTemplate]) -> Costs {
    let mut costs = Costs::default();
    for ship in self.ships.iter() {
      costs += ship.calculate_costs(missile_templates);
    };

    costs
  }
}

impl DeserializeElement for Fleet {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("Fleet")?;
    let [name, total_points, faction_key, description, ships, missile_types] = element.children
      .find_elements(["Name", "TotalPoints", "FactionKey", "Description", "Ships", "MissileTypes"])?;

    let name = name.ok_or(xml::Error::missing_element("Name"))?.children.deserialize::<String>()?;
    let total_points = total_points.ok_or(xml::Error::missing_element("TotalPoints"))?.children.deserialize::<usize>()?;
    let faction_key = faction_key.ok_or(xml::Error::missing_element("FactionKey"))?.children.deserialize::<Faction>()?;
    let description = description.map(|description| description.children.deserialize::<String>()).transpose()?.filter(|d| !d.is_empty());
    let ships = ships.ok_or(xml::Error::missing_element("Ships"))?.children.deserialize::<Vec<Ship>>()?;
    let missile_types = missile_types.map(|missile_types| missile_types.children.deserialize::<Vec<MissileTemplate>>()).transpose()?.unwrap_or_else(Vec::new);

    Ok(Fleet { name, total_points, faction_key, description, ships, missile_types })
  }
}

impl SerializeElement for Fleet {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let name = Element::new("Name", self.name.serialize_nodes()?);
    let version = Element::new("Version", Nodes::new_text(CURRENT_FLEET_VERISON.to_string()));
    let total_points = Element::new("TotalPoints", self.total_points.serialize_nodes()?);
    let faction_key = Element::new("FactionKey", self.faction_key.serialize_nodes()?);
    let description = self.description.map(String::serialize_nodes).transpose()?
      .map(|description| Element::new("Description", description));
    let sort_override_order = Element::with_attributes("SortOverrideOrder", xml::attributes!("xsi:nil" = "true"), Nodes::new());
    let ships = Element::new("Ships", self.ships.serialize_nodes()?);
    let missile_types = Element::new("MissileTypes", self.missile_types.serialize_nodes()?);

    let nodes = Nodes::from_iter(chain_iter!([name, version, total_points, faction_key], description, [sort_override_order, ships, missile_types]));
    Ok(Element::new("Fleet", nodes))
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Ship {
  pub key: Uuid,
  pub name: String,
  pub cost: usize,
  pub callsign: Option<String>,
  pub number: usize,
  pub hull_type: HullKey,
  pub hull_config: Option<Box<HullConfig>>,
  pub socket_map: Vec<HullSocket>,
  pub weapon_groups: Vec<WeaponGroup>,
  pub initial_formation: Option<InitialFormation>,
  pub missile_types: Vec<MissileTemplate>
}

impl Ship {
  pub fn calculate_costs(&self, missile_templates: &[MissileTemplate]) -> Costs {
    let mut costs = Costs::default();

    let hull = self.hull_type.hull();
    costs.hulls += hull.point_cost;

    let mut component_compounding_groups = HashMap::new();
    for hull_socket in self.socket_map.iter() {
      if let Some(hull_socket_definition) = hull.get_socket(hull_socket.key) {
        let component = hull_socket.component_name.component();

        if let Some(cost) = component.cost(hull_socket_definition.size) {
          if let Some(compounding_cost) = component.compounding {
            component_compounding_groups.entry(compounding_cost)
              .or_insert_with(Vec::new).push(cost);
          } else {
            costs.components += cost;
          };
        };

        let load = hull_socket.component_data.as_ref()
          .and_then(ComponentData::get_load).unwrap_or(&[]);
        for &MagazineSaveData { ref munition_key, quantity, .. } in load {
          if let Ok(munition_key) = munition_key.parse::<MunitionKey>() {
            costs.ammunition += munition_key.munition().point_cost * quantity;
          } else if let Some(munition_key) = munition_key.strip_prefix("$MODMIS$/") {
            if let Some(missile_template) = missile_templates.iter().find(|missile_template| {
              missile_template.associated_template_name.as_deref() == Some(munition_key)
            }) {
              costs.missiles += missile_template.calculate_cost() * quantity;
            };
          };
        };
      };
    };

    for (compounding_cost_class, mut component_costs) in component_compounding_groups {
      component_costs.sort();

      // first instance free appears to deduct the cost of the component with the most expensive base cost
      if compounding_cost_class.first_instance_free() {
        component_costs.pop();
      };

      let multiplier = compounding_cost_class.multiplier();
      if multiplier == 0 {
        costs.components += component_costs.into_iter().sum::<usize>();
      } else {
        // compounding appears to place more expensive components earlier
        // in the calculation, thus giving them lower compounding costs
        for (cost, i) in component_costs.into_iter().rev().enumerate() {
          let modifier = if i == 0 { 1 } else { i * multiplier };
          costs.components += cost * modifier;
        };
      };
    };

    costs
  }

  /// Creates a duplicate of this ship.
  /// Keys will be randomized so that placing this ship into a fleet with the original produces a valid fleet.
  #[cfg(feature = "rand")]
  pub fn dupe<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
    let key = crate::utils::gen_uuid(rng);

    let hull_config = self.hull_config.as_deref().and_then(|hull_config| {
      hull_config.recycle(self.hull_type, rng).map(Box::new)
    });

    let hull_config = hull_config.or_else(|| {
      self.hull_type.hull().config_template
        .map(|template| Box::new(rng.sample(template)))
    });

    let mut socket_map = self.socket_map.clone();
    for hull_socket in socket_map.iter_mut() {
      if let Some(load) = hull_socket.component_data.as_mut().and_then(ComponentData::get_load_mut) {
        for magazine_save_data in load.iter_mut() {
          magazine_save_data.magazine_key = rng.gen::<Key>();
        };
      };
    };

    Self {
      key,
      name: self.name.clone(),
      cost: self.cost,
      callsign: self.callsign.clone(),
      number: self.number,
      hull_type: self.hull_type,
      hull_config,
      socket_map,
      weapon_groups: self.weapon_groups.clone(),
      initial_formation: self.initial_formation,
      missile_types: self.missile_types.clone()
    }
  }
}

impl DeserializeElement for Ship {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("Ship")?;
    let [key, name, cost, callsign, number, hull_type, hull_config, socket_map, weapon_groups, initial_formation, missile_types] = element.children
      .find_elements(["Key", "Name", "Cost", "Callsign", "Number", "HullType", "HullConfig", "SocketMap", "WeaponGroups", "InitialFormation", "TemplateMissileTypes"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Uuid>()?;
    let name = name.ok_or(xml::Error::missing_element("Name"))?.children.deserialize::<String>()?;
    let cost = cost.ok_or(xml::Error::missing_element("Cost"))?.children.deserialize::<usize>()?;
    let callsign = callsign.map(|callsign| callsign.children.deserialize::<String>()).transpose()?.filter(|c| !c.is_empty());
    let number = number.ok_or(xml::Error::missing_element("Number"))?.children.deserialize::<usize>()?;
    let hull_type = hull_type.ok_or(xml::Error::missing_element("HullType"))?.children.deserialize::<HullKey>()?;
    let hull_config = hull_config.map(|hull_config| hull_config.deserialize::<Box<HullConfig>>()).transpose()?;
    let socket_map = socket_map.ok_or(xml::Error::missing_element("SocketMap"))?.children.deserialize::<Vec<HullSocket>>()?;
    let weapon_groups = weapon_groups.ok_or(xml::Error::missing_element("WeaponGroups"))?.children.deserialize::<Vec<WeaponGroup>>()?;
    let initial_formation = initial_formation.map(InitialFormation::deserialize_element).transpose()?;
    let missile_types = missile_types.map(|element| element.children.deserialize::<Vec<MissileTemplate>>()).transpose()?.unwrap_or_else(Vec::new);

    Ok(Ship { key, name, cost, callsign, number, hull_type, hull_config, socket_map, weapon_groups, initial_formation, missile_types })
  }
}

impl SerializeElement for Ship {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let save_id = Element::with_attributes("SaveID", xml::attributes!("xsi:nil" = "true"), Nodes::default());
    let key = Element::new("Key", self.key.serialize_nodes()?);
    let name = Element::new("Name", self.name.serialize_nodes()?);
    let cost = Element::new("Cost", self.cost.serialize_nodes()?);
    let callsign = Element::new("Callsign", self.callsign.map_or(Ok(Nodes::default()), String::serialize_nodes)?);
    let number = Element::new("Number", self.number.serialize_nodes()?);
    let symbol_option = Element::new("SymbolOption", Nodes::new_text("0"));
    let hull_type = Element::new("HullType", self.hull_type.serialize_nodes()?);
    let hull_config = self.hull_config.map(<Box<HullConfig>>::serialize_element).transpose()?;
    let socket_map = Element::new("SocketMap", self.socket_map.serialize_nodes()?);
    let weapon_groups = Element::new("WeaponGroups", self.weapon_groups.serialize_nodes()?);
    let initial_formation = self.initial_formation.map(InitialFormation::serialize_element).transpose()?;
    let missile_types = Element::new("TemplateMissileTypes", self.missile_types.serialize_nodes()?);

    let nodes = Nodes::from_iter(chain_iter!(
      [save_id, key, name, cost, callsign, number, symbol_option, hull_type], hull_config,
      [socket_map, weapon_groups], initial_formation, std::iter::once(missile_types)
    ));

    Ok(Element::new("Ship", nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct HullSocket {
  pub key: Key,
  pub component_name: ComponentKey,
  pub component_data: Option<ComponentData>
}

impl DeserializeElement for HullSocket {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    let [key, component_name, component_data] = element.children
      .find_elements(["Key", "ComponentName", "ComponentData"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Key>()?;
    let component_name = component_name.ok_or(xml::Error::missing_element("ComponentName"))?.children.deserialize::<ComponentKey>()?;
    let component_data = component_data.map(Element::deserialize::<ComponentData>).transpose()?;

    Ok(HullSocket { key, component_name, component_data })
  }
}

impl SerializeElement for HullSocket {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let key = Element::new("Key", self.key.serialize_nodes()?);
    let component_name = Element::new("ComponentName", Nodes::new_text(self.component_name.save_key()));
    let component_data = self.component_data.map(ComponentData::serialize_element).transpose()?;
    let nodes = Nodes::from_iter(chain_iter!([key, component_name], component_data));
    Ok(Element::new("HullSocket", nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum ComponentData {
  BulkMagazineData {
    load: Vec<MagazineSaveData>
  },
  CellLauncherData {
    missile_load: Vec<MagazineSaveData>
  },
  ResizableCellLauncherData {
    missile_load: Vec<MagazineSaveData>,
    configured_size: Vector2<usize>
  },
  DeceptionComponentData {
    identity_option: usize
  }
}

impl ComponentData {
  pub fn get_load(&self) -> Option<&[MagazineSaveData]> {
    match self {
      ComponentData::BulkMagazineData { load } => Some(load),
      ComponentData::CellLauncherData { missile_load } => Some(missile_load),
      ComponentData::ResizableCellLauncherData { missile_load, .. } => Some(missile_load),
      ComponentData::DeceptionComponentData { .. } => None
    }
  }

  pub fn get_load_mut(&mut self) -> Option<&mut Vec<MagazineSaveData>> {
    match self {
      ComponentData::BulkMagazineData { load } => Some(load),
      ComponentData::CellLauncherData { missile_load } => Some(missile_load),
      ComponentData::ResizableCellLauncherData { missile_load, .. } => Some(missile_load),
      ComponentData::DeceptionComponentData { .. } => None
    }
  }

  pub fn into_load(self) -> Option<Vec<MagazineSaveData>> {
    match self {
      ComponentData::BulkMagazineData { load } => Some(load),
      ComponentData::CellLauncherData { missile_load } => Some(missile_load),
      ComponentData::ResizableCellLauncherData { missile_load, .. } => Some(missile_load),
      ComponentData::DeceptionComponentData { .. } => None
    }
  }
}

impl DeserializeElement for ComponentData {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("ComponentData")?;
    let [xsi_type] = element.attributes.find_attributes(["xsi:type"])?;
    let xsi_type = xsi_type.ok_or(xml::Error::missing_attribute("xsi:type"))?;

    match xsi_type.as_ref() {
      "BulkMagazineData" => {
        let [load] = element.children.find_elements(["Load"])?;
        let load = load.ok_or(xml::Error::missing_element("Load"))?
          .children.deserialize::<Vec<MagazineSaveData>>()?;
        Ok(ComponentData::BulkMagazineData { load })
      },
      "CellLauncherData" => {
        let [missile_load] = element.children.find_elements(["MissileLoad"])?;
        let missile_load = missile_load.ok_or(xml::Error::missing_element("MissileLoad"))?
          .children.deserialize::<Vec<MagazineSaveData>>()?;
        Ok(ComponentData::CellLauncherData { missile_load })
      },
      "ResizableCellLauncherData" => {
        let [missile_load, configured_size] = element.children.find_elements(["MissileLoad", "ConfiguredSize"])?;
        let missile_load = missile_load.ok_or(xml::Error::missing_element("MissileLoad"))?
          .children.deserialize::<Vec<MagazineSaveData>>()?;
        let configured_size = configured_size.ok_or(xml::Error::missing_element("ConfiguredSize"))?
          .children.deserialize::<Vector2<usize>>()?;
        Ok(ComponentData::ResizableCellLauncherData { missile_load, configured_size })
      },
      "DeceptionComponentData" => {
        let [identity_option] = element.children.find_elements(["IdentityOption"])?;
        let identity_option = identity_option.ok_or(xml::Error::missing_element("IdentityOption"))?
          .children.deserialize::<usize>()?;
        Ok(ComponentData::DeceptionComponentData { identity_option })
      },
      _ => Err(FormatError::UnknownComponentDataType(xsi_type.clone()))
    }
  }
}

impl SerializeElement for ComponentData {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let (xsi_type, nodes) = match self {
      ComponentData::BulkMagazineData { load } => ("BulkMagazineData", {
        Nodes::new_one(Element::new("Load", load.serialize_nodes()?))
      }),
      ComponentData::CellLauncherData { missile_load } => ("CellLauncherData", {
        Nodes::new_one(Element::new("MissileLoad", missile_load.serialize_nodes()?))
      }),
      ComponentData::ResizableCellLauncherData { missile_load, configured_size } => ("ResizableCellLauncherData", {
        let missile_load = Element::new("MissileLoad", missile_load.serialize_nodes()?);
        let configured_size = Element::new("ConfiguredSize", configured_size.serialize_nodes()?);
        Nodes::from_iter([missile_load, configured_size])
      }),
      ComponentData::DeceptionComponentData { identity_option } => ("DeceptionComponentData", {
        Nodes::new_one(Element::new("IdentityOption", identity_option.serialize_nodes()?))
      })
    };

    Ok(Element::with_attributes("ComponentData", xml::attributes!("xsi:type" = xsi_type), nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MagazineSaveData {
  pub magazine_key: Key,
  // This is here rather than MunitionKey since these can reference custom missiles, which have unique names.
  pub munition_key: Box<str>,
  pub quantity: usize
}

impl DeserializeElement for MagazineSaveData {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("MagSaveData")?;
    let [magazine_key, munition_key, quantity] = element.children
      .find_elements(["MagazineKey", "MunitionKey", "Quantity"])?;

    let magazine_key = magazine_key.ok_or(xml::Error::missing_element("MagazineKey"))?
      .children.deserialize::<Key>()?;
    let munition_key = munition_key.ok_or(xml::Error::missing_element("MunitionKey"))?
      .children.deserialize::<String>()?.into_boxed_str();
    let quantity = quantity.ok_or(xml::Error::missing_element("Quantity"))?
      .children.deserialize::<usize>()?;

    Ok(MagazineSaveData { magazine_key, munition_key, quantity })
  }
}

impl SerializeElement for MagazineSaveData {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let magazine_key = Element::new("MagazineKey", self.magazine_key.serialize_nodes()?);
    let munition_key = Element::new("MunitionKey", self.munition_key.serialize_nodes()?);
    let quantity = Element::new("Quantity", self.quantity.serialize_nodes()?);

    Ok(Element::new("MagSaveData", Nodes::from_iter([magazine_key, munition_key, quantity])))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct WeaponGroup {
  pub name: String,
  pub members: Vec<Key>
}

impl DeserializeElement for WeaponGroup {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("WepGroup")?;
    let [name] = element.attributes.find_attributes(["Name"])?;
    let name = name.ok_or(xml::Error::missing_attribute("Name"))?;

    let member_keys = element.children.try_into_one_element()?;
    member_keys.expect_named("MemberKeys")?;

    let members = member_keys.children.deserialize_named_elements::<Key, Vec<Key>>("string")?;

    Ok(WeaponGroup { name: String::from(name), members })
  }
}

impl SerializeElement for WeaponGroup {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let member_keys = xml::serialize_named_elements(self.members, "string")?;
    let nodes = Nodes::new_one(Element::new("MemberKeys", member_keys));
    let weapon_group = Element::with_attributes("WepGroup", xml::attributes!("Name" = self.name), nodes);
    Ok(weapon_group)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InitialFormation {
  pub guide_key: Uuid,
  pub relative_position: Vector3<f32>
}

impl DeserializeElement for InitialFormation {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("InitialFormation")?;
    let [guide_key, relative_position] = element.children.find_elements(["GuideKey", "RelativePosition"])?;
    let guide_key = guide_key.ok_or(xml::Error::missing_element("GuideKey"))?.children.deserialize::<Uuid>()?;
    let relative_position = relative_position.ok_or(xml::Error::missing_element("RelativePosition"))?.children.deserialize::<Vector3<f32>>()?;
    Ok(InitialFormation { guide_key, relative_position })
  }
}

impl SerializeElement for InitialFormation {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    Ok(Element::new("InitialFormation", Nodes::from_iter([
      Element::new("GuideKey", self.guide_key.serialize_nodes()?),
      Element::new("RelativePosition", self.relative_position.serialize_nodes()?)
    ])))
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum HullConfig {
  RandomHullConfiguration {
    primary_structure: [SegmentConfiguration; 3],
    secondary_structure: SecondaryStructureConfig,
    hull_tint: Color,
    texture_variation: Vector3<f32>
  }
}

impl HullConfig {
  #[cfg(feature = "rand")]
  pub fn new<R: Rng + ?Sized>(hull_key: HullKey, rng: &mut R) -> Option<Self> {
    Self::from_variants(hull_key, rng.gen::<[Variant; 3]>(), rng)
  }

  #[cfg(feature = "rand")]
  pub fn from_variants<R: Rng + ?Sized>(hull_key: HullKey, variants: [Variant; 3], rng: &mut R) -> Option<Self> {
    hull_key.hull().config_template.map(|config_template| {
      rng.sample(config_template.with_variants(variants))
    })
  }

  #[cfg(feature = "rand")]
  pub fn recycle<R: Rng + ?Sized>(&self, hull_key: HullKey, rng: &mut R) -> Option<Self> {
    hull_key.hull().config_template.and_then(|config_template| {
      let variants = config_template.get_variants(self)?;
      Some(rng.sample(config_template.with_variants(variants)))
    })
  }
}

impl DeserializeElement for HullConfig {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("HullConfig")?;
    let [xsi_type] = element.attributes.find_attributes(["xsi:type"])?;
    let xsi_type = xsi_type.ok_or(xml::Error::missing_attribute("xsi:type"))?;

    match xsi_type.as_ref() {
      "RandomHullConfiguration" => {
        let [primary_structure, secondary_structure, hull_tint, texture_variation] = element.children
          .find_elements(["PrimaryStructure", "SecondaryStructure", "HullTint", "TextureVariation"])?;

        let primary_structure = primary_structure.ok_or(xml::Error::missing_element("PrimaryStructure"))?
          .children.deserialize::<[SegmentConfiguration; 3]>()?;
        let secondary_structure = secondary_structure.ok_or(xml::Error::missing_element("SecondaryStructure"))?
          .children.try_into_one_element()?.deserialize::<SecondaryStructureConfig>()?;
        let hull_tint = hull_tint.ok_or(xml::Error::missing_element("HullTint"))?
          .children.deserialize::<Color>()?;
        let texture_variation = texture_variation.ok_or(xml::Error::missing_element("TextureVariation"))?
          .children.deserialize::<Vector3<f32>>()?;

        Ok(HullConfig::RandomHullConfiguration { primary_structure, secondary_structure, hull_tint, texture_variation })
      },
      _ => Err(FormatError::UnknownHullConfigType(xsi_type.clone()))
    }
  }
}

impl SerializeElement for HullConfig {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    match self {
      HullConfig::RandomHullConfiguration { primary_structure, secondary_structure, hull_tint, texture_variation } => {
        let primary_structure = Element::new("PrimaryStructure", primary_structure.serialize_nodes()?);
        let secondary_structure = Element::new("SecondaryStructure", Nodes::new_one(secondary_structure.serialize_element()?));
        let hull_tint = Element::new("HullTint", hull_tint.serialize_nodes()?);
        let texture_variation = Element::new("TextureVariation", texture_variation.serialize_nodes()?);
        let nodes = Nodes::from_iter([primary_structure, secondary_structure, hull_tint, texture_variation]);
        Ok(Element::with_attributes("HullConfig", xml::attributes!("xsi:type" = "RandomHullConfiguration"), nodes))
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SegmentConfiguration {
  pub key: Uuid,
  pub dressing: Vec<usize>
}

impl DeserializeElement for SegmentConfiguration {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("SegmentConfiguration")?;
    let [key, dressing] = element.children.find_elements(["Key", "Dressing"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Uuid>()?;
    let dressing = dressing.ok_or(xml::Error::missing_element("Dressing"))?
      .children.deserialize_named_elements::<usize, Vec<usize>>("int")?;

    Ok(SegmentConfiguration { key, dressing })
  }
}

impl SerializeElement for SegmentConfiguration {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let key = Element::new("Key", self.key.serialize_nodes()?);
    let dressing = Element::new("Dressing", xml::serialize_named_elements(self.dressing, "int")?);
    Ok(Element::new("SegmentConfiguration", Nodes::from_iter([key, dressing])))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SecondaryStructureConfig {
  pub key: Uuid,
  pub segment: usize,
  pub snap_point: usize
}

impl DeserializeElement for SecondaryStructureConfig {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("SecondaryStructureConfig")?;
    let [key, segment, snap_point] = element.children.find_elements(["Key", "Segment", "SnapPoint"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Uuid>()?;
    let segment = segment.ok_or(xml::Error::missing_element("Segment"))?.children.deserialize::<usize>()?;
    let snap_point = snap_point.ok_or(xml::Error::missing_element("SnapPoint"))?.children.deserialize::<usize>()?;

    Ok(SecondaryStructureConfig { key, segment, snap_point })
  }
}

impl SerializeElement for SecondaryStructureConfig {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let key = Element::new("Key", self.key.serialize_nodes()?);
    let segment = Element::new("Segment", self.segment.serialize_nodes()?);
    let snap_point = Element::new("SnapPoint", self.snap_point.serialize_nodes()?);
    Ok(Element::new("SecondaryStructureConfig", Nodes::from_iter([key, segment, snap_point])))
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileTemplateContents {
  pub body_key: MissileBodyKey,
  pub seekers: Option<SeekerStrategy<SeekerKey>>,
  pub warheads: Vec<(WarheadKey, zsize)>,
  pub avionics: Option<(AvionicsKey, Maneuvers, Option<DefensiveDoctrine>)>,
  pub auxiliary_components: Vec<AuxiliaryKey>,
  pub engine_settings: Vec<(EngineSettings, zsize)>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileTemplate {
  pub associated_template_name: Option<String>,
  pub designation: String,
  pub nickname: String,
  pub description: String,
  pub long_description: String,
  pub cost: usize,
  pub body_key: MissileBodyKey,
  pub template_key: Uuid,
  pub base_color: Color,
  pub stripe_color: Color,
  pub sockets: Vec<MissileSocket>
}

impl MissileTemplate {
  pub fn calculate_cost(&self) -> usize {
    // TODO: actually calculate the cost, perhaps we should not trust this number
    self.cost
  }

  pub fn contents(&self) -> MissileTemplateContents {
    let mut seekers = Vec::new();
    let mut warheads = Vec::new();
    let mut avionics = None;
    let mut auxiliary_components = Vec::new();
    let mut engine_settings = Vec::new();
    for &MissileSocket { installed_component, size } in self.sockets.iter() {
      let Some(MissileComponent { component_key, settings }) = installed_component else { continue };
      match (component_key, settings) {
        (Some(component_key), Some(
          MissileComponentSettings::ActiveSeekerSettings { mode, .. } |
          MissileComponentSettings::PassiveSeekerSettings { mode, .. } |
          MissileComponentSettings::CommandSeekerSettings { mode }
        )) => {
          if let Some(seeker_key) = component_key.to_seeker_key() {
            seekers.push((seeker_key, mode));
          };
        },
        (Some(MissileComponentKey::FixedAntiRadiationSeeker), Some(
          MissileComponentSettings::PassiveARHSeekerSettings { mode, home_on_jam, .. }
        )) => {
          seekers.push((match home_on_jam {
            true => SeekerKey::FixedHomeOnJam,
            false => SeekerKey::FixedAntiRadiation
          }, mode));
        },
        (Some(component_key), Some(
          MissileComponentSettings::DirectGuidanceSettings { maneuvers, defensive_doctrine, .. } |
          MissileComponentSettings::CruiseGuidanceSettings { maneuvers, defensive_doctrine, .. }
        )) => {
          if let Some(avionics_key) = component_key.to_avionics_key() {
            avionics = Some((avionics_key, maneuvers, defensive_doctrine));
          };
        },
        (Some(component_key), _) => {
          if let Some(warhead_key) = component_key.to_warhead_key() {
            warheads.push((warhead_key, size));
          } else if let Some(auxiliary_key) = component_key.to_auxiliary_key() {
            auxiliary_components.push(auxiliary_key);
          };
        },
        (None, Some(MissileComponentSettings::MissileEngineSettings { balance_values })) => {
          engine_settings.push((balance_values, size));
        },
        (None, _) => ()
      };
    };

    let seekers = if seekers.is_empty() {
      let primary = seekers.remove(0).0;
      let secondaries = seekers.into_boxed_slice();
      Some(SeekerStrategy::new(primary, secondaries))
    } else {
      None
    };

    MissileTemplateContents {
      body_key: self.body_key,
      seekers,
      warheads,
      avionics,
      auxiliary_components,
      engine_settings
    }
  }
}

impl DeserializeElement for MissileTemplate {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("MissileTemplate")?;

    let [
      associated_template_name, designation, nickname, description, long_description,
      cost, body_key, template_key, base_color, stripe_color, sockets
    ] = element.children.find_elements([
      "AssociatedTemplateName", "Designation", "Nickname", "Description", "LongDescription",
      "Cost", "BodyKey", "TemplateKey", "BaseColor", "StripeColor", "Sockets"
    ])?;

    let associated_template_name = associated_template_name.map(|element| element.children.deserialize::<String>()).transpose()?;
    let designation = designation.ok_or(xml::Error::missing_element("Designation"))?.children.deserialize::<String>()?;
    let nickname = nickname.ok_or(xml::Error::missing_element("Nickname"))?.children.deserialize::<String>()?;
    let description = description.ok_or(xml::Error::missing_element("Description"))?.children.deserialize::<String>()?;
    let long_description = long_description.ok_or(xml::Error::missing_element("LongDescription"))?.children.deserialize::<String>()?;
    let cost = cost.ok_or(xml::Error::missing_element("Cost"))?.children.deserialize::<usize>()?;
    let body_key = body_key.ok_or(xml::Error::missing_element("BodyKey"))?.children.deserialize::<MissileBodyKey>()?;
    let template_key = template_key.ok_or(xml::Error::missing_element("TemplateKey"))?.children.deserialize::<Uuid>()?;
    let base_color = base_color.ok_or(xml::Error::missing_element("BaseColor"))?.children.deserialize::<Color>()?;
    let stripe_color = stripe_color.ok_or(xml::Error::missing_element("StripeColor"))?.children.deserialize::<Color>()?;
    let sockets = sockets.ok_or(xml::Error::missing_element("Sockets"))?.children.deserialize::<Vec<MissileSocket>>()?;

    Ok(MissileTemplate {
      associated_template_name,
      designation,
      nickname,
      description,
      long_description,
      cost,
      body_key,
      template_key,
      base_color,
      stripe_color,
      sockets
    })
  }
}

impl SerializeElement for MissileTemplate {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let name = self.associated_template_name
      .map(String::serialize_nodes).transpose()?
      .map(|nodes| Element::new("AssociatedTemplateName", nodes));
    let nodes = [
      Element::new("Designation", self.designation.serialize_nodes()?),
      Element::new("Nickname", self.nickname.serialize_nodes()?),
      Element::new("Description", self.description.serialize_nodes()?),
      Element::new("LongDescription", self.long_description.serialize_nodes()?),
      Element::new("Cost", self.cost.serialize_nodes()?),
      Element::new("BodyKey", self.body_key.serialize_nodes()?),
      Element::new("TemplateKey", self.template_key.serialize_nodes()?),
      Element::new("BaseColor", self.base_color.serialize_nodes()?),
      Element::new("StripeColor", self.stripe_color.serialize_nodes()?),
      Element::new("Sockets", self.sockets.serialize_nodes()?)
    ];

    Ok(Element::new("MissileTemplate", Nodes::from_iter(chain_iter!(name, nodes))))
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileSocket {
  pub size: zsize,
  pub installed_component: Option<MissileComponent>
}

impl DeserializeElement for MissileSocket {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("MissileSocket")?;
    let [size, installed_component] = element.children.find_elements(["Size", "InstalledComponent"])?;
    let size = size.ok_or(xml::Error::missing_element("Size"))?.children.deserialize::<zsize>()?;
    let installed_component = installed_component.map(MissileComponent::deserialize_element).transpose()?;
    Ok(MissileSocket { size, installed_component })
  }
}

impl SerializeElement for MissileSocket {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let size = Element::new("Size", self.size.serialize_nodes()?);
    let installed_component = self.installed_component.map(MissileComponent::serialize_element).transpose()?;

    let iter = std::iter::once(size).chain(installed_component);
    Ok(Element::new("MissileSocket", Nodes::from_iter(iter)))
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileComponent {
  /// Only `None` when this component is an engine.
  pub component_key: Option<MissileComponentKey>,
  pub settings: Option<MissileComponentSettings>
}

impl DeserializeElement for MissileComponent {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("InstalledComponent")?;
    let [xsi_type] = element.attributes.find_attributes(["xsi:type"])?;

    let [
      component_key, mode, reject_unvalidated, target_type, detect_pd_targets, role, hot_launch,
      self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control, balance_values
    ] = element.children.find_elements([
      "ComponentKey", "Mode", "RejectUnvalidated", "TargetType", "DetectPDTargets", "Role", "HotLaunch",
      "SelfDestructOnLost", "Maneuvers", "DefensiveDoctrine", "ApproachAngleControl", "BalanceValues"
    ])?;

    let mode = mode.ok_or(xml::Error::missing_element("Mode"))
      .and_then(|element| element.children.deserialize::<SeekerMode>());
    let reject_unvalidated = reject_unvalidated
      .map(|element| element.children.deserialize::<bool>())
      .transpose()?.unwrap_or(false);
    let target_type = target_type.ok_or(xml::Error::missing_element("TargetType"))
      .and_then(|element| element.children.deserialize::<AntiRadiationTargetType>());
    let detect_pd_targets = detect_pd_targets.ok_or(xml::Error::missing_element("DetectPDTargets"))
      .and_then(|element| element.children.deserialize::<bool>());
    let role = role.ok_or(xml::Error::missing_element("Role"))
      .and_then(|element| element.children.deserialize::<MissileRole>());
    let hot_launch = hot_launch.ok_or(xml::Error::missing_element("HotLaunch"))
      .and_then(|element| element.children.deserialize::<bool>());
    let self_destruct_on_lost = self_destruct_on_lost.ok_or(xml::Error::missing_element("SelfDestructOnLost"))
      .and_then(|element| element.children.deserialize::<bool>());
    let maneuvers = maneuvers.ok_or(xml::Error::missing_element("Maneuvers"))
      .and_then(|element| element.children.deserialize::<Maneuvers>());
    let defensive_doctrine = defensive_doctrine.ok_or(xml::Error::missing_element("DefensiveDoctrine"))
      .map_err(FormatError::from).and_then(DefensiveDoctrine::deserialize_element);
    let approach_angle_control = approach_angle_control.ok_or(xml::Error::missing_element("ApproachAngleControl"))
      .and_then(|element| element.children.deserialize::<bool>());
    let balance_values = balance_values.ok_or(xml::Error::missing_element("BalanceValues"));

    let component_key = component_key.filter(|element| !element.children.is_empty())
      .map(|element| element.children.deserialize::<MissileComponentKey>()).transpose()?;
    let settings = xsi_type.as_deref().map(|xsi_type| match xsi_type {
      "ActiveSeekerSettings" => Ok(MissileComponentSettings::ActiveSeekerSettings {
        mode: mode?,
        reject_unvalidated,
        detect_pd_targets: detect_pd_targets?
      }),
      "CommandSeekerSettings" => Ok(MissileComponentSettings::CommandSeekerSettings {
        mode: mode?
      }),
      "DirectGuidanceSettings" => Ok(MissileComponentSettings::DirectGuidanceSettings {
        hot_launch: hot_launch?,
        self_destruct_on_lost: self_destruct_on_lost?,
        maneuvers: maneuvers?,
        defensive_doctrine: if role? == MissileRole::Defensive { Some(defensive_doctrine?) } else { None },
        approach_angle_control: approach_angle_control?
      }),
      "CruiseGuidanceSettings" => Ok(MissileComponentSettings::CruiseGuidanceSettings {
        hot_launch: hot_launch?,
        self_destruct_on_lost: self_destruct_on_lost?,
        maneuvers: maneuvers?,
        defensive_doctrine: if role? == MissileRole::Defensive { Some(defensive_doctrine?) } else { None }
      }),
      "MissileEngineSettings" => Ok(MissileComponentSettings::MissileEngineSettings {
        balance_values: balance_values.and_then(|element| {
          let [a, b, c] = element.children.find_elements(["A", "B", "C"])?;
          let a = a.ok_or(xml::Error::missing_element("A"))?.children.deserialize::<f32>()?;
          let b = b.ok_or(xml::Error::missing_element("B"))?.children.deserialize::<f32>()?;
          let c = c.ok_or(xml::Error::missing_element("C"))?.children.deserialize::<f32>()?;
          Ok(EngineSettings::from_array([a, b, c]))
        })?
      }),
      "PassiveARHSeekerSettings" => Ok(MissileComponentSettings::PassiveARHSeekerSettings {
        mode: mode?,
        reject_unvalidated,
        home_on_jam: target_type? == AntiRadiationTargetType::JammingOnly
      }),
      "PassiveSeekerSettings" => Ok(MissileComponentSettings::PassiveSeekerSettings {
        mode: mode?,
        reject_unvalidated,
        detect_pd_targets: detect_pd_targets?
      }),
      _ => Err(FormatError::UnknownMissileSettingsType(xsi_type.into()))
    }).transpose()?;

    Ok(MissileComponent {
      component_key,
      settings
    })
  }
}

impl SerializeElement for MissileComponent {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let component_key = self.component_key
      .map(MissileComponentKey::serialize_nodes).transpose()?
      .map(|nodes| Element::new("ComponentKey", nodes));
    let (attributes, nodes) = match self.settings.map(serialize_settings).transpose()? {
      Some((xsi_type, elements)) => (xml::attributes!("xsi:type" = xsi_type), Nodes::from_iter(chain_iter!(component_key, elements))),
      None => (xml::Attributes::new(), Nodes::from_iter(component_key))
    };

    Ok(Element::with_attributes("InstalledComponent", attributes, nodes))
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum MissileComponentSettings {
  ActiveSeekerSettings {
    // Mode, RejectUnvalidated, DetectPDTargets
    mode: SeekerMode,
    reject_unvalidated: bool,
    detect_pd_targets: bool
  },
  CommandSeekerSettings {
    // Mode
    mode: SeekerMode
  },
  DirectGuidanceSettings {
    // Role, HotLaunch, SelfDestructOnLost, Maneuvers, DefensiveDoctrine, ApproachAngleControl
    hot_launch: bool,
    self_destruct_on_lost: bool,
    maneuvers: Maneuvers,
    defensive_doctrine: Option<DefensiveDoctrine>,
    approach_angle_control: bool
  },
  CruiseGuidanceSettings {
    // Role, HotLaunch, SelfDestructOnLost, Maneuvers, DefensiveDoctrine
    hot_launch: bool,
    self_destruct_on_lost: bool,
    maneuvers: Maneuvers,
    defensive_doctrine: Option<DefensiveDoctrine>
  },
  MissileEngineSettings {
    // BalanceValues
    balance_values: EngineSettings
  },
  PassiveARHSeekerSettings {
    // Mode, RejectUnvalidated, TargetType
    mode: SeekerMode,
    reject_unvalidated: bool,
    home_on_jam: bool
  },
  PassiveSeekerSettings {
    // Mode, RejectUnvalidated, DetectPDTargets
    mode: SeekerMode,
    reject_unvalidated: bool,
    detect_pd_targets: bool
  }
}

fn serialize_settings(settings: MissileComponentSettings) -> Result<(&'static str, Vec<Element>), Infallible> {
  Ok(match settings {
    MissileComponentSettings::ActiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets } => {
      let mode = Element::new("Mode", mode.serialize_nodes()?);
      let reject_unvalidated = Element::new("RejectUnvalidated", reject_unvalidated.serialize_nodes()?);
      let detect_pd_targets = Element::new("DetectPDTargets", detect_pd_targets.serialize_nodes()?);
      ("ActiveSeekerSettings", vec![mode, reject_unvalidated, detect_pd_targets])
    },
    MissileComponentSettings::CommandSeekerSettings { mode } => {
      let mode = Element::new("Mode", mode.serialize_nodes()?);
      ("CommandSeekerSettings", vec![mode])
    },
    MissileComponentSettings::DirectGuidanceSettings {
      hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control
    } => {
      let role = if defensive_doctrine.is_some() { MissileRole::Defensive } else { MissileRole::Offensive };
      let role = Element::new("Role", role.serialize_nodes()?);
      let hot_launch = Element::new("HotLaunch", hot_launch.serialize_nodes()?);
      let self_destruct_on_lost = Element::new("SelfDestructOnLost", self_destruct_on_lost.serialize_nodes()?);
      let maneuvers = Element::new("Maneuvers", maneuvers.serialize_nodes()?);
      let defensive_doctrine = defensive_doctrine.unwrap_or_default().serialize_element()?;
      let approach_angle_control = Element::new("ApproachAngleControl", approach_angle_control.serialize_nodes()?);
      ("DirectGuidanceSettings", vec![role, hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine, approach_angle_control])
    },
    MissileComponentSettings::CruiseGuidanceSettings {
      hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine
    } => {
      let role = if defensive_doctrine.is_some() { MissileRole::Defensive } else { MissileRole::Offensive };
      let role = Element::new("Role", role.serialize_nodes()?);
      let hot_launch = Element::new("HotLaunch", hot_launch.serialize_nodes()?);
      let self_destruct_on_lost = Element::new("SelfDestructOnLost", self_destruct_on_lost.serialize_nodes()?);
      let maneuvers = Element::new("Maneuvers", maneuvers.serialize_nodes()?);
      let defensive_doctrine = defensive_doctrine.unwrap_or_default().serialize_element()?;
      ("CruiseGuidanceSettings", vec![role, hot_launch, self_destruct_on_lost, maneuvers, defensive_doctrine])
    },
    MissileComponentSettings::MissileEngineSettings { balance_values } => {
      let [a, b, c] = balance_values.into_array();
      let balance_values = Element::new("BalanceValues", Nodes::from_iter([
        Element::new("A", a.serialize_nodes()?),
        Element::new("B", b.serialize_nodes()?),
        Element::new("C", c.serialize_nodes()?),
      ]));
      ("MissileEngineSettings", vec![balance_values])
    },
    MissileComponentSettings::PassiveARHSeekerSettings { mode, reject_unvalidated, home_on_jam } => {
      let mode = Element::new("Mode", mode.serialize_nodes()?);
      let reject_unvalidated = Element::new("RejectUnvalidated", reject_unvalidated.serialize_nodes()?);
      let target_type = if home_on_jam { AntiRadiationTargetType::JammingOnly } else { AntiRadiationTargetType::All };
      let target_type = Element::new("TargetType", target_type.serialize_nodes()?);
      ("PassiveARHSeekerSettings", vec![mode, reject_unvalidated, target_type])
    },
    MissileComponentSettings::PassiveSeekerSettings { mode, reject_unvalidated, detect_pd_targets } => {
      let mode = Element::new("Mode", mode.serialize_nodes()?);
      let reject_unvalidated = Element::new("RejectUnvalidated", reject_unvalidated.serialize_nodes()?);
      let detect_pd_targets = Element::new("DetectPDTargets", detect_pd_targets.serialize_nodes()?);
      ("PassiveSeekerSettings", vec![mode, reject_unvalidated, detect_pd_targets])
    }
  })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct DefensiveDoctrine {
  pub target_size_mask: MissileSizeMask,
  pub target_type: DefensiveTargetType,
  pub target_size_ordering: Ordering,
  pub salvo_size: usize,
  pub farthest_first: bool
}

// Defensive doctrine settings are still serialized for offensive missiles,
// but they will usually have these settings here.
impl Default for DefensiveDoctrine {
  fn default() -> Self {
    DefensiveDoctrine {
      target_size_mask: MissileSizeMask::default(),
      target_type: DefensiveTargetType::All,
      target_size_ordering: Ordering::Descending,
      salvo_size: 0,
      farthest_first: false
    }
  }
}

impl DeserializeElement for DefensiveDoctrine {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("DefensiveDoctrine")?;
    let [target_size_mask, target_type, target_size_ordering, salvo_size, farthest_first] = element.children
      .find_elements(["TargetSizeMask", "TargetType", "TargetSizeOrdering", "SalvoSize", "FarthestFirst"])?;

    let target_size_mask = target_size_mask.ok_or(xml::Error::missing_element("TargetSizeMask"))?.children.deserialize::<MissileSizeMask>()?;
    let target_type = target_type.map(|element| element.children.deserialize::<DefensiveTargetType>()).transpose()?.unwrap_or_default();
    let target_size_ordering = target_size_ordering.ok_or(xml::Error::missing_element("TargetSizeOrdering"))?.children.deserialize::<Ordering>()?;
    let salvo_size = salvo_size.ok_or(xml::Error::missing_element("SalvoSize"))?.children.deserialize::<usize>()?;
    let farthest_first = farthest_first.ok_or(xml::Error::missing_element("FarthestFirst"))?.children.deserialize::<bool>()?;

    Ok(DefensiveDoctrine {
      target_size_mask,
      target_type,
      target_size_ordering,
      salvo_size,
      farthest_first
    })
  }
}

impl SerializeElement for DefensiveDoctrine {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let target_size_mask = Element::new("TargetSizeMask", self.target_size_mask.serialize_nodes()?);
    let target_type = Element::new("TargetType", self.target_type.serialize_nodes()?);
    let target_size_ordering = Element::new("TargetSizeOrdering", self.target_size_ordering.serialize_nodes()?);
    let salvo_size = Element::new("SalvoSize", self.salvo_size.serialize_nodes()?);
    let farthest_first = Element::new("FarthestFirst", self.farthest_first.serialize_nodes()?);

    let iter = [target_size_mask, target_type, target_size_ordering, salvo_size, farthest_first];
    Ok(Element::new("DefensiveDoctrine", Nodes::from_iter(iter)))
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum DefensiveTargetType {
  #[default] All, Conventional, Hybrid
}

impl DefensiveTargetType {
  pub const fn to_str(self) -> &'static str {
    match self {
      DefensiveTargetType::All => "All",
      DefensiveTargetType::Conventional => "Conventional",
      DefensiveTargetType::Hybrid => "Hybrid"
    }
  }
}

impl FromStr for DefensiveTargetType {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "All" => Ok(DefensiveTargetType::All),
      "Conventional" => Ok(DefensiveTargetType::Conventional),
      "Hybrid" => Ok(DefensiveTargetType::Hybrid),
      _ => Err(crate::data::InvalidKey::DefensiveTargetType)
    }
  }
}

impl fmt::Display for DefensiveTargetType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

xml::impl_deserialize_nodes_parse!(DefensiveTargetType);
xml::impl_serialize_nodes_display!(DefensiveTargetType);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MissileComponentKey {
  // Seeker Components
  CommandReceiver,
  FixedActiveRadarSeeker,
  SteerableActiveRadarSeeker,
  SteerableExtendedActiveRadarSeeker,
  FixedSemiActiveRadarSeeker,
  FixedAntiRadiationSeeker,
  ElectroOpticalSeeker,
  WakeHomingSeeker,
  // Auxilliary Components
  ColdGasBottle,
  DecoyLauncher,
  ClusterDecoyLauncher,
  FastStartupModule,
  HardenedSkin,
  RadarAbsorbentCoating,
  SelfScreeningJammer,
  BoostedSelfScreeningJammer,
  // Avionics Components
  DirectGuidance,
  CruiseGuidance,
  // Payload Components
  HEImpact,
  HEKineticPenetrator,
  BlastFragmentation,
  BlastFragmentationEL
}

impl MissileComponentKey {
  pub const fn save_key(self) -> &'static str {
    match self {
      Self::CommandReceiver => SeekerKey::Command.save_key(),
      Self::FixedActiveRadarSeeker => SeekerKey::FixedActiveRadar.save_key(),
      Self::SteerableActiveRadarSeeker => SeekerKey::SteerableActiveRadar.save_key(),
      Self::SteerableExtendedActiveRadarSeeker => SeekerKey::SteerableExtendedActiveRadar.save_key(),
      Self::FixedSemiActiveRadarSeeker => SeekerKey::FixedSemiActiveRadar.save_key(),
      Self::FixedAntiRadiationSeeker => SeekerKey::FixedAntiRadiation.save_key(),
      Self::ElectroOpticalSeeker => SeekerKey::ElectroOptical.save_key(),
      Self::WakeHomingSeeker => SeekerKey::WakeHoming.save_key(),

      Self::ColdGasBottle => AuxiliaryKey::ColdGasBottle.save_key(),
      Self::DecoyLauncher => AuxiliaryKey::DecoyLauncher.save_key(),
      Self::ClusterDecoyLauncher => AuxiliaryKey::ClusterDecoyLauncher.save_key(),
      Self::FastStartupModule => AuxiliaryKey::FastStartupModule.save_key(),
      Self::HardenedSkin => AuxiliaryKey::HardenedSkin.save_key(),
      Self::RadarAbsorbentCoating => AuxiliaryKey::RadarAbsorbentCoating.save_key(),
      Self::SelfScreeningJammer => AuxiliaryKey::SelfScreeningJammer.save_key(),
      Self::BoostedSelfScreeningJammer => AuxiliaryKey::BoostedSelfScreeningJammer.save_key(),

      Self::DirectGuidance => AvionicsKey::DirectGuidance.save_key(),
      Self::CruiseGuidance => AvionicsKey::CruiseGuidance.save_key(),

      Self::HEImpact => WarheadKey::HEImpact.save_key(),
      Self::HEKineticPenetrator => WarheadKey::HEKineticPenetrator.save_key(),
      Self::BlastFragmentation => WarheadKey::BlastFragmentation.save_key(),
      Self::BlastFragmentationEL => WarheadKey::BlastFragmentationEL.save_key()
    }
  }

  pub const fn to_seeker_key(self) -> Option<SeekerKey> {
    match self {
      Self::CommandReceiver => Some(SeekerKey::Command),
      Self::FixedActiveRadarSeeker => Some(SeekerKey::FixedActiveRadar),
      Self::SteerableActiveRadarSeeker => Some(SeekerKey::SteerableActiveRadar),
      Self::SteerableExtendedActiveRadarSeeker => Some(SeekerKey::SteerableExtendedActiveRadar),
      Self::FixedSemiActiveRadarSeeker => Some(SeekerKey::FixedSemiActiveRadar),
      Self::FixedAntiRadiationSeeker => Some(SeekerKey::FixedAntiRadiation),
      Self::ElectroOpticalSeeker => Some(SeekerKey::ElectroOptical),
      Self::WakeHomingSeeker => Some(SeekerKey::WakeHoming),
      _ => None
    }
  }

  pub const fn to_auxiliary_key(self) -> Option<AuxiliaryKey> {
    match self {
      Self::ColdGasBottle => Some(AuxiliaryKey::ColdGasBottle),
      Self::DecoyLauncher => Some(AuxiliaryKey::DecoyLauncher),
      Self::ClusterDecoyLauncher => Some(AuxiliaryKey::ClusterDecoyLauncher),
      Self::FastStartupModule => Some(AuxiliaryKey::FastStartupModule),
      Self::HardenedSkin => Some(AuxiliaryKey::HardenedSkin),
      Self::RadarAbsorbentCoating => Some(AuxiliaryKey::RadarAbsorbentCoating),
      Self::SelfScreeningJammer => Some(AuxiliaryKey::SelfScreeningJammer),
      Self::BoostedSelfScreeningJammer => Some(AuxiliaryKey::BoostedSelfScreeningJammer),
      _ => None
    }
  }

  pub const fn to_avionics_key(self) -> Option<AvionicsKey> {
    match self {
      Self::DirectGuidance => Some(AvionicsKey::DirectGuidance),
      Self::CruiseGuidance => Some(AvionicsKey::CruiseGuidance),
      _ => None
    }
  }

  pub const fn to_warhead_key(self) -> Option<WarheadKey> {
    match self {
      Self::HEImpact => Some(WarheadKey::HEImpact),
      Self::HEKineticPenetrator => Some(WarheadKey::HEKineticPenetrator),
      Self::BlastFragmentation => Some(WarheadKey::BlastFragmentation),
      Self::BlastFragmentationEL => Some(WarheadKey::BlastFragmentationEL),
      _ => None
    }
  }
}

impl FromStr for MissileComponentKey {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Stock/Command Receiver" => Ok(Self::CommandReceiver),
      "Stock/Fixed Active Radar Seeker" => Ok(Self::FixedActiveRadarSeeker),
      "Stock/Steerable Active Radar Seeker" => Ok(Self::SteerableActiveRadarSeeker),
      "Stock/Steerable Extended Active Radar Seeker" => Ok(Self::SteerableExtendedActiveRadarSeeker),
      "Stock/Fixed Semi-Active Radar Seeker" => Ok(Self::FixedSemiActiveRadarSeeker),
      "Stock/Fixed Anti-Radiation Seeker" => Ok(Self::FixedAntiRadiationSeeker),
      "Stock/Electro-Optical Seeker" => Ok(Self::ElectroOpticalSeeker),
      "Stock/Wake-Homing Seeker" => Ok(Self::WakeHomingSeeker),
      "Stock/Cold Gas Bottle" => Ok(Self::ColdGasBottle),
      "Stock/Decoy Launcher" => Ok(Self::DecoyLauncher),
      "Stock/Cluster Decoy Launcher" => Ok(Self::ClusterDecoyLauncher),
      "Stock/Fast Startup Module" => Ok(Self::FastStartupModule),
      "Stock/Hardened Skin" => Ok(Self::HardenedSkin),
      "Stock/Radar Absorbent Coating" => Ok(Self::RadarAbsorbentCoating),
      "Stock/Self-Screening Jammer" => Ok(Self::SelfScreeningJammer),
      "Stock/Boosted Self-Screening Jammer" => Ok(Self::BoostedSelfScreeningJammer),
      "Stock/Direct Guidance" => Ok(Self::DirectGuidance),
      "Stock/Cruise Guidance" => Ok(Self::CruiseGuidance),
      "Stock/HE Impact" => Ok(Self::HEImpact),
      "Stock/HE Kinetic Penetrator" => Ok(Self::HEKineticPenetrator),
      "Stock/Blast Fragmentation" => Ok(Self::BlastFragmentation),
      "Stock/Blast Fragmentation EL" => Ok(Self::BlastFragmentationEL),
      _ => Err(crate::data::InvalidKey::MissileComponent)
    }
  }
}

impl fmt::Display for MissileComponentKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}

xml::impl_deserialize_nodes_parse!(MissileComponentKey);
xml::impl_serialize_nodes_display!(MissileComponentKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileSizeMask {
  pub size1: bool,
  pub size2: bool,
  pub size3: bool
}

impl MissileSizeMask {
  pub const fn from_u8(b: u8) -> Self {
    MissileSizeMask {
      size1: (b >> 3) & 0b1 == 0b1,
      size2: (b >> 2) & 0b1 == 0b1,
      size3: (b >> 1) & 0b1 == 0b1
    }
  }

  pub const fn to_u8(self) -> u8 {
    ((self.size1 as u8) << 3) |
    ((self.size2 as u8) << 2) |
    ((self.size3 as u8) << 1)
  }
}

impl Default for MissileSizeMask {
  fn default() -> Self {
    MissileSizeMask {
      size1: false,
      size2: true,
      size3: true
    }
  }
}

impl Index<MissileSize> for MissileSizeMask {
  type Output = bool;

  fn index(&self, missile_size: MissileSize) -> &Self::Output {
    match missile_size {
      MissileSize::Size1 => &self.size1,
      MissileSize::Size2 => &self.size2,
      MissileSize::Size3 => &self.size3
    }
  }
}

impl FromStr for MissileSizeMask {
  type Err = <u8 as FromStr>::Err;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let b = s.parse::<u8>()?;
    Ok(MissileSizeMask::from_u8(b))
  }
}

impl fmt::Display for MissileSizeMask {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.to_u8(), f)
  }
}

xml::impl_deserialize_nodes_parse!(MissileSizeMask);
xml::impl_serialize_nodes_display!(MissileSizeMask);

#[derive(Debug, Error, Clone, Copy)]
#[error("invalid missile size mask: unexpected char {0:?}")]
pub struct InvalidMissileSizeMask(char);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MissileRole {
  #[default] Offensive, Defensive
}

impl MissileRole {
  pub const fn to_str(self) -> &'static str {
    match self {
      MissileRole::Offensive => "Offensive",
      MissileRole::Defensive => "Defensive"
    }
  }
}

impl FromStr for MissileRole {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Offensive" => Ok(MissileRole::Offensive),
      "Defensive" => Ok(MissileRole::Defensive),
      _ => Err(crate::data::InvalidKey::MissileRole)
    }
  }
}

impl fmt::Display for MissileRole {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

xml::impl_deserialize_nodes_parse!(MissileRole);
xml::impl_serialize_nodes_display!(MissileRole);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum AntiRadiationTargetType {
  #[default] All, JammingOnly
}

impl AntiRadiationTargetType {
  pub const fn to_str(self) -> &'static str {
    match self {
      AntiRadiationTargetType::All => "All",
      AntiRadiationTargetType::JammingOnly => "JammingOnly"
    }
  }
}

impl FromStr for AntiRadiationTargetType {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "All" => Ok(AntiRadiationTargetType::All),
      "JammingOnly" => Ok(AntiRadiationTargetType::JammingOnly),
      _ => Err(crate::data::InvalidKey::AntiRadiationTargetType)
    }
  }
}

impl fmt::Display for AntiRadiationTargetType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

xml::impl_deserialize_nodes_parse!(AntiRadiationTargetType);
xml::impl_serialize_nodes_display!(AntiRadiationTargetType);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Ordering {
  Ascending, #[default] Descending, Equal
}

impl Ordering {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::Ascending => "Ascending",
      Self::Descending => "Descending",
      Self::Equal => "Equal"
    }
  }
}

impl FromStr for Ordering {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Ascending" => Ok(Self::Ascending),
      "Descending" => Ok(Self::Descending),
      "Equal" => Ok(Self::Equal),
      _ => Err(crate::data::InvalidKey::Ordering)
    }
  }
}

impl fmt::Display for Ordering {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

xml::impl_deserialize_nodes_parse!(Ordering);
xml::impl_serialize_nodes_display!(Ordering);



#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32
}

impl Color {
  pub const fn splat(v: f32, a: f32) -> Self {
    Color { r: v, g: v, b: v, a }
  }
}

impl DeserializeNodes for Color {
  type Error = FormatError;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    let [r, g, b, a] = nodes.find_elements(["r", "g", "b", "a"])?;

    let r = r.ok_or(xml::Error::missing_element("r"))?.children.deserialize::<f32>()?;
    let g = g.ok_or(xml::Error::missing_element("g"))?.children.deserialize::<f32>()?;
    let b = b.ok_or(xml::Error::missing_element("b"))?.children.deserialize::<f32>()?;
    let a = a.ok_or(xml::Error::missing_element("a"))?.children.deserialize::<f32>()?;

    Ok(Color { r, g, b, a })
  }
}

impl SerializeNodes for Color {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    let r = Element::new("r", self.r.serialize_nodes()?);
    let g = Element::new("g", self.g.serialize_nodes()?);
    let b = Element::new("b", self.b.serialize_nodes()?);
    let a = Element::new("a", self.a.serialize_nodes()?);

    Ok(Nodes::from_iter([r, g, b, a]))
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Vector3<T> {
  pub x: T,
  pub y: T,
  pub z: T
}

impl<T> Vector3<T> {
  pub const fn splat(v: T) -> Self where T: Copy {
    Vector3 { x: v, y: v, z: v }
  }
}

impl<T> DeserializeNodes for Vector3<T> where T: DeserializeNodes {
  type Error = xml::DeserializeErrorWrapper<T::Error>;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    let [x, y, z] = nodes.find_elements(["x", "y", "z"])?;

    let x = x.ok_or(xml::Error::missing_element("x"))?
      .children.deserialize::<T>().map_err(xml::DeserializeErrorWrapper::Inner)?;
    let y = y.ok_or(xml::Error::missing_element("y"))?
      .children.deserialize::<T>().map_err(xml::DeserializeErrorWrapper::Inner)?;
    let z = z.ok_or(xml::Error::missing_element("z"))?
      .children.deserialize::<T>().map_err(xml::DeserializeErrorWrapper::Inner)?;

    Ok(Vector3 { x, y, z })
  }
}

impl<T> SerializeNodes for Vector3<T> where T: SerializeNodes {
  type Error = T::Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    let x = Element::new("x", self.x.serialize_nodes()?);
    let y = Element::new("y", self.y.serialize_nodes()?);
    let z = Element::new("z", self.z.serialize_nodes()?);

    Ok(Nodes::from_iter([x, y, z]))
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Vector2<T> {
  pub x: T,
  pub y: T
}

impl<T> Vector2<T> {
  pub const fn splat(v: T) -> Self where T: Copy {
    Vector2 { x: v, y: v }
  }
}

impl<T> DeserializeNodes for Vector2<T> where T: DeserializeNodes {
  type Error = xml::DeserializeErrorWrapper<T::Error>;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    let [x, y] = nodes.find_elements(["x", "y"])?;

    let x = x.ok_or(xml::Error::missing_element("x"))?
      .children.deserialize::<T>().map_err(xml::DeserializeErrorWrapper::Inner)?;
    let y = y.ok_or(xml::Error::missing_element("y"))?
      .children.deserialize::<T>().map_err(xml::DeserializeErrorWrapper::Inner)?;

    Ok(Vector2 { x, y })
  }
}

impl<T> SerializeNodes for Vector2<T> where T: SerializeNodes {
  type Error = T::Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    let x = Element::new("x", self.x.serialize_nodes()?);
    let y = Element::new("y", self.y.serialize_nodes()?);

    Ok(Nodes::from_iter([x, y]))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Costs {
  pub hulls: usize,
  pub components: usize,
  pub ammunition: usize,
  pub missiles: usize
}

impl Costs {
  pub const fn total(self) -> usize {
    self.hulls + self.components + self.ammunition + self.missiles
  }
}

impl Add for Costs {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Costs {
      hulls: self.hulls + rhs.hulls,
      components: self.components + rhs.components,
      ammunition: self.ammunition + rhs.ammunition,
      missiles: self.missiles + rhs.missiles
    }
  }
}

impl AddAssign for Costs {
  fn add_assign(&mut self, rhs: Self) {
    self.hulls += rhs.hulls;
    self.components += rhs.components;
    self.ammunition += rhs.ammunition;
    self.missiles += rhs.missiles;
  }
}
