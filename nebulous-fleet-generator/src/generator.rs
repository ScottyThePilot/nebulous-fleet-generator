use nebulous_data::data::Faction;
use nebulous_data::data::hulls::{HullKey, HullSocket};
use nebulous_data::data::components::{ComponentKey, ComponentKind};
use nebulous_data::data::munitions::{MunitionKey, MunitionFamily};
use nebulous_data::format::{Fleet as RawFleet, Ship as RawShip};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro128StarStar;

use std::collections::BTreeMap;
use std::sync::Arc;

pub type Random = Xoroshiro128StarStar;



#[derive(Debug, Clone)]
pub struct Generator {
  rng: Random,
  fleet: GeneratorFleet,
  parameters: Arc<crate::parameters::Parameters>
}

impl Generator {
  pub fn operation_stage1(&mut self) {

  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorFleet {
  pub faction: Faction,
  pub ships: Vec<GeneratorShip>
}

impl GeneratorFleet {
  pub fn from_raw_fleet(raw_fleet: RawFleet) -> Option<Self> {
    let faction = raw_fleet.faction_key;
    let ships = raw_fleet.ships.into_iter()
      .map(GeneratorShip::from_raw_ship)
      .collect::<Option<Vec<GeneratorShip>>>()?;
    Some(GeneratorFleet { faction, ships })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorShip {
  pub key: HullKey,
  pub components: Box<[Option<GeneratorComponent>]>,
  pub component_quantities: BTreeMap<ComponentKey, usize>
}

impl GeneratorShip {
  pub fn new(key: HullKey) -> Self {
    let sockets = key.hull().sockets.len();
    let components = (0..sockets).map(|_| None).collect();
    GeneratorShip { key, components, component_quantities: BTreeMap::new() }
  }

  pub fn from_raw_ship(raw_ship: RawShip) -> Option<Self> {
    let sockets = raw_ship.hull_type.hull().sockets;
    let mut ship = Self::new(raw_ship.hull_type);
    for raw_socket in raw_ship.socket_map {
      let i = sockets.iter().position(|s| s.save_key == raw_socket.key.as_ref())?;
      let key = raw_socket.component_name;
      let magazine_contents = raw_socket.component_data
        .map_or_else(Vec::new, |d| d.into_load()).into_iter()
        .map(|mag| Some((GeneratorMunition::from_key(mag.magazine_key)?, mag.quantity)))
        .collect::<Option<BTreeMap<GeneratorMunition, usize>>>()?;
      ship.components[i] = Some(GeneratorComponent { key, magazine_contents });
    };

    Some(ship)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorComponent {
  pub key: ComponentKey,
  pub magazine_contents: BTreeMap<GeneratorMunition, usize>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneratorMunition {
  Munition(MunitionKey)
}

impl GeneratorMunition {
  pub fn from_key(key: Box<str>) -> Option<Self> {
    key.parse::<MunitionKey>().ok().map(GeneratorMunition::Munition)
  }
}

/// Iterates through hulls available to the given faction.
fn iter_faction_ships(faction: Faction) -> impl Iterator<Item = HullKey> + DoubleEndedIterator + Clone {
  HullKey::VALUES.into_iter().copied()
    .filter(move |&hull_key| hull_key.hull().faction == faction)
}

/// Iterates through components available to the given hull socket.
fn iter_socket_components(hull_key: HullKey, socket: usize) -> impl Iterator<Item = ComponentKey> + DoubleEndedIterator + Clone {
  let hull = hull_key.hull();
  let socket = &hull.sockets[socket];
  ComponentKey::VALUES.into_iter().copied().filter(move |&component_key| {
    let component = component_key.component();
    component.is_usable_on(hull_key) && component.can_fit_in(socket.size)
  })
}
