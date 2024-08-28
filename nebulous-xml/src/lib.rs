//! XML Parser/Encoder wrapper and deserialization/serialization framework.

#[cfg(feature = "uuid")]
pub extern crate uuid;

use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use quick_xml::encoding::Decoder;
use quick_xml::escape::unescape;
use quick_xml::name::QName;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText, BytesDecl};
use quick_xml::events::attributes::{Attribute, Attributes as AttributesIter};
use thiserror::Error;

use std::convert::Infallible;
use std::io::{BufRead, Write};
use std::iter::Filter;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter as SliceIter, IterMut as SliceIterMut};
use std::str::FromStr;
use std::vec::IntoIter as VecIntoIter;



#[macro_export]
macro_rules! attributes {
  ($($name:literal = $value:expr),* $(,)?) => {
    $crate::Attributes::from_iter([
      $((std::boxed::Box::<str>::from($name), std::boxed::Box::<str>::from($value))),*
    ])
  };
}

pub trait DeserializeNodes: Sized {
  type Error;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error>;
}

pub trait DeserializeElement: Sized {
  type Error;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error>;
}

impl DeserializeNodes for Nodes {
  type Error = Infallible;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    Ok(nodes)
  }
}

impl DeserializeElement for Element {
  type Error = Infallible;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    Ok(element)
  }
}

impl DeserializeNodes for String {
  type Error = Error;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    nodes.try_into_text().map_err(Error::unexpected_element_expected_text)
  }
}

impl DeserializeNodes for Box<str> {
  type Error = Error;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    String::deserialize_nodes(nodes).map(String::into_boxed_str)
  }
}

impl<T> DeserializeNodes for Vec<T> where T: DeserializeElement {
  type Error = DeserializeErrorWrapper<T::Error>;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    nodes.into_iter_raw()
      .filter_map(Node::into_element_ignore_whitespace)
      .map(|r| r.map_err(Error::unexpected_text).map_err(DeserializeErrorWrapper::Error))
      .map(|r| r.and_then(|r| T::deserialize_element(r).map_err(DeserializeErrorWrapper::Inner)))
      .collect()
  }
}

impl<T, const N: usize> DeserializeNodes for [T; N] where T: DeserializeElement {
  type Error = DeserializeErrorWrapper<T::Error>;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    let elements = nodes.try_into_elements_array::<N>()?;
    let elements = elements.into_iter()
      .map(|v| T::deserialize_element(v).map_err(DeserializeErrorWrapper::Inner))
      .collect::<Result<Vec<T>, DeserializeErrorWrapper<T::Error>>>()?;
    Ok(elements.try_into().ok().expect("infallible"))
  }
}

impl<T> DeserializeNodes for Box<T> where T: DeserializeNodes {
  type Error = T::Error;

  fn deserialize_nodes(nodes: Nodes) -> Result<Self, Self::Error> {
    T::deserialize_nodes(nodes).map(Box::new)
  }
}

impl<T> DeserializeElement for Box<T> where T: DeserializeElement {
  type Error = T::Error;

  fn deserialize_element(element: Element) -> Result<Self, Self::Error> {
    T::deserialize_element(element).map(Box::new)
  }
}



pub trait SerializeNodes: Sized {
  type Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error>;
}

pub trait SerializeElement: Sized {
  type Error;

  fn serialize_element(self) -> Result<Element, Self::Error>;
}

impl SerializeNodes for Nodes {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    Ok(self)
  }
}

impl SerializeElement for Element {
  type Error = Infallible;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    Ok(self)
  }
}

impl SerializeNodes for String {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    Ok(Nodes::new_text(self))
  }
}

impl SerializeNodes for Box<str> {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    Ok(Nodes::new_text(self))
  }
}

impl<T> SerializeNodes for Vec<T> where T: SerializeElement {
  type Error = T::Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    self.into_iter().map(T::serialize_element).collect()
  }
}

impl<T, const N: usize> SerializeNodes for [T; N] where T: SerializeElement {
  type Error = T::Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    self.into_iter().map(T::serialize_element).collect()
  }
}

impl<T> SerializeNodes for Box<T> where T: SerializeNodes {
  type Error = T::Error;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    (*self).serialize_nodes()
  }
}

impl<T> SerializeElement for Box<T> where T: SerializeElement {
  type Error = T::Error;

  fn serialize_element(self) -> Result<Element, Self::Error> {
    (*self).serialize_element()
  }
}



pub fn serialize_nodes_display<T: ToString>(value: &T) -> Nodes {
  Nodes::new_text(value.to_string())
}

pub fn deserialize_nodes_parse<T: FromStr>(nodes: Nodes) -> Result<T, Error>
where T::Err: std::error::Error + Send + Sync + 'static {
  nodes.try_into_text()
    .map_err(Error::unexpected_element_expected_text)
    .and_then(|content| {
      content.parse().map_err(|err| Error::ParseError(Box::new(err), content))
    })
}

#[macro_export]
macro_rules! impl_deserialize_nodes_parse {
  ($($Type:ty),* $(,)?) => {
    $(impl $crate::DeserializeNodes for $Type {
      type Error = $crate::Error;

      #[inline]
      fn deserialize_nodes(nodes: $crate::Nodes) -> std::result::Result<Self, Self::Error> {
        $crate::deserialize_nodes_parse(nodes)
      }
    })*
  };
}

impl_deserialize_nodes_parse! {
  bool, char,
  f32, f64,
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  std::num::NonZeroI8,
  std::num::NonZeroI16,
  std::num::NonZeroI32,
  std::num::NonZeroI64,
  std::num::NonZeroI128,
  std::num::NonZeroIsize,
  std::num::NonZeroU8,
  std::num::NonZeroU16,
  std::num::NonZeroU32,
  std::num::NonZeroU64,
  std::num::NonZeroU128,
  std::num::NonZeroUsize,
  std::net::IpAddr,
  std::net::Ipv4Addr,
  std::net::Ipv6Addr,
  std::net::SocketAddr,
  std::net::SocketAddrV4,
  std::net::SocketAddrV6
}

#[cfg(feature = "uuid")]
impl_deserialize_nodes_parse! {
  uuid::Uuid
}

#[macro_export]
macro_rules! impl_serialize_nodes_display {
  ($($Type:ty),* $(,)?) => {
    $(impl $crate::SerializeNodes for $Type {
      type Error = std::convert::Infallible;

      fn serialize_nodes(self) -> std::result::Result<$crate::Nodes, Self::Error> {
        std::result::Result::Ok($crate::serialize_nodes_display(&self))
      }
    })*
  };
}

impl_serialize_nodes_display! {
  bool, char,
  f32, f64,
  i8, i16, i32, i64, i128, isize,
  u8, u16, u32, u64, u128, usize,
  std::num::NonZeroI8,
  std::num::NonZeroI16,
  std::num::NonZeroI32,
  std::num::NonZeroI64,
  std::num::NonZeroI128,
  std::num::NonZeroIsize,
  std::num::NonZeroU8,
  std::num::NonZeroU16,
  std::num::NonZeroU32,
  std::num::NonZeroU64,
  std::num::NonZeroU128,
  std::num::NonZeroUsize,
  std::net::IpAddr,
  std::net::Ipv4Addr,
  std::net::Ipv6Addr,
  std::net::SocketAddr,
  std::net::SocketAddrV4,
  std::net::SocketAddrV6
}

#[cfg(feature = "uuid")]
impl SerializeNodes for uuid::Uuid {
  type Error = Infallible;

  fn serialize_nodes(self) -> Result<Nodes, Self::Error> {
    Ok(Nodes::new_text(self.hyphenated().to_string()))
  }
}



pub fn serialize_named_elements<I, T>(iterator: I, name: &str) -> Result<Nodes, T::Error>
where I: IntoIterator<Item = T>, T: SerializeNodes {
  iterator.into_iter()
    .map(|member| Ok(Element::new(name, member.serialize_nodes()?)))
    .collect::<Result<Nodes, T::Error>>()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
  Text(String),
  Element(Element)
}

impl Node {
  pub fn into_text(self) -> Result<String, Element> {
    match self {
      Node::Text(content) => Ok(content),
      Node::Element(element) => Err(element)
    }
  }

  pub fn into_element(self) -> Result<Element, String> {
    match self {
      Node::Text(content) => Err(content),
      Node::Element(element) => Ok(element)
    }
  }

  fn into_element_ignore_whitespace(self) -> Option<Result<Element, String>> {
    match self {
      Node::Text(content) if is_whitespace(&content) => None,
      Node::Text(content) => Some(Err(content)),
      Node::Element(element) => Some(Ok(element))
    }
  }

  pub fn is_whitespace(&self) -> bool {
    matches!(self, Node::Text(content) if is_whitespace(content))
  }
}

impl From<Element> for Node {
  fn from(value: Element) -> Self {
    Node::Element(value)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
  pub name: Box<str>,
  pub attributes: Attributes,
  pub children: Nodes
}

impl Element {
  pub fn new(name: impl Into<Box<str>>, children: impl Into<Nodes>) -> Self {
    Element { name: name.into(), attributes: Attributes::new(), children: children.into() }
  }

  pub fn with_attributes(name: impl Into<Box<str>>, attributes: Attributes, children: impl Into<Nodes>) -> Self {
    Element { name: name.into(), attributes, children: children.into() }
  }

  #[inline]
  pub fn deserialize<T: DeserializeElement>(self) -> Result<T, T::Error> {
    T::deserialize_element(self)
  }

  pub fn expect_named(&self, name: &str) -> Result<(), Error> {
    if self.name.as_ref() != name { Err(Error::UnexpectedElementExpectedElement(self.clone(), name.into())) } else { Ok(()) }
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodes {
  pub nodes: Box<[Node]>
}

impl Nodes {
  pub fn new() -> Self {
    Nodes::default()
  }

  pub fn new_text(text: impl Into<String>) -> Self {
    Nodes { nodes: Box::new([Node::Text(text.into())]) }
  }

  pub fn new_one(node: impl Into<Node>) -> Self {
    Nodes { nodes: Box::new([node.into()]) }
  }

  #[inline]
  pub fn deserialize<T: DeserializeNodes>(self) -> Result<T, T::Error> {
    T::deserialize_nodes(self)
  }

  pub fn deserialize_named_elements<T, C>(self, name: &str) -> Result<C, DeserializeErrorWrapper<T::Error>>
  where T: DeserializeNodes, C: FromIterator<T> {
    self.into_iter_raw()
      .filter_map(Node::into_element_ignore_whitespace)
      .map(|element| {
        let element = element.map_err(Error::unexpected_text)?;
        element.expect_named(name)?;
        element.children.deserialize::<T>()
          .map_err(DeserializeErrorWrapper::Inner)
      })
      .collect()
  }

  /// If all of these nodes are text nodes, returns a string with their contents,
  /// otherwise returns the first non-text node (element) that was found.
  /// If there are no nodes, an empty string is returned.
  pub fn try_into_text(self) -> Result<String, Element> {
    match <[Node; 1]>::try_from(self.nodes.into_vec()) {
      Ok([node]) => node.into_text(),
      Err(nodes) => {
        let mut out = String::new();
        for node in nodes {
          let text = node.into_text()?;
          out.push_str(&text);
        };

        Ok(out)
      }
    }
  }

  /// Attempts to resolve this nodes list as `N` nodes, ignoring any whitespace nodes if present.
  pub fn try_into_nodes_array<const N: usize>(self) -> Result<[Node; N], Error> {
    <[Node; N]>::try_from(self.into_iter().collect::<Vec<Node>>())
      .map_err(|nodes| Error::IncorrectNodesCount(nodes, N))
  }

  /// Attempts to resolve this nodes list as a list of element nodes.
  /// If any non-whitespace text nodes are encountered, they will be returned as an error.
  pub fn try_into_elements_vec(self) -> Result<Vec<Element>, Error> {
    self.into_iter_raw()
      .filter_map(Node::into_element_ignore_whitespace)
      .collect::<Result<Vec<Element>, String>>()
      .map_err(Error::unexpected_text)
  }

  /// Attempts to resolve this nodes list as `N` element nodes.
  /// If any non-whitespace text nodes are encountered, they will be returned as an error.
  pub fn try_into_elements_array<const N: usize>(self) -> Result<[Element; N], Error> {
    <[Element; N]>::try_from(self.try_into_elements_vec()?)
      .map_err(|elements| Error::IncorrectElementsCount(elements, N))
  }

  /// Attempts to resolve this nodes list as one node, ignoring any whitespace nodes if present.
  pub fn try_into_one_node(self) -> Result<Node, Error> {
    self.try_into_nodes_array::<1>().map(|[node]| node)
  }

  /// Attempts to resolve this nodes list as one element node.
  /// If any non-whitespace text nodes are encountered, they will be returned as an error.
  pub fn try_into_one_element(self) -> Result<Element, Error> {
    self.try_into_elements_array::<1>().map(|[element]| element)
  }

  /// Searches this nodes list for the given element names, returning a list of elements associated with those names.
  pub fn find_elements<const N: usize>(self, names: [&str; N]) -> Result<[Option<Element>; N], Error> {
    let mut elements: [_; N] = std::array::from_fn(|_| None);
    for node in self.into_iter_raw() {
      let Node::Element(element) = node else { continue };
      let Some(i) = names.iter().position(|&n| n == element.name.as_ref()) else { continue };
      if let Some(element) = elements[i].replace(element) {
        return Err(Error::UnexpectedElementDuplicate(element));
      };
    };

    Ok(elements)
  }

  #[inline]
  pub fn iter(&self) -> IterNodesRef {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterNodesMut {
    self.into_iter()
  }

  #[inline]
  pub fn into_iter_raw(self) -> VecIntoIter<Node> {
    self.nodes.into_vec().into_iter()
  }

  #[inline]
  pub fn iter_raw(&self) -> SliceIter<Node> {
    self.nodes.iter()
  }

  #[inline]
  pub fn iter_mut_raw(&mut self) -> SliceIterMut<Node> {
    self.nodes.iter_mut()
  }
}

impl Default for Nodes {
  fn default() -> Self {
    Nodes { nodes: Vec::new().into_boxed_slice() }
  }
}

impl IntoIterator for Nodes {
  type Item = Node;
  type IntoIter = IterNodes;

  fn into_iter(self) -> Self::IntoIter {
    self.into_iter_raw().filter(|node| !node.is_whitespace())
  }
}

impl<'a> IntoIterator for &'a Nodes {
  type Item = &'a Node;
  type IntoIter = IterNodesRef<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter_raw().filter(|node| !node.is_whitespace())
  }
}

impl<'a> IntoIterator for &'a mut Nodes {
  type Item = &'a mut Node;
  type IntoIter = IterNodesMut<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut_raw().filter(|node| !node.is_whitespace())
  }
}

pub type IterNodes = Filter<VecIntoIter<Node>, fn(&Node) -> bool>;
pub type IterNodesRef<'a> = Filter<SliceIter<'a, Node>, fn(&&'a Node) -> bool>;
pub type IterNodesMut<'a> = Filter<SliceIterMut<'a, Node>, fn(&&'a mut Node) -> bool>;

impl FromIterator<Element> for Nodes {
  fn from_iter<T: IntoIterator<Item = Element>>(iter: T) -> Self {
    iter.into_iter().map(Node::from).collect()
  }
}

impl FromIterator<Node> for Nodes {
  fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
    Nodes { nodes: iter.into_iter().collect() }
  }
}

impl<const N: usize> From<[Node; N]> for Nodes {
  fn from(value: [Node; N]) -> Self {
    value.into_iter().collect()
  }
}

impl From<Box<[Node]>> for Nodes {
  fn from(nodes: Box<[Node]>) -> Self {
    Nodes { nodes }
  }
}

impl From<Vec<Node>> for Nodes {
  fn from(nodes: Vec<Node>) -> Self {
    Nodes { nodes: nodes.into_boxed_slice() }
  }
}

impl From<Nodes> for Box<[Node]> {
  fn from(value: Nodes) -> Self {
    value.nodes
  }
}

impl Deref for Nodes {
  type Target = [Node];

  fn deref(&self) -> &Self::Target {
    self.nodes.as_ref()
  }
}

impl DerefMut for Nodes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.nodes.as_mut()
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
  pub list: Box<[(Box<str>, Box<str>)]>
}

impl Attributes {
  pub fn new() -> Self {
    Attributes::default()
  }

  pub fn find_attributes<const N: usize>(self, names: [&str; N]) -> Result<[Option<Box<str>>; N], Error> {
    let mut attributes: [_; N] = std::array::from_fn(|_| None);
    for (name, value) in self.list.into_vec().into_iter() {
      let Some(i) = names.iter().position(|&n| n == name.as_ref()) else { continue };
      if let Some(value) = attributes[i].replace(value) {
        return Err(Error::UnexpectedAttributeDuplicate(name, value));
      };
    };

    Ok(attributes)
  }
}

impl Default for Attributes {
  fn default() -> Self {
    Attributes { list: Vec::new().into_boxed_slice() }
  }
}

impl<const N: usize> From<[(Box<str>, Box<str>); N]> for Attributes {
  fn from(value: [(Box<str>, Box<str>); N]) -> Self {
    Attributes { list: Box::new(value) }
  }
}

impl From<Box<[(Box<str>, Box<str>)]>> for Attributes {
  fn from(value: Box<[(Box<str>, Box<str>)]>) -> Self {
    Attributes { list: value }
  }
}

impl From<Vec<(Box<str>, Box<str>)>> for Attributes {
  fn from(value: Vec<(Box<str>, Box<str>)>) -> Self {
    Attributes { list: value.into_boxed_slice() }
  }
}

impl From<Attributes> for Box<[(Box<str>, Box<str>)]> {
  fn from(value: Attributes) -> Self {
    value.list
  }
}

impl FromIterator<(Box<str>, Box<str>)> for Attributes {
  fn from_iter<T: IntoIterator<Item = (Box<str>, Box<str>)>>(iter: T) -> Self {
    Attributes { list: iter.into_iter().collect() }
  }
}

impl Deref for Attributes {
  type Target = [(Box<str>, Box<str>)];

  fn deref(&self) -> &Self::Target {
    &self.list
  }
}

impl DerefMut for Attributes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.list
  }
}



pub fn assert_roundtrip_nodes<T>(value: &T)
where
  T: DeserializeNodes + SerializeNodes + PartialEq + Clone + std::fmt::Debug,
  <T as DeserializeNodes>::Error: std::fmt::Debug,
  <T as SerializeNodes>::Error: std::fmt::Debug,
{
  let nodes = value.clone().serialize_nodes().expect("failed to round-trip");
  let value2 = nodes.deserialize::<T>().expect("failed to round-trip");
  assert_eq!(value, &value2, "failed to round-trip: values not equal");
}

pub fn assert_roundtrip_element<T>(value: &T)
where
  T: DeserializeElement + SerializeElement + PartialEq + Clone + std::fmt::Debug,
  <T as DeserializeElement>::Error: std::fmt::Debug,
  <T as SerializeElement>::Error: std::fmt::Debug,
{
  let element = value.clone().serialize_element().expect("failed to round-trip");
  let value2 = element.deserialize::<T>().expect("failed to round-trip");
  assert_eq!(value, &value2, "failed to round-trip: values not equal");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
  pub version: &'static str,
  pub encoding: Option<&'static str>,
  pub standalone: Option<&'static str>
}

impl Version {
  pub const VERSION_1_0: Self = Version {
    version: "1.0",
    encoding: None,
    standalone: None
  };
}

impl Default for Version {
  fn default() -> Self {
    Self::VERSION_1_0
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indent {
  pub indent_char: u8,
  pub indent_size: usize
}

impl Default for Indent {
  fn default() -> Self {
    // And so the indentation wars raged on...
    Indent {
      indent_char: b' ',
      indent_size: 2
    }
  }
}

pub fn write_nodes<W: Write>(writer: W, nodes: &Nodes, indent: Option<Indent>, version: Option<Version>) -> Result<(), Error> {
  let mut writer = if let Some(i) = indent {
    Writer::new_with_indent(writer, i.indent_char, i.indent_size)
  } else {
    Writer::new(writer)
  };

  if let Some(Version { version, encoding, standalone }) = version {
    writer.write_event(Event::Decl(BytesDecl::new(version, encoding, standalone)))?;
  };

  push_nodes_recursive(&mut writer, nodes)?;

  Ok(())
}

pub fn push_nodes_recursive<W: Write>(writer: &mut Writer<W>, nodes: &Nodes) -> Result<(), quick_xml::Error> {
  for node in nodes.iter_raw() {
    match node {
      Node::Element(element) => {
        push_element_recursive(writer, element)?;
      },
      Node::Text(text) => {
        writer.write_event(Event::Text(BytesText::new(text)))?;
      }
    };
  };

  Ok(())
}

pub fn push_element_recursive<W: Write>(writer: &mut Writer<W>, element: &Element) -> Result<(), quick_xml::Error> {
  let attributes = element.attributes.iter().map(|(k, v)| Attribute::from((&**k, &**v)));
  let event_start = BytesStart::new(&*element.name).with_attributes(attributes);
  let event_end = BytesEnd::new(&*element.name);
  if element.children.is_empty() {
    writer.write_event(Event::Empty(event_start))?;
  } else {
    writer.write_event(Event::Start(event_start))?;
    push_nodes_recursive(writer, &element.children)?;
    writer.write_event(Event::End(event_end))?;
  };

  Ok(())
}

pub fn read_nodes<R: BufRead>(reader: R) -> Result<Nodes, Error> {
  pull_nodes_recursive(&mut Reader::from_reader(reader), &mut Vec::new()).map_err(From::from)
}

fn pull_nodes_recursive<R: BufRead>(reader: &mut Reader<R>, buf: &mut Vec<u8>) -> Result<Nodes, quick_xml::Error> {
  let mut nodes = Vec::new();
  loop {
    match reader.read_event_into(buf)? {
      Event::Start(event) => {
        let name = resolve_name(event.name(), reader.decoder())?;
        let attributes = resolve_attributes(event.attributes(), reader.decoder())?;
        let children = pull_nodes_recursive(reader, buf)?;
        nodes.push(Node::Element(Element::with_attributes(name, attributes, children)));
      },
      Event::End(..) => break,
      Event::Empty(event) => {
        let name = resolve_name(event.name(), reader.decoder())?;
        let attributes = resolve_attributes(event.attributes(), reader.decoder())?;
        nodes.push(Node::Element(Element::with_attributes(name, attributes, Nodes::default())));
      },
      Event::Text(mut event) => {
        event.inplace_trim_start();
        event.inplace_trim_end();
        let text = event.unescape()?.into_owned();
        nodes.push(Node::Text(text));
      },
      Event::CData(event) => {
        let content = reader.decoder().decode(event.into_inner().as_ref())?.into_owned();
        nodes.push(Node::Text(content));
      },
      // Ignored
      Event::Comment(..) => (),
      // Deserializing XML declarations, Processing Instructions, and DTDs are unsupported
      Event::Decl(..) | Event::PI(..) | Event::DocType(..) => (),
      Event::Eof => break
    };
  };

  Ok(Nodes::from(nodes))
}

fn resolve_name(name: QName, decoder: Decoder) -> Result<Box<str>, quick_xml::Error> {
  decoder.decode(name.into_inner()).map(|name| name.into_owned().into_boxed_str())
}

fn resolve_attributes(attributes: AttributesIter, decoder: Decoder) -> Result<Attributes, quick_xml::Error> {
  attributes.map(|result| result.map_err(quick_xml::Error::from))
    .map(|result| result.and_then(|attr| resolve_attribute(attr, decoder)))
    .collect::<Result<Attributes, quick_xml::Error>>()
}

fn resolve_attribute(attribute: Attribute, decoder: Decoder) -> Result<(Box<str>, Box<str>), quick_xml::Error> {
  let key = resolve_name(attribute.key, decoder)?;
  let value_decoded = decoder.decode(attribute.value.as_ref())?;
  let value = unescape(value_decoded.as_ref())?;
  Ok((key, value.into_owned().into_boxed_str()))
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  InnerError(#[from] quick_xml::Error),
  #[error("unexpected element {:?}, expected text", .0.name)]
  UnexpectedElementExpectedText(Element),
  #[error("unexpected element {:?}, expected element {:?}", .0.name, .1)]
  UnexpectedElementExpectedElement(Element, Box<str>),
  #[error("unexpected text {:?}, expected element {:?}", .0, .1)]
  UnexpectedTextExpectedElement(String, Box<str>),
  #[error("unexpected text {:?}", .0)]
  UnexpectedText(String),
  #[error("unexpected duplicate element {:?}", .0.name)]
  UnexpectedElementDuplicate(Element),
  #[error("unexpected duplicate attribute {:?} {:?}", .0, .1)]
  UnexpectedAttributeDuplicate(Box<str>, Box<str>),
  #[error("missing element {:?}", .0)]
  MissingElement(Box<str>),
  #[error("missing attribute {:?}", .0)]
  MissingAttribute(Box<str>),
  #[error("incorrect nodes count: found {}, expected {}", .0.len(), .1)]
  IncorrectNodesCount(Vec<Node>, usize),
  #[error("incorrect elements count: found {}, expected {}", .0.len(), .1)]
  IncorrectElementsCount(Vec<Element>, usize),
  #[error("failed to parse {1:?}: {0}")]
  ParseError(Box<dyn std::error::Error + Send + Sync + 'static>, String)
}

impl Error {
  pub fn unexpected_element_duplicate(element: Element) -> Self {
    Error::UnexpectedElementDuplicate(element)
  }

  pub fn unexpected_element_expected_text(element: Element) -> Self {
    Error::UnexpectedElementExpectedText(element)
  }

  pub fn unexpected_element_expected_element(element: Element, name: impl Into<Box<str>>) -> Self {
    Error::UnexpectedElementExpectedElement(element, name.into())
  }

  pub fn unexpected_text_expected_element(text: String, name: impl Into<Box<str>>) -> Self {
    Error::UnexpectedTextExpectedElement(text, name.into())
  }

  pub fn unexpected_text(text: String) -> Self {
    Error::UnexpectedText(text)
  }

  pub fn missing_element(name: impl Into<Box<str>>) -> Self {
    Error::MissingElement(name.into())
  }

  pub fn missing_attribute(name: impl Into<Box<str>>) -> Self {
    Error::MissingAttribute(name.into())
  }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
  fn from(error: quick_xml::events::attributes::AttrError) -> Self {
    Error::InnerError(error.into())
  }
}

#[derive(Debug, Error)]
pub enum DeserializeErrorWrapper<E> {
  #[error(transparent)]
  Inner(E),
  #[error(transparent)]
  Error(#[from] Error)
}

impl<E> DeserializeErrorWrapper<E> {
  pub fn convert(self) -> E where E: From<Error> {
    match self {
      Self::Inner(inner) => inner,
      Self::Error(error) => E::from(error)
    }
  }
}

impl<E> TryFrom<DeserializeErrorWrapper<E>> for Error {
  type Error = E;

  fn try_from(value: DeserializeErrorWrapper<E>) -> Result<Self, Self::Error> {
    match value {
      DeserializeErrorWrapper::Inner(inner) => Err(inner),
      DeserializeErrorWrapper::Error(error) => Ok(error)
    }
  }
}

fn is_whitespace(s: &str) -> bool {
  s.chars().all(|c| c.is_whitespace())
}
