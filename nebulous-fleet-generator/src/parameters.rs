use nebulous_data::data::hulls::HullKey;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashSet};
use std::ops::Index;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameters {
  pub point_limit_target: usize,
  pub stages: [ParametersStage; 1],
  pub fleet_planning: ParametersFleetPlanning,
  pub hull_planning: ParametersHullPlanning,
  pub hulls: ParametersHullsList,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersStage {
  pub steps: usize,
  pub max_trunks: usize,
  pub max_branches: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersFleetPlanning {
  pub hull_limit: usize,
  /// How much to reward/penalize the generator for each general purpose hull added beyond the first hull.
  pub excessive_general_hull_score: f32,
  /// How much to reward/penalize the generator for each utility purpose hull added beyond the first hull.
  pub excessive_utility_hull_score: f32,
  /// How much to reward/penalize the generator for the variation
  /// in purposes it has selected among the general purpose hulls in the fleet.
  pub general_hull_variance_score: f32,
  /// How much to reward/penalize the generator for the variation
  /// in purposes it has selected among the utility purpose hulls in the fleet.
  pub utility_hull_variance_score: f32,
  /// More than this many points must be unbudgeted for hulls in order to apply the unbudgeted points score.
  pub unbudgeted_points_threshold: usize,
  /// If not all of the point limit is budgeted for hulls, this reward/penalty
  /// will be applied, multiplied by the number of unbudgeted points.
  pub unbudgeted_points_score: f32,
  pub hull_budget_jitter: f32,
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

impl Index<HullKey> for ParametersHullsList {
  type Output = ParametersHull;

  fn index(&self, index: HullKey) -> &Self::Output {
    match index {
      HullKey::SprinterCorvette => &self.sprinter_corvette,
      HullKey::RainesFrigate => &self.raines_frigate,
      HullKey::KeystoneDestroyer => &self.keystone_destroyer,
      HullKey::VauxhallLightCruiser => &self.vauxhall_light_cruiser,
      HullKey::AxfordHeavyCruiser => &self.axford_heavy_cruiser,
      HullKey::SolomonBattleship => &self.solomon_battleship,
      HullKey::ShuttleClipper => &self.shuttle_clipper,
      HullKey::TugboatClipper => &self.tugboat_clipper,
      HullKey::CargoFeederMonitor => &self.cargo_feeder_monitor,
      HullKey::OcelloCommandCruiser => &self.ocello_command_cruiser,
      HullKey::BulkFreighterLineShip => &self.bulk_freighter_line_ship,
      HullKey::ContainerLinerLineShip => &self.container_liner_line_ship
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersHull {
  pub score: f32,
  //pub opponent_armor_thickness_min: f32,
  //pub opponent_armor_thickness_max: f32,
  pub points_budget_min: usize,
  pub points_budget_max: usize,
  /// A list of purposes (fleet roles) that this hull is allowed to fill,
  /// and their weights (likelyhood of being chosen). All purposes are allowed if left empty.
  #[serde(default, alias = "allowed_purposes")]
  pub purposes: Option<HashSet<PurposeName>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParametersHullPlanning {
  /// A reward/penalty applied for each extra instance of a component for which only one is necessary.
  /// E.g. Scryer, Intelligence Center
  pub redundancy_score: f32,
  /// Encourages the generator to pad its power buffer such that if the 'weakest link' in the
  /// ship's power net (the component with the highest contribution to power output) is destroyed,
  /// the ship will still have full power.
  #[serde(default)]
  pub weakest_link_power_buffer_score: f32,
  /// An upper limit on *reactors* for what may be defined as the 'weakest link'.
  /// Anything producing more power than this will not have its destruction planned for.
  #[serde(default)]
  pub weakest_link_power_threshold: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Purpose {
  /// The meta-category of this purpose.
  pub meta: PurposeMeta,
  /// A value between 0 and 1, determining what the point budget should be.
  /// - At 0, the hull's configured minimum point budget will be used.
  /// - At 1, the hull's configured maximum point budget will be used.
  pub budget: f32,
  /// The reward/penalty given for choosing this purpose.
  pub score: f32,
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
  pub electronic_warfare: f32
}

impl Default for Purpose {
  fn default() -> Self {
    Purpose {
      meta: PurposeMeta::General,
      score: 100.0,
      short_range_combat: 1.0,
      medium_range_combat: 1.0,
      long_range_combat: 1.0,
      missile_combat: 1.0,
      point_defense: 1.0,
      survivability: 1.0,
      sensor_coverage: 1.0,
      electronic_warfare: 1.0,
      budget: 0.5
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum PurposeMeta {
  /// General-purposed (combat).
  #[serde(alias = "general")] General,
  /// Utility-purposed (non-combat).
  #[serde(alias = "utility")] Utility
}

pub type PurposeName = Box<str>;
pub type PurposeMap = BTreeMap<PurposeName, Purpose>;
