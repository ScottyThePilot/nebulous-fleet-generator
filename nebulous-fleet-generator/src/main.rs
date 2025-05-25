mod utils;
mod model;

extern crate chumsky;
extern crate clap;
extern crate indexmap;
extern crate nebulous_data;
extern crate rand;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

use crate::model::{FleetStrategy, FleetStrategySelection, ShipState, MissileState};

use anyhow::{Error, Context};
use clap::{Parser, Subcommand};
use indexmap::IndexMap;
use nebulous_data::format::{Root, Fleet, Ship, MissileTemplate};
use nebulous_data::xml::{DeserializeElement, DeserializeNodes};
use singlefile::container::ContainerWritable;
use singlefile_formats::toml_serde::Toml;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use std::path::{Path, PathBuf};
use std::io::{self, BufReader};
use std::fs::{self, File};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  root: Option<PathBuf>,
  #[command(subcommand)]
  command: Option<ArgCommand>
}

#[derive(Debug, Subcommand)]
enum ArgCommand {
  /// Creates a blank fleet strategy.
  CreateStrategy {
    /// The name that the strategy should be given.
    #[arg(short, long)]
    name: String,
    /// The weight that the strategy should be given. Defaults to 1.
    #[arg(short, long)]
    weight: Option<usize>
  },
  /// Creates a new ship state based on a local ship template save file.
  CreateShip {
    /// The name that the new ship state should be given.
    #[arg(short, long)]
    name: String,
    /// The author to assign to the new ship state.
    #[arg(short, long)]
    author: Option<String>,
    /// The path to the ship template save file.
    path: PathBuf
  },
  /// Creates multiple new ship states based on a local fleet save file.
  /// Each generated ship state will have a randomized suffix.
  CreateShips {
    /// The base name that each new ship state should be given.
    #[arg(short, long)]
    name: Option<String>,
    /// The author to assign to each new ship state.
    #[arg(short, long)]
    author: Option<String>,
    /// The path to the fleet save file.
    path: PathBuf
  },
  /// Creates a new missile state based on a local missile template save file.
  CreateMissile {
    /// The name that the missile state should be given.
    #[arg(short, long)]
    name: String,
    /// The author to assign to the new missile state.
    #[arg(short, long)]
    author: Option<String>,
    /// The path to the missile template save file.
    path: PathBuf
  }
}

fn main() {
  let Args { command, root } = Args::parse();
  let root = root.unwrap_or_else(|| PathBuf::from("./"));
  let result = match command {
    Some(ArgCommand::CreateStrategy { name, weight }) => run_create_strategy(root, name, weight),
    Some(ArgCommand::CreateShip { name, author, path }) => run_create_ship(root, name, author, path),
    Some(ArgCommand::CreateShips { name, author, path }) => run_create_ships(root, name, author, path),
    Some(ArgCommand::CreateMissile { name, author, path }) => run_create_missile(root, name, author, path),
    None => Ok(())
  };

  result.unwrap_or_else(|err| panic!("{err:#}"));
}

fn run_create_strategy(root: PathBuf, name: String, weight: Option<usize>) -> Result<(), Error> {
  let strategy = FleetStrategy {
    weight: weight.unwrap_or(1),
    selections: vec![FleetStrategySelection {
      in_formation: false,
      formation_hierarchy_level: 1,
      formation_hierarchy_limit: None,
      formation_ship_limit: None,
      weight_initial: 1,
      weight_additional: 1,
      predicates: Default::default()
    }]
  };

  fs::create_dir_all(root.join("strategies"))
    .context("failed to create strategies dir")?;

  let out_path = root.join(format!("strategies/{name}.toml"));
  WrapperToml::create_overwrite(&out_path, Toml, strategy)
    .with_context(|| format!("failed to write strategy to {}", out_path.display()))?;

  Ok(())
}

fn run_create_ship(root: PathBuf, name: String, author: Option<String>, in_path: PathBuf) -> Result<(), Error> {
  let ship = read_xml::<Ship>(&in_path)
    .context("failed to read ship template save file")?;
  let mut ship_state = ShipState::from_ship(&ship, &ship.missile_types)
    .context("failed to create ship state from ship template save file")?;
  ship_state.author = author;

  fs::create_dir_all(root.join("ships"))
    .context("failed to create ships dir")?;

  let out_path = root.join(format!("ships/{name}.toml"));
  WrapperToml::create_overwrite(&out_path, Toml, ship_state)
    .with_context(|| format!("failed to write ship state to {}", out_path.display()))?;

  Ok(())
}

fn run_create_ships(root: PathBuf, name: Option<String>, author: Option<String>, in_path: PathBuf) -> Result<(), Error> {
  let fleet = read_xml::<Fleet>(&in_path)
    .context("failed to read fleet save file")?;

  fs::create_dir_all(root.join("ships"))
    .context("failed to create ships dir")?;

  let random_key = rand::random::<u32>();
  for (i, ship) in fleet.ships.iter().enumerate() {
    let mut ship_state = ShipState::from_ship(&ship, &fleet.missile_types)
      .context("failed to create ship state from fleet save file")?;
    ship_state.author = author.clone();

    let name = match name.as_deref() {
      Some(name) => format!("{name}_{i}"),
      None => format!("ship_{random_key:08x}_{i}")
    };

    let out_path = root.join(format!("ships/{name}.toml"));
    WrapperToml::create_overwrite(&out_path, Toml, ship_state)
      .with_context(|| format!("failed to write ship state to {}", out_path.display()))?;
  };

  Ok(())
}

fn run_create_missile(root: PathBuf, name: String, author: Option<String>, in_path: PathBuf) -> Result<(), Error> {
  let missile_template = read_xml::<MissileTemplate>(&in_path)
    .context("failed to read missile template save file")?;
  let mut missile_state = MissileState::from_missile_template(&missile_template)
    .context("failed to create missile state from missile template save file")?;
  missile_state.author = author;

  fs::create_dir_all(root.join("missiles"))
    .context("failed to create missiles dir")?;

  let out_path = root.join(format!("missiles/{name}.toml"));
  WrapperToml::create_overwrite(&out_path, Toml, missile_state)
    .with_context(|| format!("failed to write missile state to {}", out_path.display()))?;

  Ok(())
}


type WrapperToml<T> = ContainerWritable<T, Toml<true>>;

pub struct State {
  pub strategies: IndexMap<PathBuf, WrapperToml<FleetStrategy>>,
  pub ships: IndexMap<PathBuf, WrapperToml<ShipState>>,
  pub missiles: IndexMap<PathBuf, WrapperToml<MissileState>>,
  pub path: PathBuf
}

impl State {
  pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
    let path = path.into();
    Ok(State {
      strategies: read_dir_toml_files(path.join("strategies"))?,
      ships: read_dir_toml_files(path.join("ships"))?,
      missiles: read_dir_toml_files(path.join("missiles"))?,
      path
    })
  }

  pub fn create_strategy(&mut self, name: &str, strategy: impl FnOnce() -> FleetStrategy) -> Result<(), Error> {
    let entry_path = self.path("strategies", name);
    let container = WrapperToml::create_or_else(&entry_path, Toml, strategy)
      .with_context(|| format!("failed to create file {}", entry_path.display()))?;
    self.strategies.insert(entry_path, container);
    Ok(())
  }

  pub fn create_ship(&mut self, name: &str, strategy: impl FnOnce() -> ShipState) -> Result<(), Error> {
    let entry_path = self.path("ships", name);
    let container = WrapperToml::create_or_else(&entry_path, Toml, strategy)
      .with_context(|| format!("failed to create file {}", entry_path.display()))?;
    self.ships.insert(entry_path, container);
    Ok(())
  }

  pub fn create_missile(&mut self, name: &str, strategy: impl FnOnce() -> MissileState) -> Result<(), Error> {
    let entry_path = self.path("missiles", name);
    let container = WrapperToml::create_or_else(&entry_path, Toml, strategy)
      .with_context(|| format!("failed to create file {}", entry_path.display()))?;
    self.missiles.insert(entry_path, container);
    Ok(())
  }

  fn path(&self, folder: &str, name: &str) -> PathBuf {
    let mut entry_path = self.path.clone();
    entry_path.push(folder);
    entry_path.push(name);
    entry_path
  }
}

fn read_dir_toml_files<T>(path: impl AsRef<Path>) -> Result<IndexMap<PathBuf, WrapperToml<T>>, Error>
where T: DeserializeOwned + Serialize {
  let path = path.as_ref();
  let mut files = IndexMap::new();
  for result in fs::read_dir(path).with_context(|| format!("failed to read dir {}", path.display()))? {
    let entry = result.with_context(|| format!("failed to read dir {}", path.display()))?;
    if entry.file_type().is_ok_and(|file_type| file_type.is_file()) {
      let entry_path = entry.path();
      let container = WrapperToml::open(&entry_path, Toml)
        .with_context(|| format!("failed to read file {}", entry_path.display()))?;
      files.insert(entry_path, container);
    };
  };

  Ok(files)
}

fn read_xml<T>(path: impl AsRef<Path>) -> Result<T, Error>
where T: DeserializeElement<Error = nebulous_data::format::FormatError> {
  let path = path.as_ref();
  let file = File::open(&path)
    .with_context(|| format!("failed to open file {}", path.display()))?;
  let nodes = nebulous_data::xml::read_nodes(BufReader::new(file))
    .with_context(|| format!("failed to read xml from file {}", path.display()))?;
  let value = <Root<T>>::deserialize_nodes(nodes)
    .with_context(|| format!("failed to deserialize xml from file {}", path.display()))?;
  Ok(value.element)
}
