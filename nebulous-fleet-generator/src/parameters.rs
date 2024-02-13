
#[derive(Debug, Clone)]
pub struct Parameters {
  pub hulls: ParametersHullsList
}

#[derive(Debug, Clone, Copy)]
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
  pub ocello_command_cruiser: ParametersHullSpecial,
  pub bulk_freighter_line_ship: ParametersHull,
  pub container_liner_line_ship: ParametersHull
}

#[derive(Debug, Clone, Copy)]
pub struct ParametersHull {
  pub score: f32
}

#[derive(Debug, Clone, Copy)]
pub struct ParametersHullSpecial {
  pub score: f32,
  pub unique_equipment_bonus: f32,
  pub unique_equipment_bonus_max: f32
}
