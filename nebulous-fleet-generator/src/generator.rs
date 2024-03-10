use float_ord::FloatOrd;
use nebulous_data::data::Faction;
use nebulous_data::data::hulls::HullKey;
use nebulous_data::utils::lerp;
use rand::Rng;
use rand::seq::IteratorRandom;

use crate::parameters::*;
use crate::state::{FleetState, ShipState};
use crate::utils::Random;

use std::cmp::Reverse;
use std::collections::HashSet;



fn select_purpose<'p>(rng: &mut impl Rng, parameters: &'p Parameters, hull_key: HullKey) -> &'p PurposeName {
  let hull_purposes = parameters.hulls[hull_key].purposes.as_ref();
  parameters.fleet_planning.purposes.keys()
    .filter(|&purpose| hull_purposes.map_or(true, |set| set.contains(purpose)))
    .choose(rng).expect("no purposes defined")
}

pub type GeneratorStateLayer = Vec<FleetState>;
pub type GeneratorStateLayers = Vec<GeneratorStateLayer>;

pub fn get_fleet_score(parameters: &Parameters, fleet: &FleetState) -> f32 {
  Stage0::get_fleet_score(parameters, fleet)
}

pub fn run(rng: &mut Random, parameters: &Parameters, faction: Faction) -> GeneratorStateLayer {
  let layer = vec![FleetState::new(faction)];
  let layer = run_stage::<Stage0>(rng, parameters, layer);
  layer
}

fn run_stage<S: Stage>(rng: &mut Random, parameters: &Parameters, layer: GeneratorStateLayer) -> GeneratorStateLayer {
  let parameters_stage = &parameters.stages[S::INDEX];
  let mut layers = vec![layer];
  for _ in 0..parameters_stage.steps {
    let produced_fleets = step_stage::<S>(rng, parameters, &layers);
    if produced_fleets.is_empty() { break };
    layers.last_mut().expect("no fleets")
      .truncate(parameters_stage.passthrough_trunks);
    layers.push(produced_fleets);
  };

  layers.pop().expect("no fleets")
}

fn step_stage<S: Stage>(rng: &mut Random, parameters: &Parameters, layers: &GeneratorStateLayers) -> GeneratorStateLayer {
  let parameters_stage = &parameters.stages[S::INDEX];
  let mut layer = layers.iter().flatten()
    .flat_map(|fleet| S::operation(&mut Random::derive(rng), parameters, fleet))
    .filter(|fleet| S::is_fleet_legal(parameters, fleet))
    .map(FleetState::make_sorted)
    .chain(layers.iter().flatten().cloned())
    .collect::<GeneratorStateLayer>();
  layer.sort_by_cached_key(|fleet| Reverse(FloatOrd(S::get_fleet_score(parameters, fleet))));
  layer.dedup_by(|a, b| FleetState::similar(&a, &b));
  layer.truncate(parameters_stage.max_trunks);
  layer
}

trait Stage {
  const INDEX: usize;

  fn operation(rng: &mut impl Rng, parameters: &Parameters, fleet: &FleetState) -> Vec<FleetState>;
  fn get_fleet_score(parameters: &Parameters, fleet: &FleetState) -> f32;
  fn is_fleet_legal(parameters: &Parameters, fleet: &FleetState) -> bool;
}

#[derive(Debug, Clone, Copy, Default)]
struct Stage0;

impl Stage for Stage0 {
  const INDEX: usize = 0;

  fn operation(rng: &mut impl Rng, parameters: &Parameters, fleet: &FleetState) -> Vec<FleetState> {
    if fleet.ships().len() > parameters.fleet_planning.hull_limit { return Vec::new() };
    let parameters_stage = &parameters.stages[Self::INDEX];
    let hull_keys = crate::state::iter_faction_ships(fleet.faction())
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

        let ship = ShipState::new(hull_key, purpose.clone(), points_budget);
        fleet.clone().append_ship(ship)
      })
      .collect()
  }

  fn get_fleet_score(parameters: &Parameters, fleet: &FleetState) -> f32 {
    let mut score: f32 = 0.0;
    let mut general_hulls_count: usize = 0;
    let mut utility_hulls_count: usize = 0;
    let mut general_hulls_present = HashSet::new();
    let mut utility_hulls_present = HashSet::new();
    for ship in fleet.ships() {
      let purpose = &parameters.fleet_planning.purposes[ship.purpose()];
      score += parameters.hulls[ship.hull_key()].score_weight;
      score += purpose.score;
      match purpose.meta {
        PurposeMeta::General => {
          general_hulls_count += 1;
          general_hulls_present.insert(ship.purpose().clone());
        },
        PurposeMeta::Utility => {
          utility_hulls_count += 1;
          utility_hulls_present.insert(ship.purpose().clone());
        }
      };
    };

    score /= fleet.ships().len() as f32;

    let unbudgeted_points = parameters.point_limit_target.saturating_sub(fleet.point_budget())
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

  fn is_fleet_legal(parameters: &Parameters, fleet: &FleetState) -> bool {
    fleet.point_budget() <= parameters.point_limit_target
  }
}
