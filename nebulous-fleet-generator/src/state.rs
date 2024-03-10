use nebulous_data::data::Faction;
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::components::ComponentKey;
use nebulous_data::data::munitions::MunitionKey;

use crate::parameters::*;

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt;



#[derive(Debug, Clone)]
pub struct FleetState {
  faction: Faction,
  ships: Vec<ShipState>
}

impl FleetState {
  pub const fn new(faction: Faction) -> Self {
    FleetState { faction, ships: Vec::new() }
  }

  pub const fn faction(&self) -> Faction {
    self.faction
  }

  pub fn ships(&self) -> &[ShipState] {
    &self.ships
  }

  pub fn point_budget(&self) -> usize {
    self.ships.iter().map(|ship| ship.point_budget).sum()
  }

  pub fn append_ship(mut self, ship: ShipState) -> Self {
    self.ships.push(ship);
    self
  }

  pub fn put_ship_component(mut self, ship: usize, component: usize, value: ComponentState) -> Self {
    let ship = &mut self.ships[ship];
    ship.components[component].replace(value);
    ship.update();
    self
  }

  pub fn make_sorted(mut self) -> Self {
    self.ships.sort_unstable_by_key(|ship| (Reverse(ship.hull_key), ship.purpose.clone()));
    self
  }

  pub fn similar(&self, other: &Self) -> bool {
    self.faction == other.faction &&
    self.ships.iter().zip(other.ships.iter())
      .all(|(a, b)| ShipState::similar(a, b))
  }
}

#[derive(Debug, Clone)]
pub struct ShipState {
  hull_key: HullKey,
  components: Box<[Option<ComponentState>]>,
  purpose: PurposeName,
  point_budget: usize,
  cache: ShipStateCache
}

impl ShipState {
  pub fn new(hull_key: HullKey, purpose: PurposeName, point_budget: usize) -> Self {
    let sockets = hull_key.hull().sockets.len();
    let components = (0..sockets).map(|_| None).collect();
    ShipState {
      hull_key, purpose, point_budget, components,
      cache: ShipStateCache::default()
    }
  }

  pub const fn hull_key(&self) -> HullKey {
    self.hull_key
  }

  pub const fn components(&self) -> &[Option<ComponentState>] {
    &self.components
  }

  pub const fn purpose(&self) -> &PurposeName {
    &self.purpose
  }

  pub const fn point_budget(&self) -> usize {
    self.point_budget
  }

  pub fn vacant_components(&self) -> impl Iterator<Item = usize> + '_ {
    self.components.iter().enumerate()
      .filter_map(|(i, c)| c.is_none().then_some(i))
  }

  fn update(&mut self) {
    self.cache.update_from(&self.components);
  }

  pub fn similar(&self, other: &Self) -> bool {
    self.hull_key == other.hull_key &&
    self.purpose == other.purpose
  }
}

impl fmt::Display for ShipState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} ({:?}, {} components)",
      self.hull_key.hull().name,
      self.purpose,
      self.components.len()
    )
  }
}

#[derive(Debug, Clone, Default)]
struct ShipStateCache {
  component_quantities: BTreeMap<ComponentKey, usize>
}

impl ShipStateCache {
  fn update_from(&mut self, components: &[Option<ComponentState>]) {
    self.component_quantities.clear();
    for component in components.iter().filter_map(Option::as_ref) {
      *self.component_quantities.entry(component.component_key).or_default() += 1;
    };
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentState {
  component_key: ComponentKey,
  magazine_contents: BTreeMap<MunitionState, usize>
}

impl ComponentState {
  pub const fn component_key(&self) -> ComponentKey {
    self.component_key
  }

  pub const fn magazine_contents(&self) -> &BTreeMap<MunitionState, usize> {
    &self.magazine_contents
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MunitionState {
  Munition(MunitionKey)
}

/// Iterates through hulls available to the given faction.
pub fn iter_faction_ships(faction: Faction) -> impl Iterator<Item = HullKey> + DoubleEndedIterator + Clone {
  HullKey::values().filter(move |&hull_key| hull_key.hull().faction == faction)
}

/// Iterates through components available to the given hull socket.
pub fn iter_socket_components(hull_key: HullKey, socket: usize) -> impl Iterator<Item = ComponentKey> + DoubleEndedIterator + Clone {
  let hull = hull_key.hull();
  let socket = &hull.sockets[socket];
  ComponentKey::values().filter(move |&component_key| {
    let component = component_key.component();
    component.is_usable_on(hull_key) && component.can_fit_in(socket.size)
  })
}
