use crate::format::{Color, Vector3, HullConfig, SegmentConfiguration, SecondaryStructureConfig};
use super::Variant;

use nebulous_xml::uuid::{Uuid, uuid};
#[cfg(feature = "rand")]
use rand::distributions::Distribution;
#[cfg(feature = "rand")]
use rand::seq::SliceRandom;
#[cfg(feature = "rand")]
use rand::Rng;

use std::array::from_fn as array_from_fn;
use std::iter::repeat_with;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HullConfigBulkFreighter;

#[cfg(feature = "rand")]
impl Distribution<HullConfig> for HullConfigBulkFreighter {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HullConfig {
    let variants = rng.gen::<[Variant; 3]>();
    let dressing_counts = get_dressing_counts(variants);

    let primary_structure = array_from_fn(|i| {
      let key = *variants[i].select_array(&PRIMARY_STRUCTURES[i]);
      let dressing = repeat_with(|| rng.gen_range(0..=1))
        .take(dressing_counts[i]).collect();
      SegmentConfiguration { key, dressing }
    });

    let bridge_locations = can_mount_bridges(variants);
    let mut bridge_location = rng.gen_range(0..3);
    while !bridge_locations[bridge_location] {
      bridge_location = rng.gen_range(0..3);
    };

    let secondary_structure = SecondaryStructureConfig {
      key: *SECONDARY_STRUCTURES.choose(rng).unwrap(),
      segment: bridge_location,
      snap_point: 0
    };

    HullConfig::RandomHullConfiguration {
      primary_structure,
      secondary_structure,
      hull_tint: Color::splat(rng.gen_range(MIN_TINT..MAX_TINT), 1.0),
      texture_variation: Vector3 {
        x: rng.gen_range(MIN_VARIATION..MAX_VARIATION),
        y: rng.gen_range(MIN_VARIATION..MAX_VARIATION),
        z: rng.gen_range(MIN_VARIATION..MAX_VARIATION)
      }
    }
  }
}

pub(super) const MIN_TINT: f32 = 0.35;
pub(super) const MAX_TINT: f32 = 0.80;

pub(super) const MIN_VARIATION: f32 = -1000.0;
pub(super) const MAX_VARIATION: f32 = 1000.0;

pub(super) const PRIMARY_STRUCTURES: [[Uuid; 3]; 3] = [
  [
    uuid!("29eb9c63-6c47-40f2-8f46-4ed4da8d3386"),
    uuid!("38e7a28f-1b06-4b73-98ee-f03d1d8a81fe"),
    uuid!("c534a876-3f8a-4315-a194-5dda0f84c2b3")
  ],
  [
    uuid!("d4c9a66d-81e6-49ee-9b33-82d7a1522bbf"),
    uuid!("e2c11e02-b770-495e-a3c2-3dc998eac5a6"),
    uuid!("429f178e-e369-4f51-8054-2e01dd0abea1")
  ],
  [
    uuid!("a8bf77b9-b7e3-4498-bf91-d3e777a7f688"),
    uuid!("2f2b451c-4776-405c-9914-cad4764f1072"),
    uuid!("78d72a9a-893c-41c6-bddd-f198dfcf77ee")
  ]
];

pub(super) const SECONDARY_STRUCTURES: [Uuid; 4] = [
  uuid!("42d07c1a-156b-4057-aaca-7a2024751423"),
  uuid!("59344a67-9e7b-43df-9f7c-505ad9a0ab87"),
  uuid!("9ebcea74-e9c9-45b3-b616-e12e3f491024"),
  uuid!("c9d04445-3558-46b4-b6fc-7dca8617d438")
];

const fn get_dressing_counts(variants: [Variant; 3]) -> [usize; 3] {
  // front dressing is 2 or 3 depending on the front segment
  // middle dressing can be between 5 and 8 depending on the front and middle segments
  // rear dressing is always 0
  let front = variants[0].select(3, 3, 2);
  let middle = variants[0].select(1, 1, 0) + variants[1].select(1, 2, 0) + 5;
  [front, middle, 0]
}

const fn can_mount_bridges(variants: [Variant; 3]) -> [bool; 3] {
  let front = !matches!(variants[0], Variant::V1);
  let rear = !matches!(variants[2], Variant::V1);
  [front, true, rear]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn correct_dressing_counts() {
    use Variant::*;
    assert_eq!(get_dressing_counts([V0, V0, V0]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V0, V0, V1]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V0, V0, V2]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V0, V1, V0]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V0, V1, V1]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V0, V1, V2]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V0, V2, V0]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V0, V2, V1]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V0, V2, V2]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V1, V0, V0]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V1, V0, V1]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V1, V0, V2]), [3, 7, 0]);
    assert_eq!(get_dressing_counts([V1, V1, V0]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V1, V1, V1]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V1, V1, V2]), [3, 8, 0]);
    assert_eq!(get_dressing_counts([V1, V2, V0]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V1, V2, V1]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V1, V2, V2]), [3, 6, 0]);
    assert_eq!(get_dressing_counts([V2, V0, V0]), [2, 6, 0]);
    assert_eq!(get_dressing_counts([V2, V0, V1]), [2, 6, 0]);
    assert_eq!(get_dressing_counts([V2, V0, V2]), [2, 6, 0]);
    assert_eq!(get_dressing_counts([V2, V1, V0]), [2, 7, 0]);
    assert_eq!(get_dressing_counts([V2, V1, V1]), [2, 7, 0]);
    assert_eq!(get_dressing_counts([V2, V1, V2]), [2, 7, 0]);
    assert_eq!(get_dressing_counts([V2, V2, V0]), [2, 5, 0]);
    assert_eq!(get_dressing_counts([V2, V2, V1]), [2, 5, 0]);
    assert_eq!(get_dressing_counts([V2, V2, V2]), [2, 5, 0]);
  }
}
