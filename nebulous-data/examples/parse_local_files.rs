use nebulous_data::format::{Fleet, Ship, MissileTemplate, FormatError, Root};
use nebulous_data::xml::{DeserializeNodes, DeserializeElement, read_nodes};
use steamlocate::SteamDir;
use walkdir::WalkDir;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;



const NEBULOUS_STEAM_APPID: u32 = 887570;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// If a copy of NEBULOUS: Fleet Command can be located (through steam) on your machine,
/// this will attempt to parse all `.fleet` files, `.ship` files, and `.missile` files
/// inside your `Nebulous/Saves` folder, reporting any parser errors encountered.
fn main() {
  let mut steam_dir = SteamDir::locate()
    .expect("failed to locate steam on this system");
  let nebulous = steam_dir.app(&NEBULOUS_STEAM_APPID)
    .expect("failed to locate nebulous on this system");
  println!("found nebulous ({NEBULOUS_STEAM_APPID}) at {}", nebulous.path.display());
  let saves_dir = nebulous.path.join("Saves");

  for result in WalkDir::new(&saves_dir) {
    let entry = result.expect("failed to traverse saves dir");
    if !entry.file_type().is_file() { continue };
    let path = entry.into_path();

    let result = match path.extension() {
      Some(ext) if ext.eq_ignore_ascii_case("fleet") => parse_file::<Fleet>(&path),
      Some(ext) if ext.eq_ignore_ascii_case("ship") => parse_file::<Ship>(&path),
      Some(ext) if ext.eq_ignore_ascii_case("missile") => parse_file::<MissileTemplate>(&path),
      Some(..) | None => continue
    };

    let path_stripped = path.strip_prefix(&nebulous.path).unwrap_or(&path);
    match result {
      Ok(()) => println!("successfully parsed file {}", path_stripped.display()),
      Err(err) => println!("failed to parse file {}: {}", path_stripped.display(), err)
    };
  };
}

fn parse_file<T>(path: &Path) -> Result<(), Error>
where T: DeserializeElement<Error = FormatError> {
  let reader = BufReader::new(File::open(path)?);
  let nodes = read_nodes(reader)?;
  <Root<T>>::deserialize_nodes(nodes)?;

  Ok(())
}
