use crate::data::{Faction, MissileSize};
use crate::data::hulls::HullKey;
use crate::data::missiles::Maneuvers;
use crate::data::missiles::bodies::MissileBodyKey;
use crate::data::missiles::seekers::SeekerMode;
use crate::data::missiles::engines::EngineSettings;
use crate::data::components::ComponentKey;

use bytemuck::Contiguous;
use xml::{DeserializeElement, DeserializeNodes, SerializeElement, SerializeNodes, Element, Nodes, Attributes};

pub use xml::uuid::Uuid;
pub use xml::{read_nodes, write_nodes};

use std::convert::Infallible;
use std::fmt;
use std::num::NonZeroUsize as zsize;
use std::ops::Index;
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
pub struct Fleet {
  pub name: String,
  pub total_points: usize,
  pub faction_key: Faction,
  pub description: Option<String>,
  pub ships: Vec<Ship>,
  pub missile_types: Vec<MissileTemplate>
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
    let missile_types = missile_types.ok_or(xml::Error::missing_element("MissileTypes"))?.children.deserialize::<Vec<MissileTemplate>>()?;

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
    let missile_types = missile_types.ok_or(xml::Error::missing_element("TemplateMissileTypes"))?.children.deserialize::<Vec<MissileTemplate>>()?;

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
pub struct HullSocket {
  pub key: Box<str>,
  pub component_name: ComponentKey,
  pub component_data: Option<ComponentData>
}

impl DeserializeElement for HullSocket {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    let [key, component_name, component_data] = element.children
      .find_elements(["Key", "ComponentName", "ComponentData"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<String>()?.into_boxed_str();
    let component_name = component_name.ok_or(xml::Error::missing_element("ComponentName"))?.children.deserialize::<ComponentKey>()?;
    let component_data = component_data.map(Element::deserialize::<ComponentData>).transpose()?;

    Ok(HullSocket { key, component_name, component_data })
  }
}

impl SerializeElement for HullSocket {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let key = Element::new("Key", Nodes::new_text(self.key));
    let component_name = Element::new("ComponentName", Nodes::new_text(self.component_name.save_key()));
    let component_data = self.component_data.map(ComponentData::serialize_element).transpose()?;
    let nodes = Nodes::from_iter(chain_iter!([key, component_name], component_data));
    Ok(Element::new("HullSocket", nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
  }
}

impl ComponentData {
  pub const fn get_load(&self) -> &Vec<MagazineSaveData> {
    match self {
      ComponentData::BulkMagazineData { load } => load,
      ComponentData::CellLauncherData { missile_load } => missile_load,
      ComponentData::ResizableCellLauncherData { missile_load, .. } => missile_load
    }
  }

  pub fn get_load_mut(&mut self) -> &mut Vec<MagazineSaveData> {
    match self {
      ComponentData::BulkMagazineData { load } => load,
      ComponentData::CellLauncherData { missile_load } => missile_load,
      ComponentData::ResizableCellLauncherData { missile_load, .. } => missile_load
    }
  }

  pub fn into_load(self) -> Vec<MagazineSaveData> {
    match self {
      ComponentData::BulkMagazineData { load } => load,
      ComponentData::CellLauncherData { missile_load } => missile_load,
      ComponentData::ResizableCellLauncherData { missile_load, .. } => missile_load
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
      })
    };

    Ok(Element::with_attributes("ComponentData", xml::attributes!("xsi:type" = xsi_type), nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagazineSaveData {
  pub magazine_key: Box<str>,
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
      .children.deserialize::<String>()?.into_boxed_str();
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
pub struct WeaponGroup {
  pub name: String,
  pub members: Vec<Box<str>>
}

impl DeserializeElement for WeaponGroup {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("WepGroup")?;
    let [name] = element.attributes.find_attributes(["Name"])?;
    let name = name.ok_or(xml::Error::missing_attribute("Name"))?;

    let member_keys = element.children.try_into_one_element()?;
    member_keys.expect_named("MemberKeys")?;

    let members = member_keys.children.deserialize_named_elements::<Box<str>, Vec<Box<str>>>("string")?;

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
pub struct InitialFormation {
  guide_key: Uuid,
  relative_position: Vector3<f32>
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
pub enum HullConfig {
  RandomHullConfiguration {
    primary_structure: [SegmentConfiguration; 3],
    secondary_structure: SecondaryStructureConfig,
    hull_tint: Color,
    texture_variation: Vector3<f32>
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
    let reject_unvalidated = reject_unvalidated.ok_or(xml::Error::missing_element("RejectUnvalidated"))
      .and_then(|element| element.children.deserialize::<bool>());
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
        reject_unvalidated: reject_unvalidated?,
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
        reject_unvalidated: reject_unvalidated?,
        home_on_jam: target_type? == AntiRadiationTargetType::JammingOnly
      }),
      "PassiveSeekerSettings" => Ok(MissileComponentSettings::PassiveSeekerSettings {
        mode: mode?,
        reject_unvalidated: reject_unvalidated?,
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
      Self::CommandReceiver => "Stock/Command Receiver",
      Self::FixedActiveRadarSeeker => "Stock/Fixed Active Radar Seeker",
      Self::SteerableActiveRadarSeeker => "Stock/Steerable Active Radar Seeker",
      Self::SteerableExtendedActiveRadarSeeker => "Stock/Steerable Extended Active Radar Seeker",
      Self::FixedSemiActiveRadarSeeker => "Stock/Fixed Semi-Active Radar Seeker",
      Self::FixedAntiRadiationSeeker => "Stock/Fixed Anti-Radiation Seeker",
      Self::ElectroOpticalSeeker => "Stock/Electro-Optical Seeker",
      Self::WakeHomingSeeker => "Stock/Wake-Homing Seeker",
      Self::ColdGasBottle => "Stock/Cold Gas Bottle",
      Self::DecoyLauncher => "Stock/Decoy Launcher",
      Self::ClusterDecoyLauncher => "Stock/Cluster Decoy Launcher",
      Self::FastStartupModule => "Stock/Fast Startup Module",
      Self::HardenedSkin => "Stock/Hardened Skin",
      Self::RadarAbsorbentCoating => "Stock/Radar Absorbent Coating",
      Self::SelfScreeningJammer => "Stock/Self-Screening Jammer",
      Self::BoostedSelfScreeningJammer => "Stock/Boosted Self-Screening Jammer",
      Self::DirectGuidance => "Stock/Direct Guidance",
      Self::CruiseGuidance => "Stock/Cruise Guidance",
      Self::HEImpact => "Stock/HE Impact",
      Self::HEKineticPenetrator => "Stock/HE Kinetic Penetrator",
      Self::BlastFragmentation => "Stock/Blast Fragmentation",
      Self::BlastFragmentationEL => "Stock/Blast Fragmentation EL",
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
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32
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
pub struct Vector3<T> {
  pub x: T,
  pub y: T,
  pub z: T
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
pub struct Vector2<T> {
  pub x: T,
  pub y: T
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
