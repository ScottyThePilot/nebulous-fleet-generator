use crate::parameters::*;

use nebulous_data::data::{Buff, Buffs, Faction};
use nebulous_data::data::hulls::{HullKey, HullSocket};
use nebulous_data::data::components::ComponentKey;
use nebulous_data::data::munitions::MunitionKey;
use nebulous_data::prelude::*;

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
    self.ships.sort_unstable_by_key(|ship| {
      (Reverse(ship.hull_key()), ship.purpose.clone())
    });
    self
  }

  pub fn put_ship_component(mut self, ship: usize, component: usize, value: ComponentState) -> Self {
    let ship = &mut self.ships[ship];
    ship.hull_state.replace_ship_component(component, value);
    ship.cache.update_from(&ship.hull_state);
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
  purpose: PurposeName,
  point_budget: usize,
  hull_state: HullState,
  cache: ShipStateCache
}

impl ShipState {
  pub fn new(hull_key: HullKey, purpose: PurposeName, point_budget: usize) -> Self {
    ShipState {
      purpose, point_budget,
      hull_state: HullState::new(hull_key),
      cache: ShipStateCache::default()
    }
  }

  pub const fn hull_key(&self) -> HullKey {
    self.hull_state.hull_key
  }

  pub const fn components(&self) -> &[Option<ComponentState>] {
    &self.hull_state.components
  }

  pub const fn purpose(&self) -> &PurposeName {
    &self.purpose
  }

  pub const fn point_budget(&self) -> usize {
    self.point_budget
  }

  pub fn similar(&self, other: &Self) -> bool {
    self.hull_key() == other.hull_key() &&
    self.purpose() == other.purpose()
  }

  pub fn iter_component_states(&self) -> impl Iterator<Item = &ComponentState> + DoubleEndedIterator + Clone {
    self.hull_state.iter_component_states()
  }

  pub fn iter_vacant_components(&self) -> impl Iterator<Item = usize> + DoubleEndedIterator + Clone + '_ {
    self.hull_state.iter_vacant_components()
  }
}

impl fmt::Display for ShipState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} ({:?}, {} components)",
      self.hull_state.hull_key.hull().name,
      self.purpose,
      self.hull_state.components.len()
    )
  }
}

#[derive(Debug, Clone)]
struct HullState {
  hull_key: HullKey,
  components: Box<[Option<ComponentState>]>
}

impl HullState {
  fn new(hull_key: HullKey) -> Self {
    let sockets = hull_key.hull().sockets.len();
    let components = (0..sockets).map(|_| None).collect();
    HullState { hull_key, components }
  }

  fn replace_ship_component(&mut self, component: usize, value: ComponentState) -> Option<ComponentState> {
    self.components[component].replace(value)
  }

  fn iter_vacant_components(&self) -> impl Iterator<Item = usize> + DoubleEndedIterator + Clone + '_ {
    self.components.iter().enumerate()
      .filter_map(|(i, c)| c.is_none().then_some(i))
  }

  fn iter_sockets_maybe(&self) -> impl Iterator<Item = (&Option<ComponentState>, &'static HullSocket)> + DoubleEndedIterator + Clone {
    self.components.iter().zip(self.hull_key.hull().sockets)
  }

  fn iter_sockets(&self) -> impl Iterator<Item = (&ComponentState, &'static HullSocket)> + DoubleEndedIterator + Clone {
    self.iter_sockets_maybe().flat_map(|(component_state, hull_socket)| {
      component_state.as_ref().map(|component_state| (component_state, hull_socket))
    })
  }

  fn iter_component_states(&self) -> impl Iterator<Item = &ComponentState> + DoubleEndedIterator + Clone {
    self.components.iter().flatten()
  }

  fn iter_component_buffs(&self) -> impl Iterator<Item = Buff> + DoubleEndedIterator + Clone + '_ {
    self.iter_component_states().flat_map(|component_state| {
      component_state.component_key.component().buffs.iter().copied()
    })
  }

  fn iter_hull_buffs(&self) -> impl Iterator<Item = Buff> + DoubleEndedIterator + Clone {
    self.hull_key.hull().buffs.iter().copied()
  }

  fn iter_buffs(&self) -> impl Iterator<Item = Buff> + DoubleEndedIterator + Clone + '_ {
    self.iter_hull_buffs().chain(self.iter_component_buffs())
  }
}

#[derive(Debug, Clone, Default)]
struct ShipStateCache {
  buffs: Box<Buffs>,
  component_quantities: BTreeMap<ComponentKey, usize>,
  crew_complement: usize,
  crew_assigned: usize,
  power_production: f32,
  power_consumption: f32
}

impl ShipStateCache {
  fn crew(&self) -> isize {
    self.crew_complement as isize - self.crew_assigned as isize
  }

  fn power(&self) -> f32 {
    self.power_production * (self.buffs.powerplant_efficiency + 1.0) - self.power_consumption
  }

  fn update_from(&mut self, hull_state: &HullState) {
    let crew_complement = hull_state.hull_key.hull().base_crew_complement;

    self.crew_complement = crew_complement;
    self.crew_assigned = 0;
    self.power_production = 0.0;
    self.power_consumption = 0.0;

    self.buffs = Box::new(hull_state.iter_buffs().collect());

    self.component_quantities.clear();
    for (component_state, hull_socket) in hull_state.iter_sockets() {
      let component = component_state.component_key.component();
      *self.component_quantities.entry(component_state.component_key).or_default() += 1;

      match component.crew(hull_socket.size) {
        crew if crew > 0 => self.crew_complement += crew as usize,
        crew if crew < 0 => self.crew_assigned += -crew as usize,
        _ => ()
      };

      match component.power {
        power if power > 0 => self.power_production += power as f32,
        power if power < 0 => self.power_consumption += -power as f32,
        _ => ()
      };
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
