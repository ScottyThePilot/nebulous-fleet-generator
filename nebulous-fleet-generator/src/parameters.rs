//! # Terminology
//! - Score: Points (can be positive or negative) given to the generator based on the fleet is has generated.
//! - Reward: A positive score given.
//! - Penalty: A negative score given.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameters {
  pub point_limit_target: Option<usize>,
  pub fleet_planning: ParametersFleetPlanning,
  pub hull_planning: ParametersHullPlanning,
  pub hulls: ParametersHullsList,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersFleetPlanning {
  /// How much to reward/penalize the generator for each general purpose hull added beyond the first hull.
  pub excessive_general_hull_score: f32,
  /// How much to reward/penalize the generator for each utility purpose hull added beyond the first hull.
  pub excessive_utility_hull_score: f32,
  /// A configurable list of purposes (AKA fleet roles).
  /// Each hull is assigned a role when it is created, and the priorities of
  /// that role will affect the decisions the generator makes with respect to it.
  pub purposes: PurposeMap,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersHullsList {
  pub sprinter_corvette: ParametersHull,
  pub raines_frigate: ParametersHull,
  pub keystone_destroyer: ParametersHull,
  pub vauxhall_light_cruiser: ParametersHull,
  pub axford_heavy_cruiser: ParametersHull,
  pub solomon_battleship: ParametersHull,
  pub shuttle_clipper: ParametersHull,
  pub tugboat_clipper: ParametersHull,
  pub cargo_feeder_monitor: ParametersHull,
  pub ocello_command_cruiser: ParametersHull,
  pub bulk_freighter_line_ship: ParametersHull,
  pub container_liner_line_ship: ParametersHull
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersHull {
  pub score: f32,
  pub opponent_armor_thickness_min: f32,
  pub opponent_armor_thickness_max: f32,
  pub points_budget_min: usize,
  pub points_budget_max: usize,
  /// A list of purposes (fleet roles) that this hull is allowed to fill,
  /// and their weights (likelyhood of being chosen). All purposes are allowed if left empty.
  pub allowed_purposes: Option<HashMap<PurposeName, f32>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersHullPlanning {
  /// Encourages the generator to pad its power buffer such that if the 'weakest link' in the
  /// ship's power net (the component with the highest contribution to power output) is destroyed,
  /// the ship will still have full power.
  pub weakest_link_power_buffer_score: f32,
  /// An upper limit on *reactors* for what may be defined as the 'weakest link'.
  /// Anything producing more power than this will not have its destruction planned for.
  pub weakest_link_power_threshold: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ParametersPriorities {
  /// The meta-category of this priorities list.
  pub meta: ParametersPriorityMeta,
  /// The level of importance assigned to weaponry suitable in short-range combat (e.g. beam weapons).
  pub short_range_combat: f32,
  /// The level of importance assigned to weaponry suitable in medium-range combat (e.g. chemical weapons).
  pub medium_range_combat: f32,
  /// The level of importance assigned to weaponry suitable in long-range combat (e.g. railguns).
  pub long_range_combat: f32,
  /// The level of importance assigned to weaponry capable of offensive missile combat.
  pub missile_combat: f32,
  /// The level of importance assigned to equipment capable of defending against enemy missiles.
  pub point_defense: f32,
  /// The level of importance assigned to equipment improving survivability and repairability of the vessel.
  /// The generator will not attempt to use modules as ablative armor.
  pub survivability: f32,
  /// The level of importance assigned to equipment improving sensor coverage, resilience, and range.
  pub sensor_coverage: f32,
  /// The level of importance assigned to electronic warfare equipment.
  pub electronic_warfare: f32,
  /// Multiplies the base budget estimate of the hull by this amount.
  pub budget_modifier: f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum ParametersPriorityMeta {
  /// General-purpose (combat).
  General,
  /// Utility (non-combat).
  ///
  /// Fleets may be penalized for using too many utility ships.
  Utility
}

pub type PurposeName = Box<str>;
pub type PurposeMap = HashMap<PurposeName, ParametersPriorities>;
