use super::engines::Engine;
use crate::data::Faction;
use crate::utils::ContiguousExt;

use bytemuck::Contiguous;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissileBody {
  pub name: &'static str,
  pub save_key: &'static str,
  pub faction: Option<Faction>,
  pub variant: MissileBodyVariant
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MissileBodyVariant {
  Conventional {
    base_segments_length: usize,
    base_slider_range: (usize, usize),
    engine: &'static Engine,
    payload_mask: MissileComponentsMask,
    slots: &'static [(MissileComponentsMask, usize)]
  },
  Hybrid {
    cruise_segments_length: usize,
    cruise_engine: &'static Engine,
    sprint_segments_length: usize,
    sprint_slider_range: (usize, usize),
    sprint_engine: &'static Engine,
    payload_mask: MissileComponentsMask,
    slots: &'static [(MissileComponentsMask, usize)]
  }
}

impl MissileBodyVariant {
  pub const fn len(self) -> usize {
    let (mut len, mut slots) = match self {
      MissileBodyVariant::Conventional { base_segments_length, slots, .. } => {
        (base_segments_length, slots)
      },
      MissileBodyVariant::Hybrid { cruise_segments_length, sprint_segments_length, slots, .. } => {
        (cruise_segments_length + sprint_segments_length, slots)
      }
    };

    loop {
      if let Some((&slot, rest)) = slots.split_first() {
        slots = rest;
        len += slot.1;
      } else {
        break len;
      };
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MissileComponentsMask {
  pub allow_seekers: bool,
  pub allow_auxiliary: bool,
  pub allow_avionics: bool,
  pub allow_warheads: bool
}

impl MissileComponentsMask {
  pub const ALL: Self = Self { allow_seekers: true, allow_auxiliary: true, allow_avionics: true, allow_warheads: true };
  pub const NONE: Self = Self { allow_seekers: false, allow_auxiliary: false, allow_avionics: false, allow_warheads: false };
  pub const ALL_EXCEPT_AVIONICS: Self = Self { allow_avionics: false, ..Self::ALL };

  pub const ONLY_SEEKERS: Self = Self { allow_seekers: true, ..Self::NONE };
  pub const ONLY_AUXILIARY: Self = Self { allow_auxiliary: true, ..Self::NONE };
  pub const ONLY_WARHEADS: Self = Self { allow_warheads: true, ..Self::NONE };
  pub const ONLY_SEEKERS_AUXILIARY: Self = Self { allow_warheads: false, ..Self::ALL_EXCEPT_AVIONICS };
  pub const ONLY_WARHEADS_AUXILIARY: Self = Self { allow_seekers: false, ..Self::ALL_EXCEPT_AVIONICS };
  pub const ONLY_AVIONICS: Self = Self { allow_avionics: true, ..Self::NONE };
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MissileBodyKey {
  SGM1Balestra,
  SGM2Tempest,
  SGMH2Cyclone,
  SGMH3Atlatl,
  SGT3Pilum,
  CM4Container,
  CMS4Container,
}

impl MissileBodyKey {
  pub const fn save_key(self) -> &'static str {
    self.missile_body().save_key
  }

  pub const fn missile_body(self) -> &'static MissileBody {
    use self::list::*;

    match self {
      Self::SGM1Balestra => &SGM1_BALESTRA,
      Self::SGM2Tempest => &SGM2_TEMPEST,
      Self::SGMH2Cyclone => &SGMH2_CYCLONE,
      Self::SGMH3Atlatl => &SGMH3_ATLATL,
      Self::SGT3Pilum => &SGT3_PILUM,
      Self::CM4Container => &CM4_CONTAINER,
      Self::CMS4Container => &CMS4_CONTAINER
    }
  }
}

impl FromStr for MissileBodyKey {
  type Err = crate::data::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    MissileBodyKey::values()
      .find(|hull_key| hull_key.save_key() == s)
      .ok_or(crate::data::InvalidKey::MissileBody)
  }
}

impl fmt::Display for MissileBodyKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}

pub mod list {
  use super::*;
  use crate::data::missiles::engines::list::*;

  pub const SGM1_BALESTRA: MissileBody = MissileBody {
    name: "SGM-1",
    save_key: "Stock/SGM-1 Body",
    faction: None,
    variant: MissileBodyVariant::Conventional {
      base_segments_length: 7,
      base_slider_range: (3, 6),
      engine: &SGM1_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const SGM2_TEMPEST: MissileBody = MissileBody {
    name: "SGM-2",
    save_key: "Stock/SGM-2 Body",
    faction: None,
    variant: MissileBodyVariant::Conventional {
      base_segments_length: 10,
      base_slider_range: (4, 9),
      engine: &SGM2_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS_AUXILIARY,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const SGMH2_CYCLONE: MissileBody = MissileBody {
    name: "SGM-H-2",
    save_key: "Stock/SGM-H-2 Body",
    faction: Some(Faction::Alliance),
    variant: MissileBodyVariant::Hybrid {
      cruise_segments_length: 1,
      cruise_engine: &SGMH2_CRUISE_ENGINE,
      sprint_segments_length: 8,
      sprint_slider_range: (1, 7),
      sprint_engine: &SGMH2_SPRINT_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const SGMH3_ATLATL: MissileBody = MissileBody {
    name: "SGM-H-3",
    save_key: "Stock/SGM-H-3 Body",
    faction: Some(Faction::Alliance),
    variant: MissileBodyVariant::Hybrid {
      cruise_segments_length: 1,
      cruise_engine: &SGMH3_CRUISE_ENGINE,
      sprint_segments_length: 10,
      sprint_slider_range: (1, 9),
      sprint_engine: &SGMH3_SPRINT_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const SGT3_PILUM: MissileBody = MissileBody {
    name: "SGT-3",
    save_key: "Stock/SGT-3 Body",
    faction: None,
    variant: MissileBodyVariant::Conventional {
      base_segments_length: 14,
      base_slider_range: (5, 13),
      engine: &SGT3_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS_AUXILIARY,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const CM4_CONTAINER: MissileBody = MissileBody {
    name: "CM-4",
    save_key: "Stock/CM-4 Body",
    faction: Some(Faction::Protectorate),
    variant: MissileBodyVariant::Conventional {
      base_segments_length: 14,
      base_slider_range: (5, 13),
      engine: &CM4_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_WARHEADS,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };

  pub const CMS4_CONTAINER: MissileBody = MissileBody {
    name: "CM-S-4",
    save_key: "Stock/CM-S-4 Body",
    faction: Some(Faction::Protectorate),
    variant: MissileBodyVariant::Conventional {
      base_segments_length: 14,
      base_slider_range: (5, 13),
      engine: &CM4_ENGINE,
      payload_mask: MissileComponentsMask::ONLY_AUXILIARY,
      slots: &[
        (MissileComponentsMask::ONLY_AVIONICS, 1),
        (MissileComponentsMask::ALL_EXCEPT_AVIONICS, 1),
        (MissileComponentsMask::ONLY_SEEKERS_AUXILIARY, 1),
        (MissileComponentsMask::ONLY_SEEKERS, 1)
      ]
    }
  };
}
