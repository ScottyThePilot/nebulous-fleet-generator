//! XML Parser/Encoder wrapper and deserialization/serialization framework.

pub extern crate uuid;

use rxml::{EventRead, PullParser};
use rxml::parser::{ResolvedEvent, ResolvedQName};
use rxml::strings::{CDataStr, NameStr};
use rxml::writer::{Encoder, SimpleNamespaces, Item};
pub use rxml::parser::XmlVersion;
use thiserror::Error;

use std::collections::HashMap;
use std::convert::Infallible;
use std::io::BufRead;
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
    self.into_iter()
      .map(|value| T::serialize_element(value).map(Node::Element))
      .collect::<Result<Vec<Node>, Self::Error>>()
      .map(Nodes::from)
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
  std::net::SocketAddrV6,
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

pub type Attributes = HashMap<Box<str>, Box<str>>;

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
    Element { name: name.into(), attributes: HashMap::new(), children: children.into() }
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

  pub fn find_attribute(&self, attribute_name: &str) -> Result<&Box<str>, Error> {
    self.attributes.get(attribute_name).ok_or_else(|| {
      Error::MissingAttribute(self.attributes.clone(), attribute_name.into())
    })
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodes {
  pub nodes: Box<[Node]>
}

impl Nodes {
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
    Nodes { nodes: Box::new([]) }
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

impl From<Nodes> for Vec<Node> {
  fn from(value: Nodes) -> Self {
    value.nodes.into_vec()
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



pub fn write_nodes(nodes: &Nodes, version: Option<XmlVersion>) -> Result<Vec<u8>, Error> {
  let mut buffer = Vec::new();
  let mut encoder = Encoder::new();
  if let Some(version) = version { encoder.encode(Item::XmlDeclaration(version), &mut buffer)? };
  push_nodes_recursive(&mut buffer, &mut encoder, nodes)?;
  Ok(buffer)
}

fn push_nodes_recursive(
  buffer: &mut Vec<u8>,
  encoder: &mut Encoder<SimpleNamespaces>,
  nodes: &Nodes
) -> Result<(), Error> {
  for node in nodes.iter_raw() {
    match node {
      Node::Element(element) => {
        push_element_recursive(buffer, encoder, element)?;
      },
      Node::Text(text) => {
        let text = CDataStr::from_str(text)?;
        encoder.encode(Item::Text(text), buffer)?;
      }
    };
  };

  Ok(())
}

fn push_element_recursive(
  buffer: &mut Vec<u8>,
  encoder: &mut Encoder<SimpleNamespaces>,
  element: &Element
) -> Result<(), Error> {
  let (namespace, local) = NameStr::from_str(&element.name)?.split_name()?;
  let namespace = namespace.map(|ns| encoder.inner().lookup_prefix(Some(ns))).transpose()?;
  encoder.encode(Item::ElementHeadStart(namespace, local), buffer)?;
  for (name, value) in element.attributes.iter() {
    let (namespace, local) = NameStr::from_str(&name)?.split_name()?;
    let namespace = namespace.map(|ns| encoder.inner().lookup_prefix(Some(ns))).transpose()?;
    let value = CDataStr::from_str(value)?;
    encoder.encode(Item::Attribute(namespace, local, value), buffer)?;
  };
  encoder.encode(Item::ElementHeadEnd, buffer)?;
  push_nodes_recursive(buffer, encoder, &element.children)?;
  encoder.encode(Item::ElementFoot, buffer)?;
  Ok(())
}

pub fn read_nodes<R: BufRead>(reader: R) -> Result<Nodes, Error> {
  pull_nodes_recursive(&mut PullParser::new(reader))
}

fn pull_nodes_recursive<R: BufRead>(reader: &mut PullParser<R>) -> Result<Nodes, Error> {
  let mut nodes = Vec::new();
  while let Some(resolved_event) = reader.read()? {
    match resolved_event {
      ResolvedEvent::XmlDeclaration(_, _) => (),
      ResolvedEvent::StartElement(_, name, attributes) => {
        let name = join_name(name);
        let attributes = attributes.into_iter()
          .map(|(key, value)| (join_name(key), String::from(value).into_boxed_str()))
          .collect::<HashMap<Box<str>, Box<str>>>();
        let children = pull_nodes_recursive(reader)?;
        nodes.push(Node::Element(Element { name, attributes, children }));
      },
      ResolvedEvent::EndElement(_) => break,
      ResolvedEvent::Text(_, text) => {
        let content = String::from(text);
        nodes.push(Node::Text(content));
      }
    };
  };

  Ok(Nodes::from(nodes))
}

fn join_name(name: ResolvedQName) -> Box<str> {
  let (namespace, local) = name;
  String::into_boxed_str(match namespace {
    Some(namespace) => format!("{}:{}", namespace.as_str(), local.as_str()),
    None => String::from(local)
  })
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  XmlParserError(#[from] rxml::error::Error),
  #[error(transparent)]
  XmlEncoderError(#[from] rxml::writer::EncodeError),
  #[error("undeclared namespace")]
  XmlEncoderPrefixError(rxml::writer::PrefixError),
  #[error("unexpected duplicate element {:?}", .0.name)]
  UnexpectedElementDuplicate(Element),
  #[error("unexpected element {:?}, expected text", .0.name)]
  UnexpectedElementExpectedText(Element),
  #[error("unexpected element {:?}, expected element {:?}", .0.name, .1)]
  UnexpectedElementExpectedElement(Element, Box<str>),
  #[error("unexpected text {:?}, expected element {:?}", .0, .1)]
  UnexpectedTextExpectedElement(String, Box<str>),
  #[error("unexpected text {:?}", .0)]
  UnexpectedText(String),
  #[error("missing element {:?}", .0)]
  MissingElement(Box<str>),
  #[error("missing attribute {:?} in attributes map {:?}", .1, .0)]
  MissingAttribute(HashMap<Box<str>, Box<str>>, Box<str>),
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
}

impl From<rxml::error::XmlError> for Error {
  fn from(value: rxml::error::XmlError) -> Self {
    Error::XmlParserError(rxml::error::Error::Xml(value))
  }
}

impl From<rxml::writer::PrefixError> for Error {
  fn from(value: rxml::writer::PrefixError) -> Self {
    Error::XmlEncoderPrefixError(value)
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
