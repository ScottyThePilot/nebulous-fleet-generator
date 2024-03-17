extern crate float_ord;
extern crate nebulous_data;
extern crate rand;
extern crate rand_xoshiro;
extern crate serde;
extern crate singlefile;
extern crate singlefile_formats;

mod generator;
mod parameters;
mod state;
mod utils;

use nebulous_data::data::Faction;
use singlefile::container::ContainerReadonly;
use singlefile_formats::toml_serde::{PrettyToml, Toml};

use crate::parameters::Parameters;
use crate::utils::Random;

type ParametersWrapper = ContainerReadonly<Parameters, PrettyToml>;

fn main() {
  let parameters = ParametersWrapper::open("parameters.toml", Toml)
    .expect("failed to open parameters").into_value();

  let mut rng = Random::new();
  for fleet in crate::generator::run(&mut rng, &parameters, Faction::Alliance) {
    let score = crate::generator::get_fleet_score(&parameters, &fleet);
    println!("{} ({}, {score:.4} score)", fleet.faction().name(), fleet.point_budget());
    for ship in fleet.ships() {
      println!("- {ship}");
    };

    println!();
  };
}
