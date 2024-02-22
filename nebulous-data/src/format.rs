use crate::data::Faction;
use crate::data::hulls::HullKey;
use crate::data::components::ComponentKey;

use xml::uuid::Uuid;
use xml::{DeserializeElement, DeserializeNodes, SerializeElement, SerializeNodes, Element, Node, Nodes};

use std::collections::{HashSet, HashMap};
use std::convert::Infallible;



#[derive(Debug, Error)]
pub enum FormatError {
  #[error(transparent)]
  XmlError(#[from] xml::Error)
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
    let description = description.map(|description| description.children.deserialize::<String>()).transpose()?;
    let ships = ships.ok_or(xml::Error::missing_element("Ships"))?.children.deserialize::<Vec<Ship>>()?;
    let missile_types = missile_types.ok_or(xml::Error::missing_element("MissileTypes"))?.children.deserialize::<Vec<MissileTemplate>>()?;

    Ok(Fleet { name, total_points, faction_key, description, ships, missile_types })
  }
}

impl SerializeElement for Fleet {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
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
  pub hull_config: Option<HullConfig>,
  pub socket_map: Vec<HullSocket>,
  pub weapon_groups: Vec<WeaponGroup>,
  pub missile_types: Vec<MissileTemplate>
}

impl DeserializeElement for Ship {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("Ship")?;
    let [key, name, cost, callsign, number, hull_type, hull_config, socket_map, weapon_groups, missile_types] = element.children
      .find_elements(["Key", "Name", "Cost", "Callsign", "Number", "HullType", "HullConfig", "SocketMap", "WeaponGroups", "TemplateMissileTypes"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Uuid>()?;
    let name = name.ok_or(xml::Error::missing_element("Name"))?.children.deserialize::<String>()?;
    let cost = cost.ok_or(xml::Error::missing_element("Cost"))?.children.deserialize::<usize>()?;
    let callsign = callsign.map(|callsign| callsign.children.deserialize::<String>()).transpose()?;
    let number = number.ok_or(xml::Error::missing_element("Number"))?.children.deserialize::<usize>()?;
    let hull_type = hull_type.ok_or(xml::Error::missing_element("HullType"))?.children.deserialize::<HullKey>()?;
    let hull_config = hull_config.map(|hull_config| hull_config.deserialize::<HullConfig>()).transpose()?;
    let socket_map = socket_map.ok_or(xml::Error::missing_element("SocketMap"))?.children.deserialize::<Vec<HullSocket>>()?;
    let weapon_groups = weapon_groups.ok_or(xml::Error::missing_element("WeaponGroups"))?.children.deserialize::<Vec<WeaponGroup>>()?;
    let missile_types = missile_types.ok_or(xml::Error::missing_element("TemplateMissileTypes"))?.children.deserialize::<Vec<MissileTemplate>>()?;

    Ok(Ship { key, name, cost, callsign, number, hull_type, hull_config, socket_map, weapon_groups, missile_types })
  }
}

impl SerializeElement for Ship {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let save_id = Element::with_attributes("SaveID", xsi_nil(), Nodes::default());
    let key = Element::new("Key", self.key.serialize_nodes()?);
    let name = Element::new("Name", self.name.serialize_nodes()?);
    let cost = Element::new("Cost", self.cost.serialize_nodes()?);
    let callsign = Element::new("Callsign", self.callsign.map_or(Ok(Nodes::default()), String::serialize_nodes)?);
    let number = Element::new("Number", self.number.serialize_nodes()?);
    let symbol_option = Element::new("SymbolOption", Nodes::new_text("0"));
    let hull_type = Element::new("HullType", self.hull_type.serialize_nodes()?);
    let hull_config = self.hull_config.map(HullConfig::serialize_element).transpose()?;
    let socket_map = Element::new("SocketMap", self.socket_map.serialize_nodes()?);
    let weapon_groups = Element::new("WeaponGroups", self.weapon_groups.serialize_nodes()?);
    let missile_types = Element::new("TemplateMissileTypes", self.missile_types.serialize_nodes()?);

    let a = [save_id, key, name, cost, callsign, number, symbol_option, hull_type];
    let b = [socket_map, weapon_groups, missile_types];

    let nodes = Nodes::from_iter(a.into_iter().chain(hull_config).chain(b));
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
    let nodes = [key, component_name].into_iter().chain(component_data).collect::<Nodes>();
    Ok(Element::new("HullSocket", nodes))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentData {
  BulkMagazineData,
  CellLauncherData,
  ResizableCellLauncherData
}

impl DeserializeElement for ComponentData {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {

    Ok(todo!())
  }
}

impl SerializeElement for ComponentData {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponGroup {
  pub name: String,
  pub members: HashSet<Box<str>>
}

impl DeserializeElement for WeaponGroup {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("WepGroup")?;
    let name = element.find_attribute("Name")?.as_ref().to_owned();

    let member_keys = element.children.try_into_one_element()?;
    member_keys.expect_named("MemberKeys")?;

    let members = member_keys.children.deserialize_named_elements::<Box<str>, HashSet<Box<str>>>("string")?;

    Ok(WeaponGroup { name, members })
  }
}

impl SerializeElement for WeaponGroup {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let member_keys = xml::serialize_named_elements(self.members, "string")?;
    let member_keys = Element::new("MemberKeys", member_keys);
    let attributes = xml::attributes!("Name" = self.name);
    let weapon_group = Element::with_attributes("WepGroup", attributes, Nodes::new_one(member_keys));
    Ok(weapon_group)
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

    Ok(todo!())
  }
}

impl SerializeElement for HullConfig {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryStructureConfig {
  pub key: Uuid,
  pub segment: usize,
  pub snap_point: usize
}

impl DeserializeElement for SecondaryStructureConfig {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {

    Ok(todo!())
  }
}

impl SerializeElement for SecondaryStructureConfig {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissileTemplate {}

impl DeserializeElement for MissileTemplate {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {

    Ok(todo!())
  }
}

impl SerializeElement for MissileTemplate {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
  }
}

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



fn xsi_nil() -> HashMap<Box<str>, Box<str>> {
  xml::attributes!("xsi:nil" = "true")
}
