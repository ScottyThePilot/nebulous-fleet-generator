use crate::data::Faction;
use crate::data::hulls::HullKey;
use crate::data::components::ComponentKey;

use xml::uuid::Uuid;
use xml::{DeserializeElement, SerializeElement, Element, Node, Nodes};

use std::collections::HashSet;
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ship {
  pub key: Uuid,
  pub name: String,
  pub cost: usize,
  pub callsign: Option<String>,
  pub number: usize,
  pub hull_type: HullKey,
  pub socket_map: Vec<SocketEntry>,
  pub weapon_groups: Vec<WeaponGroup>,
  pub missile_types: Vec<MissileTemplate>
}

impl DeserializeElement for Ship {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    element.expect_named("Ship")?;
    let [key, name, cost, callsign, number, hull_type, socket_map, weapon_groups, missile_types] = element.children
      .find_elements(["Key", "Name", "Cost", "Callsign", "Number", "HullType", "SocketMap", "WeaponGroups", "TemplateMissileTypes"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<Uuid>()?;
    let name = name.ok_or(xml::Error::missing_element("Name"))?.children.deserialize::<String>()?;
    let cost = cost.ok_or(xml::Error::missing_element("Cost"))?.children.deserialize::<usize>()?;
    let callsign = callsign.map(|callsign| callsign.children.deserialize::<String>()).transpose()?;
    let number = number.ok_or(xml::Error::missing_element("Number"))?.children.deserialize::<usize>()?;
    let hull_type = hull_type.ok_or(xml::Error::missing_element("HullType"))?.children.deserialize::<HullKey>()?;
    let socket_map = socket_map.ok_or(xml::Error::missing_element("SocketMap"))?.children.deserialize::<Vec<SocketEntry>>()?;
    let weapon_groups = weapon_groups.ok_or(xml::Error::missing_element("WeaponGroups"))?.children.deserialize::<Vec<WeaponGroup>>()?;
    let missile_types = missile_types.ok_or(xml::Error::missing_element("TemplateMissileTypes"))?.children.deserialize::<Vec<MissileTemplate>>()?;

    Ok(Ship { key, name, cost, callsign, number, hull_type, socket_map, weapon_groups, missile_types })
  }
}

impl SerializeElement for Ship {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {

    Ok(todo!())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketEntry {
  pub key: Box<str>,
  pub component_name: ComponentKey,
  pub component_data: Option<ComponentData>
}

impl DeserializeElement for SocketEntry {
  type Error = FormatError;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    let [key, component_name, component_data] = element.children
      .find_elements(["Key", "ComponentName", "ComponentData"])?;

    let key = key.ok_or(xml::Error::missing_element("Key"))?.children.deserialize::<String>()?.into_boxed_str();
    let component_name = component_name.ok_or(xml::Error::missing_element("ComponentName"))?.children.deserialize::<ComponentKey>()?;
    let component_data = component_data.map(Element::deserialize::<ComponentData>).transpose()?;

    Ok(SocketEntry { key, component_name, component_data })
  }
}

impl SerializeElement for SocketEntry {
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

    let mut members = HashSet::new();
    for element in member_keys.children.try_into_elements_vec()? {
      element.expect_named("string")?;
      let member = element.children.deserialize::<String>()?;
      members.insert(member.into_boxed_str());
    };

    Ok(WeaponGroup { name, members })
  }
}

impl SerializeElement for WeaponGroup {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    let member_keys = self.members.into_iter()
      .map(|member| Element::new("string", Nodes::new_text(member)))
      .collect::<Nodes>();
    let member_keys = Element::new("MemberKeys", member_keys);
    let attributes = std::iter::once(("Name".into(), self.name.into_boxed_str())).collect();
    let weapon_group = Element::with_attributes("WepGroup", attributes, Nodes::new_one(member_keys));
    Ok(weapon_group)
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
