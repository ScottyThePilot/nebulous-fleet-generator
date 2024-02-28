use float_ord::FloatOrd;
use nebulous_data::data::Faction;
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::components::ComponentKey;
use nebulous_data::data::munitions::MunitionKey;
use nebulous_data::utils::lerp;
use rand::Rng;
use rand::seq::IteratorRandom;

use crate::parameters::*;
use crate::utils::{Random, new_rng_derive};

use std::collections::{BTreeMap, HashSet};
use std::cmp::Reverse;
use std::fmt;



fn select_purpose<'p>(rng: &mut impl Rng, parameters: &'p Parameters, hull_key: HullKey) -> &'p PurposeName {
  let hull_purposes = parameters.hulls[hull_key].purposes.as_ref();
  parameters.fleet_planning.purposes.keys()
    .filter(|&purpose| hull_purposes.map_or(true, |set| set.contains(purpose)))
    .choose(rng).expect("no purposes defined")
}

pub type GeneratorState = Vec<GeneratorFleet>;

pub fn run_stage<S: Stage>(rng: &mut Random, parameters: &Parameters, mut state: GeneratorState) -> GeneratorState {
  let parameters_stage = &parameters.stages[S::INDEX];
  for _ in 0..parameters_stage.steps {
    let produced_state = step_stage::<S>(rng, parameters, &state);
    if produced_state.is_empty() { break } else { state = produced_state };
  };

  state
}

pub fn step_stage<S: Stage>(rng: &mut Random, parameters: &Parameters, state: &GeneratorState) -> GeneratorState {
  let parameters_stage = &parameters.stages[S::INDEX];
  let mut state = state.iter()
    .flat_map(|fleet| S::operation(&mut new_rng_derive(rng), parameters, fleet))
    .filter(|fleet| S::is_fleet_legal(parameters, fleet))
    .chain(state.iter().cloned())
    .collect::<GeneratorState>();
  state.iter_mut().for_each(GeneratorFleet::sort);
  state.sort_by_cached_key(|fleet| Reverse(FloatOrd(S::get_fleet_score(parameters, fleet))));
  state.dedup();
  state.truncate(parameters_stage.max_trunks);
  state
}

pub struct Stage0;

impl Stage for Stage0 {
  const INDEX: usize = 0;

  fn operation(rng: &mut impl Rng, parameters: &Parameters, fleet: &GeneratorFleet) -> Vec<GeneratorFleet> {
    if fleet.ships.len() > parameters.fleet_planning.hull_limit { return Vec::new() };
    let parameters_stage = &parameters.stages[Self::INDEX];
    let hull_keys = iter_faction_ships(fleet.faction)
      .take(parameters_stage.max_branches.unwrap_or(usize::MAX))
      .choose_multiple(rng, parameters_stage.max_trunks);
    hull_keys.into_iter()
      .map(|hull_key| {
        let purpose = select_purpose(rng, parameters, hull_key);
        let priorities = &parameters.fleet_planning.purposes[purpose];
        let parameters_hull = &parameters.hulls[hull_key];
        let j = parameters.fleet_planning.hull_budget_jitter;

        let points_budget = lerp(
          parameters_hull.points_budget_min as f32,
          parameters_hull.points_budget_max as f32,
          (priorities.budget + rng.gen_range(-j..j)).clamp(0.0, 1.0)
        ).round() as usize;

        let ship = GeneratorShip::new(hull_key, purpose.clone(), points_budget);

        let mut fleet = fleet.clone();
        fleet.ships.push(ship);
        fleet
      })
      .collect()
  }

  fn get_fleet_score(parameters: &Parameters, fleet: &GeneratorFleet) -> f32 {
    let mut score: f32 = 0.0;
    let mut general_hulls_count: usize = 0;
    let mut utility_hulls_count: usize = 0;
    let mut general_hulls_present = HashSet::new();
    let mut utility_hulls_present = HashSet::new();
    for ship in &fleet.ships {
      let purpose = &parameters.fleet_planning.purposes[&ship.purpose];
      score += parameters.hulls[ship.hull_key].score;
      score += purpose.score;
      match purpose.meta {
        PurposeMeta::General => {
          general_hulls_count += 1;
          general_hulls_present.insert(ship.purpose.clone());
        },
        PurposeMeta::Utility => {
          utility_hulls_count += 1;
          utility_hulls_present.insert(ship.purpose.clone());
        }
      };
    };

    score /= fleet.ships.len() as f32;

    let unbudgeted_points = parameters.point_limit_target.saturating_sub(fleet.total_point_budget())
      .saturating_sub(parameters.fleet_planning.unbudgeted_points_threshold);
    score += unbudgeted_points as f32 * parameters.fleet_planning.unbudgeted_points_score;

    score += general_hulls_count.saturating_sub(1) as f32 *
      parameters.fleet_planning.excessive_general_hull_score;
    score += utility_hulls_count.saturating_sub(1) as f32 *
      parameters.fleet_planning.excessive_utility_hull_score;

    score += general_hulls_present.len().saturating_sub(1) as f32 *
      parameters.fleet_planning.general_hull_variance_score;
    score += utility_hulls_present.len().saturating_sub(1) as f32 *
      parameters.fleet_planning.utility_hull_variance_score;

    score
  }

  fn is_fleet_legal(parameters: &Parameters, fleet: &GeneratorFleet) -> bool {
    fleet.total_point_budget() <= parameters.point_limit_target
  }
}

pub trait Stage {
  const INDEX: usize;

  fn operation(rng: &mut impl Rng, parameters: &Parameters, fleet: &GeneratorFleet) -> Vec<GeneratorFleet>;
  fn get_fleet_score(parameters: &Parameters, fleet: &GeneratorFleet) -> f32;
  fn is_fleet_legal(parameters: &Parameters, fleet: &GeneratorFleet) -> bool;
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorFleet {
  pub faction: Faction,
  pub ships: Vec<GeneratorShip>
}

impl GeneratorFleet {
  pub const fn new(faction: Faction) -> Self {
    GeneratorFleet { faction, ships: Vec::new() }
  }

  pub fn total_point_budget(&self) -> usize {
    self.ships.iter().map(|ship| ship.point_budget).sum()
  }

  fn sort(&mut self) {
    self.ships.sort_by_key(|ship| {
      (Reverse(ship.hull_key), ship.purpose.clone())
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorShip {
  pub hull_key: HullKey,
  pub purpose: PurposeName,
  pub point_budget: usize,
  pub components: Box<[Option<GeneratorComponent>]>
}

impl GeneratorShip {
  pub fn new(hull_key: HullKey, purpose: PurposeName, point_budget: usize) -> Self {
    let sockets = hull_key.hull().sockets.len();
    let components = (0..sockets).map(|_| None).collect();
    GeneratorShip { hull_key, purpose, point_budget, components }
  }
}

impl fmt::Display for GeneratorShip {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorComponent {
  pub component_key: ComponentKey,
  pub magazine_contents: BTreeMap<GeneratorMunition, usize>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneratorMunition {
  Munition(MunitionKey)
}

/// Iterates through hulls available to the given faction.
fn iter_faction_ships(faction: Faction) -> impl Iterator<Item = HullKey> + DoubleEndedIterator + Clone {
  HullKey::VALUES.into_iter().copied()
    .filter(move |&hull_key| hull_key.hull().faction == faction)
}

/// Iterates through components available to the given hull socket.
#[allow(dead_code)]
fn iter_socket_components(hull_key: HullKey, socket: usize) -> impl Iterator<Item = ComponentKey> + DoubleEndedIterator + Clone {
  let hull = hull_key.hull();
  let socket = &hull.sockets[socket];
  ComponentKey::VALUES.into_iter().copied().filter(move |&component_key| {
    let component = component_key.component();
    component.is_usable_on(hull_key) && component.can_fit_in(socket.size)
  })
}

#[allow(dead_code)]
fn choose_weighted_btree_map<'t, T, R: Rng>(rng: &mut R, map: &'t BTreeMap<T, f32>) -> Option<&'t T> {
  let total = map.values().copied().sum::<f32>();
  let t = rng.gen_range(0.0..total);

  let mut accumulator = 0.0;
  for (key, &weight) in map {
    accumulator += weight;
    if t < accumulator { return Some(key) };
  };

  None
}
