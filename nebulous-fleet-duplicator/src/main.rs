use nebulous_data::format::{Fleet, Root};
use nebulous_data::xml::{DeserializeNodes, SerializeNodes, Indent, Version, read_nodes, write_nodes};
use rand::SeedableRng;
use rand::rngs::OsRng;
use rand_xoshiro::Xoroshiro128StarStar;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

type Random = Xoroshiro128StarStar;

fn main() {
  let mut args = std::env::args_os().skip(1);
  let in_path = PathBuf::from(args.next().expect("no input path provided"));
  let out_path = args.next().map(PathBuf::from).unwrap_or_else(|| {
    let file_stem = in_path.file_stem().expect("invalid path");
    let extension = in_path.extension().expect("invalid path");
    let mut file_name = file_stem.to_owned();
    file_name.push(" (2x)");
    file_name.push(".");
    file_name.push(extension);

    in_path.with_file_name(file_name)
  });

  println!("reading fleet from {}", in_path.display());
  let reader = BufReader::new(File::open(&in_path).expect("failed to open file"));
  let nodes = read_nodes(reader).expect("failed to read file");
  let mut root = <Root<Fleet>>::deserialize_nodes(nodes).expect("failed to deserialize nodes");
  println!("successfully read fleet from {}", in_path.display());

  let original_name = root.element.name.clone();
  let original_ship_count = root.element.ships.len();

  let mut rng = Random::from_rng(OsRng).expect("failed to seed prng");
  root.element.double(&mut rng);
  println!("doubled fleet {original_name:?} of {original_ship_count} ships");
  if original_ship_count * 2 > 10 {
    println!("warning: the resulting fleet has more than 10 ships, it may not load correctly");
  };

  println!("writing fleet to {}", out_path.display());
  let writer = BufWriter::new(File::create(&out_path).expect("failed to create file"));
  let nodes = root.serialize_nodes().expect("failed to serialize nodes");
  write_nodes(writer, &nodes, Some(Indent::default()), Some(Version::default())).expect("failed to write nodes");
  println!("successfully wrote fleet to {}", out_path.display());

  pause();
}

fn pause() {
  use std::io::{Read, stdin};
  stdin().lock().read(&mut []).unwrap();
}
