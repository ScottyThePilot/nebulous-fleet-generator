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
pub struct HullConfigContainerLiner;

#[cfg(feature = "rand")]
impl Distribution<HullConfig> for HullConfigContainerLiner {
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

pub(super) const MIN_TINT: f32 = super::bulk_freighter::MIN_TINT;
pub(super) const MAX_TINT: f32 = super::bulk_freighter::MAX_TINT;

pub(super) const MIN_VARIATION: f32 = super::bulk_freighter::MIN_VARIATION;
pub(super) const MAX_VARIATION: f32 = super::bulk_freighter::MAX_VARIATION;

pub(super) const PRIMARY_STRUCTURES: [[Uuid; 3]; 3] = [
  [
    uuid!("541cf476-4952-4234-a35a-5f1aa9089316"),
    uuid!("2d7c228c-cbd6-425e-9590-a2f8ae8d5915"),
    uuid!("bb034299-84c2-456f-b271-c91249cd4375")
  ],
  [
    uuid!("09354e51-953c-451a-b415-3e3361812650"),
    uuid!("18a6bc15-58b0-479c-82c3-1722768f033d"),
    uuid!("2c68a462-a143-4c89-aea0-df09d4786e92")
  ],
  [
    uuid!("aff1eba2-048e-4477-956b-574f4d468f1d"),
    uuid!("674e0528-3e0c-48e4-8e5e-d3a559869104"),
    uuid!("2dbd82fe-d365-4367-aef5-9bb2d3528528")
  ]
];

pub(super) const SECONDARY_STRUCTURES: [Uuid; 4] = super::bulk_freighter::SECONDARY_STRUCTURES;

const fn get_dressing_counts(variants: [Variant; 3]) -> [usize; 3] {
  // front dressing is always 1
  // middle dressing is 3 or 4 depending on the middle segment
  // rear dressing is always 1
  [1, variants[1].select(0, 1, 0) + 3, 1]
}

const fn can_mount_bridges(variants: [Variant; 3]) -> [bool; 3] {
  let front = !matches!(variants[0], Variant::V1);
  [front, true, true]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn correct_dressing_counts() {
    use Variant::*;
    assert_eq!(get_dressing_counts([V0, V0, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V0, V0, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V0, V0, V2]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V0, V1, V0]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V0, V1, V1]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V0, V1, V2]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V0, V2, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V0, V2, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V0, V2, V2]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V0, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V0, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V0, V2]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V1, V0]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V1, V1, V1]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V1, V1, V2]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V1, V2, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V2, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V1, V2, V2]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V0, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V0, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V0, V2]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V1, V0]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V2, V1, V1]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V2, V1, V2]), [1, 4, 1]);
    assert_eq!(get_dressing_counts([V2, V2, V0]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V2, V1]), [1, 3, 1]);
    assert_eq!(get_dressing_counts([V2, V2, V2]), [1, 3, 1]);
  }
}
