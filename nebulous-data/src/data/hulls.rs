use super::components::ComponentKind;
use super::{Buff, Direction, Faction};
use crate::utils::{ContiguousExt, Size};

use bytemuck::Contiguous;

use std::fmt;
use std::str::FromStr;



#[derive(Debug, Clone, Copy)]
pub struct Hull {
  pub name: &'static str,
  pub save_key: &'static str,
  pub faction: Faction,
  pub point_cost: usize,
  pub mass: f32,
  pub max_speed: f32,
  pub max_turn_speed: f32,
  pub linear_motor: f32,
  pub angular_motor: f32,
  pub base_integrity: f32,
  pub armor_thickness: f32,
  pub base_crew_complement: usize,
  pub buffs: &'static [(Buff, f32)],
  pub sockets: &'static [HullSocket],
  pub socket_symmetries: &'static [(&'static str, &'static str)]
}

#[derive(Debug, Clone, Copy)]
pub struct HullSocket {
  pub save_key: &'static str,
  pub kind: ComponentKind,
  pub size: Size,
  pub direction: Option<Direction>,
  pub desirability: f32
}

impl HullSocket {
  #[inline]
  const fn mount(save_key: &'static str, size: Size, direction: Direction, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Mount, size, direction: Some(direction), desirability }
  }

  #[inline]
  const fn mount_unknown(save_key: &'static str, size: Size, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Mount, size, direction: None, desirability }
  }

  #[inline]
  const fn compartment(save_key: &'static str, size: Size, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Compartment, size, direction: None, desirability }
  }

  #[inline]
  const fn module(save_key: &'static str, size: Size, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Mount, size, direction: None, desirability }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Contiguous)]
pub enum HullKey {
  SprinterCorvette,
  RainesFrigate,
  KeystoneDestroyer,
  VauxhallLightCruiser,
  AxfordHeavyCruiser,
  SolomonBattleship,
  ShuttleClipper,
  TugboatClipper,
  CargoFeederMonitor,
  OcelloCommandCruiser,
  BulkFreighterLineShip,
  ContainerLinerLineShip
}

impl HullKey {
  pub const fn faction(self) -> Faction {
    self.hull().faction
  }

  pub const fn save_key(self) -> &'static str {
    self.hull().save_key
  }

  pub const fn hull(self) -> &'static Hull {
    use self::list::*;

    match self {
      Self::SprinterCorvette => &SPRINTER_CORVETTE,
      Self::RainesFrigate => &RAINES_FRIGATE,
      Self::KeystoneDestroyer => &KEYSTONE_DESTROYER,
      Self::VauxhallLightCruiser => &VAUXHALL_LIGHT_CRUISER,
      Self::AxfordHeavyCruiser => &AXFORD_HEAVY_CRUISER,
      Self::SolomonBattleship => &SOLOMON_BATTLESHIP,
      Self::ShuttleClipper => &SHUTTLE_CLIPPER,
      Self::TugboatClipper => &TUGBOAT_CLIPPER,
      Self::CargoFeederMonitor => &CARGO_FEEDER_MONITOR,
      Self::OcelloCommandCruiser => &OCELLO_COMMAND_CRUISER,
      Self::BulkFreighterLineShip => &BULK_FREIGHTER_LINE_SHIP,
      Self::ContainerLinerLineShip => &CONTAINER_LINER_LINE_SHIP
    }
  }
}

impl FromStr for HullKey {
  type Err = super::InvalidKey;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    HullKey::values()
      .find(|hull_key| hull_key.save_key() == s)
      .ok_or(super::InvalidKey::Hull)
  }
}

impl fmt::Display for HullKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(self.save_key())
  }
}



pub mod list {
  use super::*;
  use super::Direction::*;

  pub const SPRINTER_CORVETTE: Hull = Hull {
    name: "Sprinter Corvette",
    save_key: "Stock/Sprinter Corvette",
    faction: Faction::Alliance,
    point_cost: 100,
    mass: 3080.0,
    max_speed: 35.0,
    max_turn_speed: 5.729578,
    linear_motor: 5.0,
    angular_motor: 0.9,
    base_integrity: 1000.0,
    armor_thickness: 8.0,
    base_crew_complement: 40,
    buffs: &[
      (Buff::FlankDamageProbability, -0.2),
      (Buff::PositionalError, -0.15),
      (Buff::Sensitivity, 0.2)
    ],
    sockets: &[
      HullSocket::mount("wDsRnL5nKkyYvKgD6VcPHg", Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount("Z48ot_dQfkWb6AVYjaM_gA", Size::new(3, 2, 3), Down, 1.0),
      HullSocket::mount("IUdNSVZm2Eu9n3F5HnSAng", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount("ZxY9ONYz80SiLNSObvjNzQ", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment("XPaYCjBqdEqBxEIsVLP0oA", Size::new(4, 1, 8), 1.0),
      HullSocket::compartment("xRYIkvssd0mPY3VIgrnQ5A", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("rXO8xwG1MkqzI_2pU-O_qQ", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("GefqwCQzg0qXA3EFRmDtPw", Size::new(3, 1, 3), 1.0),
      HullSocket::module("4WpJyiOKVEqCwR99l47MkA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("4EHx4mQhNUi-WuuQtQF5fA", Size::new(8, 3, 6), 1.0),
      HullSocket::module("TSPPW9ECe06-MGzR1i0WvQ", Size::new(2, 2, 2), 1.0),
      HullSocket::module("MBOvGazj6UWpt2OOy44s7w", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("IUdNSVZm2Eu9n3F5HnSAng", "ZxY9ONYz80SiLNSObvjNzQ"),
      ("TSPPW9ECe06-MGzR1i0WvQ", "MBOvGazj6UWpt2OOy44s7w")
    ]
  };

  pub const RAINES_FRIGATE: Hull = Hull {
    name: "Raines Frigate",
    save_key: "Stock/Raines Frigate",
    faction: Faction::Alliance,
    point_cost: 125,
    mass: 5095.0,
    max_speed: 22.0,
    max_turn_speed: 4.5836625,
    linear_motor: 6.0,
    angular_motor: 1.1,
    base_integrity: 3000.0,
    armor_thickness: 15.0,
    base_crew_complement: 50,
    buffs: &[
      (Buff::FlankDamageProbability, -0.15),
      (Buff::MissileProgrammingChannels, 1.0)
    ],
    sockets: &[
      HullSocket::mount("PDKmGfvpykODc3XHDQ7WBw", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount("WwMGqYiU7E6lp7ID47phqA", Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount("gkfIwYzn7kGhG6MW9uxBmw", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("D3zoB0PetEC973iUck_mNQ", Size::new(3, 4, 5), Down, 1.0),
      HullSocket::compartment("98N-YI_1WUOs--qPYlt-7g", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("6Tzo7268MEqgyFxnGvLKUg", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("O_Y-AJc7r0alOg36rvY6MQ", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("5APOD4UfSkGdkm9WWdAvLA", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("lorATFhkTkeZQYThYOHazg", Size::new(4, 1, 6), 1.0),
      HullSocket::module("bRgUlaoQJ0S7M91zMpdAdA", Size::new(6, 6, 6), 1.0),
      HullSocket::module("Jmtnwo0KQki5QyEPPlBHrA", Size::new(8, 3, 6), 1.0),
      HullSocket::module("V42tXibIR0e4u6riIHYZOw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("p-m7ijS6ukuqzuBT5NE_lA", Size::new(2, 2, 2), 1.0),
      HullSocket::module("foApM84hT0GpEw4GaFv38w", Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      ("V42tXibIR0e4u6riIHYZOw", "p-m7ijS6ukuqzuBT5NE_lA")
    ]
  };

  pub const KEYSTONE_DESTROYER: Hull = Hull {
    name: "Keystone Destroyer",
    save_key: "Stock/Keystone Destroyer",
    faction: Faction::Alliance,
    point_cost: 200,
    mass: 8095.0,
    max_speed: 20.0,
    max_turn_speed: 4.5836625,
    linear_motor: 9.0,
    angular_motor: 1.2,
    base_integrity: 4000.0,
    armor_thickness: 22.0,
    base_crew_complement: 50,
    buffs: &[
      (Buff::MissileProgrammingChannels, 1.0),
      (Buff::OverheatDamageChanceBeam, -0.75),
      (Buff::PowerplantEfficiency, 0.25)
    ],
    sockets: &[
      HullSocket::mount("NpYYAkA5Z0uAElOjDg3Rag", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("fSYv4j5-eEObwJEVhoBSvg", Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount("x21sadthgE2FtDWSJKINGQ", Size::new(3, 4, 4), Left, 1.0),
      HullSocket::mount("IxKwpY5d9EicYRRTMn8xVw", Size::new(3, 4, 4), Right, 1.0),
      HullSocket::mount("JpOU9MgWXEmfLcQoAK4otA", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount("LCosRblddUCjPogeJ5eSeA", Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount("hGeid9yk80GqIN5IUlp8aw", Size::new(4, 12, 4), Fore, 1.0),
      HullSocket::compartment("PlyaCcjDo0qu155JIZgm6A", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("ytRGy84X_kW1odHYCBICDQ", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("yM3Am4PqyUOIByTKzM6sPA", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("H1ekC2-9c02VyNXY4IeLYA", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("ln5c_ZvdCUSMVWZL_-m3yw", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("H-oABJbYw0qgKd4QzU0WaA", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("tCBEdcIUhk6AL-X6sCujsQ", Size::new(6, 1, 4), 1.0),
      HullSocket::module("AWYvQUZ26k20yHOrVQKg6Q", Size::new(6, 6, 6), 1.0),
      HullSocket::module("tmHTz6HkrE-RUtN94W6lyQ", Size::new(8, 8, 6), 1.0),
      HullSocket::module("m5J1jQdOfUKM9r7NZjMpKA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("h29TSNGRo0m_DmeRr2BqDw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("E9sMCEquVk-NNj5heGmIlQ", Size::new(2, 2, 2), 1.0),
      HullSocket::module("gx2UKetKWUm5BfZhiD5phQ", Size::new(2, 2, 2), 1.0),
      HullSocket::module("ZJTmbGdjAk-XEKihIYHEuw", Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      ("x21sadthgE2FtDWSJKINGQ", "IxKwpY5d9EicYRRTMn8xVw")
    ]
  };

  pub const VAUXHALL_LIGHT_CRUISER: Hull = Hull {
    name: "Vauxhall Light Cruiser",
    save_key: "Stock/Vauxhall Light Cruiser",
    faction: Faction::Alliance,
    point_cost: 350,
    mass: 10140.0,
    max_speed: 26.0,
    max_turn_speed: 4.5836625,
    linear_motor: 13.0,
    angular_motor: 3.0,
    base_integrity: 5500.0,
    armor_thickness: 30.0,
    base_crew_complement: 75,
    buffs: &[
      (Buff::MissileProgrammingChannels, 2.0)
    ],
    sockets: &[
      HullSocket::mount("T9Ebo41iA0eBXNYx8uyisw", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("2QQdxC4UE0KOM42r82-ETQ", Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount("IFKM9E04aUaHS0IoNDMShA", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("rX6hes7UqkyHjEyKbFaWTQ", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("BRMwuusKC02YrbFXRrrNzA", Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount("vO1oPhlSuUih_cdAZk3Hqg", Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount("RLHQUFf200uLZjKX5axkMw", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount("uyPDg0tD3U6YKz18bVdkPg", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount("Xicf0TT7pEaFy_x1uk7ueQ", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount("XTg1H1Popku5gxW8sei5XQ", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment("a8H-rPyVSk6uBOrP7lQYUA", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("NBbnpHfpDUSS6bNgDlSjzw", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("f_SO68qAGU-sw69T9GEWow", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("9R0tfjsN-kiETUBLeCCmGw", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("-bAF6R6aiU2Afn8T_oKY4g", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("-7H3WGDwyESDI3lkHzBdGQ", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("o1o5WOogUE255YVi43KlcQ", Size::new(4, 1, 8), 1.0),
      HullSocket::compartment("DAcDeA0yskSjje92CY-F7w", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("jKvCg9nYmkKfAn4-ht_Y6A", Size::new(6, 1, 8), 1.0),
      HullSocket::module("8VKMmbA23Em1f-MBZKO2vA", Size::new(6, 6, 6), 1.0),
      HullSocket::module("udQjNCWlA0awkv6LhH55uQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("3eXWUksWXk-5q9MW52rghw", Size::new(8, 12, 10), 1.0),
      HullSocket::module("_jqFwgf3EkKUUqHzVWidzw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("qulLVFUtzk2Qm8rZtfpFDw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("N_gGUWCQx0mUOX9RaFoI6A", Size::new(3, 3, 3), 1.0),
      HullSocket::module("E1ZSbpYpZkufCyifYiiK8Q", Size::new(2, 2, 2), 1.0),
      HullSocket::module("-hETYcmKH0eeXzkFpnhgeg", Size::new(2, 2, 2), 1.0),
      HullSocket::module("hU2L93VfQU-jutnRC0dKgw", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("RLHQUFf200uLZjKX5axkMw", "uyPDg0tD3U6YKz18bVdkPg"),
      ("Xicf0TT7pEaFy_x1uk7ueQ", "XTg1H1Popku5gxW8sei5XQ")
    ]
  };

  pub const AXFORD_HEAVY_CRUISER: Hull = Hull {
    name: "Axford Heavy Cruiser",
    save_key: "Stock/Axford Heavy Cruiser",
    faction: Faction::Alliance,
    point_cost: 600,
    mass: 13140.0,
    max_speed: 18.0,
    max_turn_speed: 4.5836625,
    linear_motor: 14.0,
    angular_motor: 2.55,
    base_integrity: 6500.0,
    armor_thickness: 40.0,
    base_crew_complement: 90,
    buffs: &[
      (Buff::MaxRepair, 0.15),
      (Buff::MissileProgrammingChannels, 2.0)
    ],
    sockets: &[
      HullSocket::mount("whDP9rtbukKsocnrqnqaLQ", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("8CtCKPLfZEOOFvm1DhV-oQ", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("N7yRYYUuG0uOzd5aufgViA", Size::new(8, 7, 8), Down, 1.0),
      HullSocket::mount("lYEQo4jJvEGxRqJa2MB25A", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("9qreGP2Iw0KwuloNpNFTSg", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("v9Sqa5DcCkibdi29AkakNg", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("3MoeYUx1HUS6xNDBMsssig", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("LbD9Txe-S0a_nODTqtrk7g", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount("KxV0hkkGBE2AOUCo5axQbg", Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount("tyGXrucCjUe0FVyP1YDpUw", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount("rXZhdtbK3EC8K_cu6-r_Xg", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::compartment("ylUvmASKlEiyCtyCsNRKWg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("TX2TktZe7EWB9s-MqlDQgA", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("cMs0g0g3WUeZJQASb7omew", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("iHYg3DdYAUKS-uIBaRpNqA", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("j2ST2yYfwk2_tBIayI3sQg", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("SV_4BgcxwU24YIbep-k74g", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("A8e_Z_d8WU2qzo6HTQwCBA", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("2xmOS4ns_EaMYjBXrrhhig", Size::new(6, 1, 6), 1.0),
      HullSocket::compartment("y1xLixKUzUSKT7Nz7aUkLQ", Size::new(6, 1, 6), 1.0),
      HullSocket::compartment("RmUeePb-8EGbFaRgVjCXsA", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("eYp1IPfupUGVITrkVhgZJw", Size::new(4, 1, 6), 1.0),
      HullSocket::module("lxkgmNwqjUua0sRee2pDJg", Size::new(6, 6, 6), 1.0),
      HullSocket::module("_Aun-HW-DkOGytVmYSn5IQ", Size::new(6, 6, 6), 1.0),
      HullSocket::module("2ZLEJvxYf0CATl1lBi3cuQ", Size::new(12, 8, 10), 1.0),
      HullSocket::module("ba-3VQ-zGEmDCfU12uaqaA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("hQ-Mzyns0UO4oMp7gG0d8A", Size::new(3, 3, 3), 1.0),
      HullSocket::module("oyAxfnepkEG46VjtvQWXpw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("Yo9hPw6Mf060B3vqwSO4GA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("CgpRQa-660KEed1wTRL4vw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("awwdeDnNs0G-RceQF1TByA", Size::new(2, 2, 2), 1.0),
      HullSocket::module("tnZD-B5Ls0qAyBtdz4yoTA", Size::new(2, 2, 2), 1.0),
      HullSocket::module("S8ALcpOMYEqJog1v2q8UMQ", Size::new(2, 2, 2), 1.0),
      HullSocket::module("rQWETsDGdE-AR8N3NPfwXg", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("lYEQo4jJvEGxRqJa2MB25A", "v9Sqa5DcCkibdi29AkakNg"),
      ("9qreGP2Iw0KwuloNpNFTSg", "3MoeYUx1HUS6xNDBMsssig"),
      ("tyGXrucCjUe0FVyP1YDpUw", "rXZhdtbK3EC8K_cu6-r_Xg")
    ]
  };

  pub const SOLOMON_BATTLESHIP: Hull = Hull {
    name: "Solomon Battleship",
    save_key: "Stock/Solomon Battleship",
    faction: Faction::Alliance,
    point_cost: 1000,
    mass: 21220.0,
    max_speed: 16.0,
    max_turn_speed: 4.5836625,
    linear_motor: 18.0,
    angular_motor: 3.85,
    base_integrity: 8000.0,
    armor_thickness: 58.0,
    base_crew_complement: 150,
    buffs: &[
      (Buff::MaxRepair, 0.05)
    ],
    sockets: &[
      HullSocket::mount("vvpl_pV8B0W33ernrw31jg", Size::new(8, 7, 8), Up, 1.0),
      HullSocket::mount("AznvpY2By0GiQxgeOTktww", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("N5nMrtk4bkiSDzOffAmI1w", Size::new(8, 7, 8), Up, 1.0),
      HullSocket::mount("HsGDhCR_T0KH1ddzUnSHmA", Size::new(8, 7, 8), Down, 1.0),
      HullSocket::mount("oRZYmSFdsk2BFB03PqyS2w", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("csaNZW85vkO8uIQ9XDe1iw", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("eBG0UhrBiky4nUx81rPbLw", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("M6NaSvLam06C-1DTggI99g", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount("3HKNGt305UC23fGT8OKcLw", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("R_Y-LTCc0kydl6L8Sq5Dmw", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("-gtO0aqGDEOH-Tm612wAoA", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("37oZ894X7kG2Dq0E5mEKDQ", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount("ZjH6aPXnl0eOdklaPsRNAA", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount("4dNKr5i87EeDGq9coaWYJg", Size::new(2, 2, 2), Down, 1.0),
      HullSocket::compartment("XAbGPXPgDEuDieQDqbA9MQ", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("SX-a4J4eek2nmJztXePU3Q", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("4sUWwP0VukOFbXT1YAMa9g", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("09AenxIgokykwMTCMJz24w", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("RtZe85sJUkO5b1xVdgPlqQ", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("8YQv6H5fN0SylDc4p2fT-w", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("6P8ohHGH_UaCAMEE5jFoRg", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("80T00-71i0Wz-uNk04dzvQ", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("GbGNS6ZFo0qx-dxuEmZTbg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("tkt9JPA2V0CWAO-Dfj8FHQ", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("QkO76Z2PAUq8iD4cuc_P4A", Size::new(6, 1, 4), 1.0),
      HullSocket::compartment("lp31cinWLE-ZGgtisuoaug", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("wbgVupdnzEGle6f8tezZYQ", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("Jl5HmssP406Au7n4NHceQw", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("-wa5U_hdw0mHKI1PglDeGg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("97l_f_ktb0yoEquIa75icg", Size::new(6, 1, 8), 1.0),
      HullSocket::module("Cz_cm8aRt0mRiYxdq26V7A", Size::new(8, 8, 8), 1.0),
      HullSocket::module("eOvmJ625f0yGavYASYtYYQ", Size::new(8, 8, 8), 1.0),
      HullSocket::module("1I5WRAX0oU6zIv7yEvqfmQ", Size::new(16, 8, 10), 1.0),
      HullSocket::module("98aZYYnrXUWKXheUXHoLLw", Size::new(12, 8, 10), 1.0),
      HullSocket::module("cVsLdxme_kGSU2jWlYnXzg", Size::new(3, 3, 3), 1.0),
      HullSocket::module("rWWXZNnbKUi7djDPaKTttQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("tgWD8fbOBEWq4KfvYDN_BQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("4wfe6HbB9kCN7CDcF7_POQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("yAg2wUCkJ0a6Ef3m96WyYw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("rTbYkycwWku-Ye1FCBJbvw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("-ao2fBk-zkK3QTdVH4fqHA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("5lOFQqLf-kOQq0zbIv0MKw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("PWWRnNTo60Cx9faCbUhx-g", Size::new(2, 2, 2), 1.0),
      HullSocket::module("31ilgFi2f0ScvJbFia-Opg", Size::new(2, 2, 2), 1.0),
      HullSocket::module("MjJFzcmp2kCLRWmjC30TKA", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("oRZYmSFdsk2BFB03PqyS2w", "3HKNGt305UC23fGT8OKcLw"),
      ("csaNZW85vkO8uIQ9XDe1iw", "R_Y-LTCc0kydl6L8Sq5Dmw"),
      ("eBG0UhrBiky4nUx81rPbLw", "-gtO0aqGDEOH-Tm612wAoA"),
      ("M6NaSvLam06C-1DTggI99g", "37oZ894X7kG2Dq0E5mEKDQ")
    ]
  };

  pub const SHUTTLE_CLIPPER: Hull = Hull {
    name: "Shuttle Clipper",
    save_key: "Stock/Shuttle",
    faction: Faction::Protectorate,
    point_cost: 50,
    mass: 1580.0,
    max_speed: 35.0,
    max_turn_speed: 5.729578,
    linear_motor: 5.0,
    angular_motor: 0.65,
    base_integrity: 1000.0,
    armor_thickness: 5.0,
    base_crew_complement: 55,
    buffs: &[
      (Buff::FlankDamageProbability, -0.2)
    ],
    sockets: &[
      HullSocket::mount("3mlCA6C6m0GI3wONZK_aqg", Size::new(3, 2, 3), Up, 1.0),
      HullSocket::mount("fbAgHM9u302peFMSC0TidQ", Size::new(3, 2, 3), Down, 1.0),
      HullSocket::mount("rbZhGCN7UUWcgLsNZkZXIA", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::compartment("hh8PA01jxEytn9KuihcJ3Q", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("wDWAeeoAK0mAx3hbF1tMlA", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("CcSXjIupV0Kw-jxaV6HpEA", Size::new(3, 1, 3), 1.0),
      HullSocket::module("bInyh5AAt0y_m-3UaI8Ysw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("-harMwD0o0OExuMqT63fTQ", Size::new(8, 3, 6), 1.0),
      HullSocket::module("zUeDN4FpYEyyWHIhTIgZFg", Size::new(2, 2, 2), 1.0),
      HullSocket::module("uXJ_fvsZ3USAs1SpiTrZ1g", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[]
  };

  pub const TUGBOAT_CLIPPER: Hull = Hull {
    name: "Tugboat Clipper",
    save_key: "Stock/Tugboat",
    faction: Faction::Protectorate,
    point_cost: 75,
    mass: 3595.0,
    max_speed: 26.0,
    max_turn_speed: 5.729578,
    linear_motor: 6.0,
    angular_motor: 1.5,
    base_integrity: 1000.0,
    armor_thickness: 5.0,
    base_crew_complement: 40,
    buffs: &[],
    sockets: &[
      HullSocket::mount("03vxralGq0CFx1XKxjOcBA", Size::new(2, 5, 2), Fore, 1.0),
      HullSocket::mount("cIGspM4oYU2etRfjIcuCCA", Size::new(3, 2, 5), Up, 1.0),
      HullSocket::mount("0upOgm0H0UuZ-h2YvRjmBg", Size::new(3, 2, 5), Down, 1.0),
      HullSocket::mount("KnFHQkb9uUe98IweLYf9wg", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount("5kd20hGcxUuTlY-6JYaudA", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment("F6bmJdCO3EqzXK9SBJoXUw", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("-PqoLKimSUOZtKMLo5QHXg", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("DakKS2gy6UyM58u3YG-HfQ", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("peqaYrE39UupWCXPY_PgoA", Size::new(3, 1, 3), 1.0),
      HullSocket::compartment("6OieSH0A6kKFmquctn9_aQ", Size::new(4, 1, 6), 1.0),
      HullSocket::module("NSAbhn9brE-aAirJFIYG-Q", Size::new(6, 6, 6), 1.0),
      HullSocket::module("_qlP8rsUpUiJpkRc1tcjPQ", Size::new(8, 3, 6), 1.0),
      HullSocket::module("RMDTRlITGUm1ZgbIgWq1Xw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("v7vnexI1xUemaSmSkn0QNA", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("KnFHQkb9uUe98IweLYf9wg", "5kd20hGcxUuTlY-6JYaudA")
    ]
  };

  pub const CARGO_FEEDER_MONITOR: Hull = Hull {
    name: "Cargo Feeder Monitor",
    save_key: "Stock/Bulk Feeder",
    faction: Faction::Protectorate,
    point_cost: 175,
    mass: 5095.0,
    max_speed: 20.0,
    max_turn_speed: 5.729578,
    linear_motor: 5.0,
    angular_motor: 1.0,
    base_integrity: 1000.0,
    armor_thickness: 48.0,
    base_crew_complement: 40,
    buffs: &[],
    sockets: &[
      HullSocket::mount("4V6twrIUcEKE11tom-tDXQ", Size::new(6, 12, 6), Fore, 1.0),
      HullSocket::mount("yN7f-9tEXEKlBNskqP6EbA", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("Z2uDs34J-ESu64-i9YGnxQ", Size::new(6, 4, 6), Down, 1.0),
      HullSocket::mount("gb6_3kWBY0KlEWeex3ruDg", Size::new(3, 2, 5), Right, 1.0),
      HullSocket::mount("_y-Bd-EWjUeMIrmzG3UxvQ", Size::new(3, 2, 5), Left, 1.0),
      HullSocket::compartment("gDQghm5iik2syYcoJqUPFg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("vtme83CXg0qDA4_YK6NqwA", Size::new(6, 3, 8), 1.0),
      HullSocket::compartment("TTM8O-MUDky06__LwvSo8w", Size::new(6, 1, 3), 1.0),
      HullSocket::compartment("rGVFgJ8GyUePoRkz33c1sw", Size::new(6, 1, 3), 1.0),
      HullSocket::compartment("_hUvj899GkWP0bnjDN1ODw", Size::new(6, 3, 8), 1.0),
      HullSocket::compartment("N4KIOsNqeUGVAUrx5OW3DQ", Size::new(6, 3, 8), 1.0),
      HullSocket::module("9K_1ZNCgFESrvaoQMOH3yw", Size::new(6, 6, 6), 1.0),
      HullSocket::module("HNv6B-78FEuhV8YT9A6uZw", Size::new(10, 5, 8), 1.0),
      HullSocket::module("HvdEBsnpUUypPN4uW67paQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("iDdlfFwKkUO9HbumzymceA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("DaqWEX-8gEeIzYrhp8NIMw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("EvZ6JN5xV0aUlKEeeKxFTQ", Size::new(2, 2, 2), 1.0),
      HullSocket::module("CU_oraqJxEaB3dTzZW2MkA", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("gb6_3kWBY0KlEWeex3ruDg", "_y-Bd-EWjUeMIrmzG3UxvQ")
    ]
  };

  pub const OCELLO_COMMAND_CRUISER: Hull = Hull {
    name: "Ocello Command Cruiser",
    save_key: "Stock/Ocello Cruiser",
    faction: Faction::Protectorate,
    point_cost: 575,
    mass: 12140.0,
    max_speed: 20.0,
    max_turn_speed: 5.729578,
    linear_motor: 14.0,
    angular_motor: 2.2,
    base_integrity: 6500.0,
    armor_thickness: 30.0,
    base_crew_complement: 80,
    buffs: &[
      (Buff::MissileProgrammingChannels, 2.0)
    ],
    sockets: &[
      HullSocket::mount("iJzzxinSN0uJIjrt9M3aGA", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("lQ7GplJ1ikW0K_QhXK7GvA", Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount("60X-7VSlxUq6Qas-KwszMA", Size::new(6, 4, 6), Down, 1.0),
      HullSocket::mount("4v5e01-gEUiTNnQJevNpGg", Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount("o78jebL2nkezXdHJ6xA3WA", Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount("Gpm2KFKPCU-0ECKr0YsJAw", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount("-lwFeyvDUECyC-AuVfATog", Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount("rI5k7ytZH0S7yQLZnFB_nQ", Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount("0J9pyeFp6U6KMEAXiuNHHw", Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount("mP5XoHZ3Wk-DsFk3YLAxzg", Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount("xe7LNTrStUuS8PMCn6KYsQ", Size::new(2, 2, 2), Right, 1.0),
      HullSocket::compartment("4NKBOsswyUGpVA-YP-T5ng", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("bzRgWNrBdE6BjUaGy8G1tg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("foWiXJZCEEKQx0LmO1IMnw", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("viC38BuOaEuyTlVDRdF0gg", Size::new(6, 1, 8), 1.0),
      HullSocket::compartment("GXIxYPfk8E2cbQnK0U1QNw", Size::new(4, 1, 6), 1.0),
      HullSocket::compartment("q5iHLt9VEU-166x4NJmr4Q", Size::new(6, 1, 6), 1.0),
      HullSocket::compartment("vXXZA02qQEyreCSFbtLJOA", Size::new(6, 1, 6), 1.0),
      HullSocket::compartment("S03aWjfX90epbCcytu9_xw", Size::new(6, 1, 6), 1.0),
      HullSocket::compartment("aImgD6jcq0GZXRi5MmyeWw", Size::new(6, 1, 6), 1.0),
      HullSocket::module("hjHxzsG5iUCkaG0eGd52bA", Size::new(6, 6, 6), 1.0),
      HullSocket::module("Fp6_NyEq1E-ZGeZxM9kjUg", Size::new(6, 6, 6), 1.0),
      HullSocket::module("-XwaOMN6eUSsztvywrmXVQ", Size::new(8, 12, 10), 1.0),
      HullSocket::module("26mxVeOdd0SB5ZSaG_hJ0w", Size::new(3, 3, 3), 1.0),
      HullSocket::module("c2zMLaokmU6GUJMYfS3N_Q", Size::new(3, 3, 3), 1.0),
      HullSocket::module("t3_3a8clbky2DQa0g_ODgg", Size::new(3, 3, 3), 1.0),
      HullSocket::module("73x1IW-ccUSI_0bIcWMFPw", Size::new(2, 2, 2), 1.0),
      HullSocket::module("LNhnTOaflUaLPxvcpEctGg", Size::new(2, 2, 2), 1.0),
      HullSocket::module("yPVEWDqw0EOqF2I5_u-ETA", Size::new(2, 2, 2), 1.0),
      HullSocket::module("FDuYw4ygU0ik6wQg6rW2Qg", Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      ("4v5e01-gEUiTNnQJevNpGg", "o78jebL2nkezXdHJ6xA3WA"),
      ("mP5XoHZ3Wk-DsFk3YLAxzg", "xe7LNTrStUuS8PMCn6KYsQ")
    ]
  };

  pub const BULK_FREIGHTER_LINE_SHIP: Hull = Hull {
    name: "Bulk Freighter Line Ship",
    save_key: "Stock/Bulk Hauler",
    faction: Faction::Protectorate,
    point_cost: 350,
    mass: 15095.0,
    max_speed: 23.0,
    max_turn_speed: 7.448451,
    linear_motor: 40.0,
    angular_motor: 7.5,
    base_integrity: 3000.0,
    armor_thickness: 20.0,
    base_crew_complement: 100,
    buffs: &[
      (Buff::MissileProgrammingChannels, 1.0),
      (Buff::RepairTeamMoveSpeed, 0.2)
    ],
    sockets: &[
      // Depending on the bulk freighter's hull config, mounts 14 and 15 may be oriented differently
      HullSocket::mount("E9IMUD_Uz0661-nYgJh4Mg", Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount("auQ7ijGeX0evdikZQeBxTg", Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount("d3N-B_JnDUKAdVRBJ6nvFQ", Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount("xvNT1_DYcUuSpU6rrmL_4A", Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount("t0T4CSAPnEq4rvHAVl84-Q", Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount("grYBjxIS3E--lmrqenuK8A", Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount("8pltGexHAEyFL2ljOe0oMQ", Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount("tcQfj6XOHUWFSAx_W0cc1A", Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount("bxvsvg5znk6TqiIKq8H0kg", Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount("9EF9u0V1eUetleBnkQhHkA", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount("I68tNng-K0yG-AoUx0HcSw", Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount("Sb5-VSuwlkWpT6ViRiJ3vg", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount("6ZiODnoXqkaNWvaqyGjnwg", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount_unknown("87PSgg9GREmGU9MIRAXXhg", Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown("VDQhZPo-rkWkzwozat27fw", Size::new(3, 4, 3), 1.0),
      HullSocket::compartment("y1Cs8LR0KUyCKUAJBIOyOg", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("fYoX2amTl0KrZurPKjU9gg", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("3Ky8PvdxqUKMRXZa0xLF2A", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("T9TN0ZYumUaCxcSRjO_qTw", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("T4d-8WnA_UiHSJh6PXiXpA", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("6LpDL27QuEW0PZCm35Nx5w", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("QIOFQpQSKESIdNzf_JEFyA", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("FVcPIcV_00eu62NQQK3oLg", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("1FrTJ5o850ynlq4dRDmMbA", Size::new(8, 1, 8), 1.0),
      HullSocket::module("vOzzHo4KfE60wZ3Q1s38GA", Size::new(12, 12, 12), 1.0),
      HullSocket::module("6evWdeLGT0uALe075-nhgw", Size::new(6, 6, 6), 1.0),
      HullSocket::module("JVCLf3RKKUOBEXaWWkVLMw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("JHJi2y5Xskytpw5-Qbu95A", Size::new(3, 3, 3), 1.0),
      HullSocket::module("yryHYXgMWEy969gRuaDO6w", Size::new(3, 3, 3), 1.0),
      HullSocket::module("M3mc7A4WOU-2fq5XgRSECA", Size::new(3, 3, 3), 1.0),
      HullSocket::module("S-rQo19r20K0L0PRer3yMw", Size::new(3, 3, 3), 1.0),
      HullSocket::module("MQyTgOLGCkyeljdvbkkFDA", Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      ("E9IMUD_Uz0661-nYgJh4Mg", "auQ7ijGeX0evdikZQeBxTg"),
      ("d3N-B_JnDUKAdVRBJ6nvFQ", "xvNT1_DYcUuSpU6rrmL_4A"),
      ("t0T4CSAPnEq4rvHAVl84-Q", "grYBjxIS3E--lmrqenuK8A"),
      ("8pltGexHAEyFL2ljOe0oMQ", "tcQfj6XOHUWFSAx_W0cc1A"),
      ("Sb5-VSuwlkWpT6ViRiJ3vg", "6ZiODnoXqkaNWvaqyGjnwg")
    ]
  };

  pub const CONTAINER_LINER_LINE_SHIP: Hull = Hull {
    name: "Container Liner Line Ship",
    save_key: "Stock/Container Hauler",
    faction: Faction::Protectorate,
    point_cost: 1000,
    mass: 15095.0,
    max_speed: 20.0,
    max_turn_speed: 4.0107045,
    linear_motor: 40.0,
    angular_motor: 6.0,
    base_integrity: 3000.0,
    armor_thickness: 20.0,
    base_crew_complement: 100,
    buffs: &[
      (Buff::MissileProgrammingChannels, 3.0)
    ],
    sockets: &[
      HullSocket::mount_unknown("uLHDwjLFekuYaY3h0JwZoQ", Size::new(20, 5, 30), 1.0),
      HullSocket::mount_unknown("44SMwZSbRkOQyQmJ458Y1Q", Size::new(20, 5, 30), 1.0),
      HullSocket::mount_unknown("VkODw--0zk-K4FkajpPjwQ", Size::new(20, 5, 20), 1.0),
      HullSocket::mount_unknown("F-nHNZm4Z0WZMJakJAwlPw", Size::new(20, 5, 20), 1.0),
      HullSocket::mount("y70HHuLWd0uphO2H-hvZGQ", Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount("NjfVPFfJqUmkw-AR_2KBjg", Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount_unknown("TcS59cU8X0aEm3ds7Nqixg", Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown("3WEnDKibY0-WJvtq9-tmuA", Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown("6HbbAtLFGUOi-kq06ErQdA", Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown("bF4isjieO0m0PsOlE-zgkQ", Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown("KuKTAiCM50uIV7XV1S8wrw", Size::new(3, 4, 3), 1.0),
      HullSocket::compartment("y1Cs8LR0KUyCKUAJBIOyOg", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("iWiBr9uCyUCBub-IIW2bZg", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("QhdtovcTk0WSpMAUsBIF6Q", Size::new(8, 3, 8), 1.0),
      HullSocket::compartment("T9TN0ZYumUaCxcSRjO_qTw", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("T4d-8WnA_UiHSJh6PXiXpA", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("hs-SaPpehUu8ZbwsuJSvXw", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("jhY0RlqxLkW69clFCGXELA", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("OE8qVccIYEGwfvJXsiFgoA", Size::new(8, 1, 8), 1.0),
      HullSocket::compartment("UAmAj39M3E2zPnVXF46kZA", Size::new(8, 1, 8), 1.0),
      HullSocket::module("yxnQxeuUHUO22pm-LQu4Cg", Size::new(12, 12, 12), 1.0),
      HullSocket::module("_uEQzHKRzEqqIqa1c8wJcw", Size::new(6, 6, 6), 1.0),
      HullSocket::module("M0ewvimLKU2vXoznZNDoYQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("cXpBal32p0-R2pA6hN056w", Size::new(3, 3, 3), 1.0),
      HullSocket::module("OY726V7rAUKDR92MQM7gnQ", Size::new(3, 3, 3), 1.0),
      HullSocket::module("SmUC0Jc9FEuCeKc_ZJMxIg", Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      ("uLHDwjLFekuYaY3h0JwZoQ", "44SMwZSbRkOQyQmJ458Y1Q"),
      ("VkODw--0zk-K4FkajpPjwQ", "F-nHNZm4Z0WZMJakJAwlPw"),
      ("y70HHuLWd0uphO2H-hvZGQ", "NjfVPFfJqUmkw-AR_2KBjg")
    ]
  };
}
