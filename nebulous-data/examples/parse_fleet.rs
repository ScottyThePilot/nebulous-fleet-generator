use nebulous_data::format::{Fleet, Root};
use nebulous_data::xml::{DeserializeNodes, SerializeNodes, Indent, Version, read_nodes, write_nodes};

use std::fs::File;
use std::io::{BufReader, Cursor};

/// Performs a round-trip on a fleet file
///
/// Takes two arguments:
/// 1. Input path for the .fleet file
/// 2. Output path for the .fleet file
fn main() {
  let mut args = std::env::args_os().skip(1);
  let in_path = args.next().expect("no input path provided");
  let out_path = args.next().expect("no output path provided");

  let reader = BufReader::new(File::open(in_path).expect("failed to open file"));
  let nodes = read_nodes(reader).expect("failed to read file");
  let fleet = <Root<Fleet>>::deserialize_nodes(nodes).expect("failed to deserialize nodes");
  let nodes = fleet.clone().serialize_nodes().expect("failed to serialize nodes");

  let mut buffer = Cursor::new(Vec::new());
  write_nodes(&mut buffer, &nodes, Some(Indent::default()), Some(Version::default())).expect("failed to write nodes");
  std::fs::write(out_path, buffer.get_ref()).expect("failed to save file");

  println!("{:#?}", fleet.element);
}
