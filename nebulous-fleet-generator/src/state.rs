use crate::parameters::*;

use nebulous_data::data::{Buff, Buffs, Faction};
use nebulous_data::data::hulls::{HullKey, HullSocket};
use nebulous_data::data::components::ComponentKey;
use nebulous_data::data::munitions::{MunitionKey, MunitionFamily};
use nebulous_data::prelude::*;

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::fmt;



#[derive(Debug, Clone)]
pub struct FleetState {
  faction: Faction,
  ships: Vec<Arc<ShipState>>
}

impl FleetState {
  pub const fn new(faction: Faction) -> Self {
    FleetState { faction, ships: Vec::new() }
  }

  pub const fn faction(&self) -> Faction {
    self.faction
  }

  pub fn ships(&self) -> &[Arc<ShipState>] {
    &self.ships
  }

  pub fn point_budget(&self) -> usize {
    self.ships.iter().map(|ship| ship.point_budget).sum()
  }

  pub fn append_ship(mut self, ship: ShipState) -> Self {
    self.ships.push(Arc::new(ship));
    self.ships.sort_unstable_by_key(|ship| {
      (Reverse(ship.hull_key()), ship.purpose.clone())
    });
    self
  }

  pub fn put_ship_component(mut self, ship: usize, component: usize, value: ComponentState) -> Self {
    let ship = Arc::make_mut(&mut self.ships[ship]);
    ship.inner.replace_ship_component(component, value);
    ship.cache.update_from(&ship.inner);
    self
  }

  pub fn iter_ship_vacant_components(&self) -> impl Iterator<Item = (usize, usize)> + DoubleEndedIterator + Clone + '_ {
    self.ships.iter().enumerate().flat_map(|(i, ship)| {
      ship.iter_vacant_components().map(move |j| (i, j))
    })
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
  inner: ShipStateInner,
  cache: ShipStateCache
}

impl ShipState {
  pub fn new(hull_key: HullKey, purpose: PurposeName, point_budget: usize) -> Self {
    ShipState {
      purpose, point_budget,
      inner: ShipStateInner::new(hull_key),
      cache: ShipStateCache::default()
    }
  }

  pub const fn hull_key(&self) -> HullKey {
    self.inner.hull_key
  }

  pub const fn components(&self) -> &[Option<ComponentState>] {
    &self.inner.components
  }

  pub const fn purpose(&self) -> &PurposeName {
    &self.purpose
  }

  pub const fn point_budget(&self) -> usize {
    self.point_budget
  }

  pub fn crew(&self) -> isize {
    self.cache.crew()
  }

  pub fn power(&self) -> f32 {
    self.cache.power()
  }

  pub fn similar(&self, other: &Self) -> bool {
    self.hull_key() == other.hull_key() &&
    self.purpose() == other.purpose()
  }

  pub fn iter_component_states(&self) -> impl Iterator<Item = &ComponentState> + DoubleEndedIterator + Clone {
    self.inner.iter_component_states()
  }

  pub fn iter_vacant_components(&self) -> impl Iterator<Item = usize> + DoubleEndedIterator + Clone + '_ {
    self.inner.iter_vacant_components()
  }
}

impl fmt::Display for ShipState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} ({:?}, {} components)",
      self.inner.hull_key.hull().name,
      self.purpose,
      self.inner.components.len()
    )
  }
}

#[derive(Debug, Clone)]
struct ShipStateInner {
  hull_key: HullKey,
  components: Box<[Option<ComponentState>]>
}

impl ShipStateInner {
  fn new(hull_key: HullKey) -> Self {
    let sockets = hull_key.hull().sockets.len();
    let components = (0..sockets).map(|_| None).collect();
    ShipStateInner { hull_key, components }
  }

  fn replace_ship_component(&mut self, component: usize, value: ComponentState) -> Option<ComponentState> {
    self.components[component].replace(value)
  }

  fn iter_vacant_components(&self) -> impl Iterator<Item = usize> + DoubleEndedIterator + Clone + '_ {
    self.components.iter().enumerate().filter_map(|(i, c)| c.is_none().then_some(i))
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
  components: BTreeMap<ComponentKey, ComponentStateCache>,
  munition_families: HashMap<MunitionFamily, Option<f32>>,
  crew_complement: usize,
  crew_assigned: usize,
  power_production: f32,
  power_consumption: f32
}

impl ShipStateCache {
  /// Net crew complement, i.e. this will be positive when there are sufficient crew,
  /// and negative when there are insufficient crew.
  fn crew(&self) -> isize {
    self.crew_complement as isize - self.crew_assigned as isize
  }

  /// Net power production, i.e. this will be positive when there is sufficient power,
  /// and negative when there is insufficient power.
  fn power(&self) -> f32 {
    self.power_production * (self.buffs.powerplant_efficiency + 1.0) - self.power_consumption
  }

  fn clear(&mut self) {
    //self.buffs = Box::new(Buffs::default());
    self.components.clear();
    self.munition_families.clear();
    self.crew_complement = 0;
    self.crew_assigned = 0;
    self.power_production = 0.0;
    self.power_consumption = 0.0;
  }

  fn update_from(&mut self, inner: &ShipStateInner) {
    self.clear();

    self.buffs = Box::new(inner.iter_buffs().collect());

    self.crew_complement += inner.hull_key.hull().base_crew_complement;

    for (component_state, hull_socket) in inner.iter_sockets() {
      let component = component_state.component_key.component();

      let component_entry = self.components.entry(component_state.component_key).or_default();
      component_entry.quantity += 1;

      if let Some(munition_family) = component.munition_family() {
        let entry = self.munition_families.entry(munition_family).or_insert(None);
        if let Some(fire_rate) = component.fire_rate(&self.buffs) {
          *entry.get_or_insert(0.0) += fire_rate;
        };
      };

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

#[derive(Debug, Clone, Default)]
struct ComponentStateCache {
  quantity: usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentState {
  component_key: ComponentKey,
  magazine_contents: Option<BTreeMap<MunitionState, usize>>
}

impl ComponentState {
  pub const fn component_key(&self) -> ComponentKey {
    self.component_key
  }

  pub fn cost(&self) -> usize {
    self.iter_magazine_contents()
      .map(|(munition_state, quantity)| munition_state.cost(quantity))
      .chain(std::iter::once(self.component_key.component().point_cost))
      .sum::<usize>()
  }

  pub const fn magazine_contents(&self) -> Option<&BTreeMap<MunitionState, usize>> {
    self.magazine_contents.as_ref()
  }

  pub fn iter_magazine_contents(&self) -> impl Iterator<Item = (&MunitionState, usize)> {
    self.magazine_contents.iter().flatten()
      .map(|(munition_state, &quantity)| (munition_state, quantity))
  }

  pub fn get_munition_quantity(&mut self, munition: MunitionState) -> &mut usize {
    self.magazine_contents
      .get_or_insert_with(BTreeMap::new)
      .entry(munition).or_default()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MunitionState {
  Munition(MunitionKey)
}

impl MunitionState {
  pub fn cost(&self, quantity: usize) -> usize {
    todo!()
  }
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

fn merge_opt<T: Copy>(lhs: &mut Option<T>, rhs: Option<T>, f: impl FnOnce(T, T) -> T) {
  *lhs = Option::zip(*lhs, rhs).map(|(lhs, rhs)| f(lhs, rhs))
}
