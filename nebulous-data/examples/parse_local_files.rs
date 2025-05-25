use nebulous_data::format::{Fleet, Ship, MissileTemplate, FormatError, Root};
use nebulous_data::xml::{DeserializeNodes, DeserializeElement, read_nodes};
use walkdir::WalkDir;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[cfg(not(feature = "steam-utils"))]
compile_error!("feature `steam-utils` must be enabled for this example");

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// If a copy of NEBULOUS: Fleet Command can be located (through steam) on your machine,
/// this will attempt to parse all `.fleet` files, `.ship` files, and `.missile` files
/// inside your `Nebulous/Saves` folder, reporting any parser errors encountered.
fn main() {
  let nebulous_dir = nebulous_data::get_local_nebulous_dir()
    .expect("failed to locate nebulous on this system");
  let saves_dir = nebulous_dir.join("Saves");

  for result in WalkDir::new(&saves_dir) {
    let entry = result.expect("failed to traverse saves dir");
    if !entry.file_type().is_file() { continue };
    let path = entry.into_path();

    let result = match path.extension() {
      Some(ext) if ext.eq_ignore_ascii_case("fleet") => {
        parse_file::<Fleet>(&path).map(visit_fleet_file)
      },
      Some(ext) if ext.eq_ignore_ascii_case("ship") => {
        parse_file::<Ship>(&path).map(visit_ship_file)
      },
      Some(ext) if ext.eq_ignore_ascii_case("missile") => {
        parse_file::<MissileTemplate>(&path).map(visit_missile_file)
      },
      Some(..) | None => continue
    };

    let path_stripped = path.strip_prefix(&nebulous_dir).unwrap_or(&path);
    match result {
      Ok(()) => println!("successfully parsed file {}", path_stripped.display()),
      Err(err) => println!("failed to parse file {}: {}", path_stripped.display(), err)
    };
  };
}

fn visit_fleet_file(fleet: Fleet) {
  for mut ship in fleet.ships {
    ship.missile_types.extend(fleet.missile_types.iter().cloned());
    visit_ship_file(ship);
  };
}

fn visit_ship_file(ship: Ship) {
  let costs = ship.calculate_costs(&ship.missile_types);
  if costs.total() != ship.cost {
    println!("cost of ship {:?}: {} (included), {} {:?} (calculated)", ship.name, ship.cost, costs.total(), costs);
  };
}

fn visit_missile_file(missile: MissileTemplate) {
  let cost = missile.calculate_cost();
  if cost != missile.cost {
    let name = missile.display_missile_key().to_string();
    println!("cost of missile template {:?}: {} (included), {} (calculated)", name, missile.cost, cost);
  };
}

fn parse_file<T>(path: &Path) -> Result<T, Error>
where T: DeserializeElement<Error = FormatError> {
  let reader = BufReader::new(File::open(path)?);
  let nodes = read_nodes(reader)?;
  let value = <Root<T>>::deserialize_nodes(nodes)?;
  Ok(value.element)
}
