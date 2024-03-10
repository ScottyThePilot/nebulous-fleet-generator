use crate::data::Faction;

use bytemuck::Contiguous;

use std::fmt;
use std::str::FromStr;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissileBody {
  pub name: &'static str,
  pub save_key: &'static str,
  pub faction: Option<Faction>
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
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

  #[inline]
  pub const fn values() -> crate::utils::ContiguousEnumValues<Self> {
    crate::utils::ContiguousEnumValues::new()
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

  pub const SGM1_BALESTRA: MissileBody = MissileBody {
    name: "SGM-1",
    save_key: "Stock/SGM-1 Body",
    faction: None
  };

  pub const SGM2_TEMPEST: MissileBody = MissileBody {
    name: "SGM-2",
    save_key: "Stock/SGM-2 Body",
    faction: None
  };

  pub const SGMH2_CYCLONE: MissileBody = MissileBody {
    name: "SGM-H-2",
    save_key: "Stock/SGM-H-2 Body",
    faction: Some(Faction::Alliance)
  };

  pub const SGMH3_ATLATL: MissileBody = MissileBody {
    name: "SGM-H-3",
    save_key: "Stock/SGM-H-3 Body",
    faction: Some(Faction::Alliance)
  };

  pub const SGT3_PILUM: MissileBody = MissileBody {
    name: "SGT-3",
    save_key: "Stock/SGT-3 Body",
    faction: None
  };

  pub const CM4_CONTAINER: MissileBody = MissileBody {
    name: "CM-4",
    save_key: "Stock/CM-4 Body",
    faction: Some(Faction::Protectorate)
  };

  pub const CMS4_CONTAINER: MissileBody = MissileBody {
    name: "CM-S-4",
    save_key: "Stock/CM-S-4 Body",
    faction: Some(Faction::Protectorate)
  };
}
