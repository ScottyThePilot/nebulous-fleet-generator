extern crate float_ord;
extern crate nebulous_data;
extern crate rand;
extern crate rand_xoshiro;
extern crate serde;
extern crate singlefile;
extern crate singlefile_formats;

pub mod generator;
pub mod parameters;
mod utils;

use nebulous_data::data::Faction;
use singlefile::container::ContainerReadonly;
use singlefile_formats::toml_serde::{PrettyToml, Toml};

use crate::generator::{GeneratorFleet, Stage, Stage0, run_stage};
use crate::parameters::Parameters;

type ParametersWrapper = ContainerReadonly<Parameters, PrettyToml>;

fn main() {
  let parameters = ParametersWrapper::open("parameters.toml", Toml)
    .expect("failed to open parameters").into_value();

  let mut rng = crate::utils::new_rng();
  let state = vec![GeneratorFleet::new(Faction::Protectorate)];
  let state = run_stage::<Stage0>(&mut rng, &parameters, state);

  for fleet in &state {
    let score = Stage0::get_fleet_score(&parameters, &fleet);
    println!("{} ({}, {score:.4} score)", fleet.faction.name(), fleet.total_point_budget());
    for ship in &fleet.ships {
      println!("- {ship}");
    };

    println!();
  };
}
