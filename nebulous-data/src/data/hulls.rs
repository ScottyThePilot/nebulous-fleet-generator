use super::components::ComponentKind;
use super::{Buff, Direction, Faction};
use crate::format::key::Key;
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
  pub socket_symmetries: &'static [(Key, Key)]
}

#[derive(Debug, Clone, Copy)]
pub struct HullSocket {
  pub save_key: Key,
  pub kind: ComponentKind,
  pub size: Size,
  pub direction: Option<Direction>,
  pub desirability: f32
}

impl HullSocket {
  #[inline]
  const fn mount(save_key: Key, size: Size, direction: Direction, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Mount, size, direction: Some(direction), desirability }
  }

  #[inline]
  const fn mount_unknown(save_key: Key, size: Size, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Mount, size, direction: None, desirability }
  }

  #[inline]
  const fn compartment(save_key: Key, size: Size, desirability: f32) -> Self {
    HullSocket { save_key, kind: ComponentKind::Compartment, size, direction: None, desirability }
  }

  #[inline]
  const fn module(save_key: Key, size: Size, desirability: f32) -> Self {
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
      HullSocket::mount(key!("wDsRnL5nKkyYvKgD6VcPHg"), Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount(key!("Z48ot_dQfkWb6AVYjaM_gA"), Size::new(3, 2, 3), Down, 1.0),
      HullSocket::mount(key!("IUdNSVZm2Eu9n3F5HnSAng"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount(key!("ZxY9ONYz80SiLNSObvjNzQ"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment(key!("XPaYCjBqdEqBxEIsVLP0oA"), Size::new(4, 1, 8), 1.0),
      HullSocket::compartment(key!("xRYIkvssd0mPY3VIgrnQ5A"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("rXO8xwG1MkqzI_2pU-O_qQ"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("GefqwCQzg0qXA3EFRmDtPw"), Size::new(3, 1, 3), 1.0),
      HullSocket::module(key!("4WpJyiOKVEqCwR99l47MkA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("4EHx4mQhNUi-WuuQtQF5fA"), Size::new(8, 3, 6), 1.0),
      HullSocket::module(key!("TSPPW9ECe06-MGzR1i0WvQ"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("MBOvGazj6UWpt2OOy44s7w"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("IUdNSVZm2Eu9n3F5HnSAng"), key!("ZxY9ONYz80SiLNSObvjNzQ")),
      (key!("TSPPW9ECe06-MGzR1i0WvQ"), key!("MBOvGazj6UWpt2OOy44s7w"))
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
      HullSocket::mount(key!("PDKmGfvpykODc3XHDQ7WBw"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount(key!("WwMGqYiU7E6lp7ID47phqA"), Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount(key!("gkfIwYzn7kGhG6MW9uxBmw"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("D3zoB0PetEC973iUck_mNQ"), Size::new(3, 4, 5), Down, 1.0),
      HullSocket::compartment(key!("98N-YI_1WUOs--qPYlt-7g"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("6Tzo7268MEqgyFxnGvLKUg"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("O_Y-AJc7r0alOg36rvY6MQ"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("5APOD4UfSkGdkm9WWdAvLA"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("lorATFhkTkeZQYThYOHazg"), Size::new(4, 1, 6), 1.0),
      HullSocket::module(key!("bRgUlaoQJ0S7M91zMpdAdA"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("Jmtnwo0KQki5QyEPPlBHrA"), Size::new(8, 3, 6), 1.0),
      HullSocket::module(key!("V42tXibIR0e4u6riIHYZOw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("p-m7ijS6ukuqzuBT5NE_lA"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("foApM84hT0GpEw4GaFv38w"), Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      (key!("V42tXibIR0e4u6riIHYZOw"), key!("p-m7ijS6ukuqzuBT5NE_lA"))
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
      HullSocket::mount(key!("NpYYAkA5Z0uAElOjDg3Rag"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("fSYv4j5-eEObwJEVhoBSvg"), Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount(key!("x21sadthgE2FtDWSJKINGQ"), Size::new(3, 4, 4), Left, 1.0),
      HullSocket::mount(key!("IxKwpY5d9EicYRRTMn8xVw"), Size::new(3, 4, 4), Right, 1.0),
      HullSocket::mount(key!("JpOU9MgWXEmfLcQoAK4otA"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount(key!("LCosRblddUCjPogeJ5eSeA"), Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount(key!("hGeid9yk80GqIN5IUlp8aw"), Size::new(4, 12, 4), Fore, 1.0),
      HullSocket::compartment(key!("PlyaCcjDo0qu155JIZgm6A"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("ytRGy84X_kW1odHYCBICDQ"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("yM3Am4PqyUOIByTKzM6sPA"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("H1ekC2-9c02VyNXY4IeLYA"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("ln5c_ZvdCUSMVWZL_-m3yw"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("H-oABJbYw0qgKd4QzU0WaA"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("tCBEdcIUhk6AL-X6sCujsQ"), Size::new(6, 1, 4), 1.0),
      HullSocket::module(key!("AWYvQUZ26k20yHOrVQKg6Q"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("tmHTz6HkrE-RUtN94W6lyQ"), Size::new(8, 8, 6), 1.0),
      HullSocket::module(key!("m5J1jQdOfUKM9r7NZjMpKA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("h29TSNGRo0m_DmeRr2BqDw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("E9sMCEquVk-NNj5heGmIlQ"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("gx2UKetKWUm5BfZhiD5phQ"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("ZJTmbGdjAk-XEKihIYHEuw"), Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      (key!("x21sadthgE2FtDWSJKINGQ"), key!("IxKwpY5d9EicYRRTMn8xVw"))
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
      HullSocket::mount(key!("T9Ebo41iA0eBXNYx8uyisw"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("2QQdxC4UE0KOM42r82-ETQ"), Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount(key!("IFKM9E04aUaHS0IoNDMShA"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("rX6hes7UqkyHjEyKbFaWTQ"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("BRMwuusKC02YrbFXRrrNzA"), Size::new(3, 4, 5), Up, 1.0),
      HullSocket::mount(key!("vO1oPhlSuUih_cdAZk3Hqg"), Size::new(3, 4, 5), Down, 1.0),
      HullSocket::mount(key!("RLHQUFf200uLZjKX5axkMw"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount(key!("uyPDg0tD3U6YKz18bVdkPg"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount(key!("Xicf0TT7pEaFy_x1uk7ueQ"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount(key!("XTg1H1Popku5gxW8sei5XQ"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment(key!("a8H-rPyVSk6uBOrP7lQYUA"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("NBbnpHfpDUSS6bNgDlSjzw"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("f_SO68qAGU-sw69T9GEWow"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("9R0tfjsN-kiETUBLeCCmGw"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("-bAF6R6aiU2Afn8T_oKY4g"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("-7H3WGDwyESDI3lkHzBdGQ"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("o1o5WOogUE255YVi43KlcQ"), Size::new(4, 1, 8), 1.0),
      HullSocket::compartment(key!("DAcDeA0yskSjje92CY-F7w"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("jKvCg9nYmkKfAn4-ht_Y6A"), Size::new(6, 1, 8), 1.0),
      HullSocket::module(key!("8VKMmbA23Em1f-MBZKO2vA"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("udQjNCWlA0awkv6LhH55uQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("3eXWUksWXk-5q9MW52rghw"), Size::new(8, 12, 10), 1.0),
      HullSocket::module(key!("_jqFwgf3EkKUUqHzVWidzw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("qulLVFUtzk2Qm8rZtfpFDw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("N_gGUWCQx0mUOX9RaFoI6A"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("E1ZSbpYpZkufCyifYiiK8Q"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("-hETYcmKH0eeXzkFpnhgeg"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("hU2L93VfQU-jutnRC0dKgw"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("RLHQUFf200uLZjKX5axkMw"), key!("uyPDg0tD3U6YKz18bVdkPg")),
      (key!("Xicf0TT7pEaFy_x1uk7ueQ"), key!("XTg1H1Popku5gxW8sei5XQ"))
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
      HullSocket::mount(key!("whDP9rtbukKsocnrqnqaLQ"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("8CtCKPLfZEOOFvm1DhV-oQ"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("N7yRYYUuG0uOzd5aufgViA"), Size::new(8, 7, 8), Down, 1.0),
      HullSocket::mount(key!("lYEQo4jJvEGxRqJa2MB25A"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("9qreGP2Iw0KwuloNpNFTSg"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("v9Sqa5DcCkibdi29AkakNg"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("3MoeYUx1HUS6xNDBMsssig"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("LbD9Txe-S0a_nODTqtrk7g"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount(key!("KxV0hkkGBE2AOUCo5axQbg"), Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount(key!("tyGXrucCjUe0FVyP1YDpUw"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount(key!("rXZhdtbK3EC8K_cu6-r_Xg"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::compartment(key!("ylUvmASKlEiyCtyCsNRKWg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("TX2TktZe7EWB9s-MqlDQgA"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("cMs0g0g3WUeZJQASb7omew"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("iHYg3DdYAUKS-uIBaRpNqA"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("j2ST2yYfwk2_tBIayI3sQg"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("SV_4BgcxwU24YIbep-k74g"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("A8e_Z_d8WU2qzo6HTQwCBA"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("2xmOS4ns_EaMYjBXrrhhig"), Size::new(6, 1, 6), 1.0),
      HullSocket::compartment(key!("y1xLixKUzUSKT7Nz7aUkLQ"), Size::new(6, 1, 6), 1.0),
      HullSocket::compartment(key!("RmUeePb-8EGbFaRgVjCXsA"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("eYp1IPfupUGVITrkVhgZJw"), Size::new(4, 1, 6), 1.0),
      HullSocket::module(key!("lxkgmNwqjUua0sRee2pDJg"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("_Aun-HW-DkOGytVmYSn5IQ"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("2ZLEJvxYf0CATl1lBi3cuQ"), Size::new(12, 8, 10), 1.0),
      HullSocket::module(key!("ba-3VQ-zGEmDCfU12uaqaA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("hQ-Mzyns0UO4oMp7gG0d8A"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("oyAxfnepkEG46VjtvQWXpw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("Yo9hPw6Mf060B3vqwSO4GA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("CgpRQa-660KEed1wTRL4vw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("awwdeDnNs0G-RceQF1TByA"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("tnZD-B5Ls0qAyBtdz4yoTA"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("S8ALcpOMYEqJog1v2q8UMQ"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("rQWETsDGdE-AR8N3NPfwXg"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("lYEQo4jJvEGxRqJa2MB25A"), key!("v9Sqa5DcCkibdi29AkakNg")),
      (key!("9qreGP2Iw0KwuloNpNFTSg"), key!("3MoeYUx1HUS6xNDBMsssig")),
      (key!("tyGXrucCjUe0FVyP1YDpUw"), key!("rXZhdtbK3EC8K_cu6-r_Xg"))
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
      HullSocket::mount(key!("vvpl_pV8B0W33ernrw31jg"), Size::new(8, 7, 8), Up, 1.0),
      HullSocket::mount(key!("AznvpY2By0GiQxgeOTktww"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("N5nMrtk4bkiSDzOffAmI1w"), Size::new(8, 7, 8), Up, 1.0),
      HullSocket::mount(key!("HsGDhCR_T0KH1ddzUnSHmA"), Size::new(8, 7, 8), Down, 1.0),
      HullSocket::mount(key!("oRZYmSFdsk2BFB03PqyS2w"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("csaNZW85vkO8uIQ9XDe1iw"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("eBG0UhrBiky4nUx81rPbLw"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("M6NaSvLam06C-1DTggI99g"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount(key!("3HKNGt305UC23fGT8OKcLw"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("R_Y-LTCc0kydl6L8Sq5Dmw"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("-gtO0aqGDEOH-Tm612wAoA"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("37oZ894X7kG2Dq0E5mEKDQ"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount(key!("ZjH6aPXnl0eOdklaPsRNAA"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount(key!("4dNKr5i87EeDGq9coaWYJg"), Size::new(2, 2, 2), Down, 1.0),
      HullSocket::compartment(key!("XAbGPXPgDEuDieQDqbA9MQ"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("SX-a4J4eek2nmJztXePU3Q"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("4sUWwP0VukOFbXT1YAMa9g"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("09AenxIgokykwMTCMJz24w"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("RtZe85sJUkO5b1xVdgPlqQ"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("8YQv6H5fN0SylDc4p2fT-w"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("6P8ohHGH_UaCAMEE5jFoRg"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("80T00-71i0Wz-uNk04dzvQ"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("GbGNS6ZFo0qx-dxuEmZTbg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("tkt9JPA2V0CWAO-Dfj8FHQ"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("QkO76Z2PAUq8iD4cuc_P4A"), Size::new(6, 1, 4), 1.0),
      HullSocket::compartment(key!("lp31cinWLE-ZGgtisuoaug"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("wbgVupdnzEGle6f8tezZYQ"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("Jl5HmssP406Au7n4NHceQw"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("-wa5U_hdw0mHKI1PglDeGg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("97l_f_ktb0yoEquIa75icg"), Size::new(6, 1, 8), 1.0),
      HullSocket::module(key!("Cz_cm8aRt0mRiYxdq26V7A"), Size::new(8, 8, 8), 1.0),
      HullSocket::module(key!("eOvmJ625f0yGavYASYtYYQ"), Size::new(8, 8, 8), 1.0),
      HullSocket::module(key!("1I5WRAX0oU6zIv7yEvqfmQ"), Size::new(16, 8, 10), 1.0),
      HullSocket::module(key!("98aZYYnrXUWKXheUXHoLLw"), Size::new(12, 8, 10), 1.0),
      HullSocket::module(key!("cVsLdxme_kGSU2jWlYnXzg"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("rWWXZNnbKUi7djDPaKTttQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("tgWD8fbOBEWq4KfvYDN_BQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("4wfe6HbB9kCN7CDcF7_POQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("yAg2wUCkJ0a6Ef3m96WyYw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("rTbYkycwWku-Ye1FCBJbvw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("-ao2fBk-zkK3QTdVH4fqHA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("5lOFQqLf-kOQq0zbIv0MKw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("PWWRnNTo60Cx9faCbUhx-g"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("31ilgFi2f0ScvJbFia-Opg"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("MjJFzcmp2kCLRWmjC30TKA"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("oRZYmSFdsk2BFB03PqyS2w"), key!("3HKNGt305UC23fGT8OKcLw")),
      (key!("csaNZW85vkO8uIQ9XDe1iw"), key!("R_Y-LTCc0kydl6L8Sq5Dmw")),
      (key!("eBG0UhrBiky4nUx81rPbLw"), key!("-gtO0aqGDEOH-Tm612wAoA")),
      (key!("M6NaSvLam06C-1DTggI99g"), key!("37oZ894X7kG2Dq0E5mEKDQ"))
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
      HullSocket::mount(key!("3mlCA6C6m0GI3wONZK_aqg"), Size::new(3, 2, 3), Up, 1.0),
      HullSocket::mount(key!("fbAgHM9u302peFMSC0TidQ"), Size::new(3, 2, 3), Down, 1.0),
      HullSocket::mount(key!("rbZhGCN7UUWcgLsNZkZXIA"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::compartment(key!("hh8PA01jxEytn9KuihcJ3Q"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("wDWAeeoAK0mAx3hbF1tMlA"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("CcSXjIupV0Kw-jxaV6HpEA"), Size::new(3, 1, 3), 1.0),
      HullSocket::module(key!("bInyh5AAt0y_m-3UaI8Ysw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("-harMwD0o0OExuMqT63fTQ"), Size::new(8, 3, 6), 1.0),
      HullSocket::module(key!("zUeDN4FpYEyyWHIhTIgZFg"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("uXJ_fvsZ3USAs1SpiTrZ1g"), Size::new(2, 2, 2), 1.0)
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
      HullSocket::mount(key!("03vxralGq0CFx1XKxjOcBA"), Size::new(2, 5, 2), Fore, 1.0),
      HullSocket::mount(key!("cIGspM4oYU2etRfjIcuCCA"), Size::new(3, 2, 5), Up, 1.0),
      HullSocket::mount(key!("0upOgm0H0UuZ-h2YvRjmBg"), Size::new(3, 2, 5), Down, 1.0),
      HullSocket::mount(key!("KnFHQkb9uUe98IweLYf9wg"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::mount(key!("5kd20hGcxUuTlY-6JYaudA"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::compartment(key!("F6bmJdCO3EqzXK9SBJoXUw"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("-PqoLKimSUOZtKMLo5QHXg"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("DakKS2gy6UyM58u3YG-HfQ"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("peqaYrE39UupWCXPY_PgoA"), Size::new(3, 1, 3), 1.0),
      HullSocket::compartment(key!("6OieSH0A6kKFmquctn9_aQ"), Size::new(4, 1, 6), 1.0),
      HullSocket::module(key!("NSAbhn9brE-aAirJFIYG-Q"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("_qlP8rsUpUiJpkRc1tcjPQ"), Size::new(8, 3, 6), 1.0),
      HullSocket::module(key!("RMDTRlITGUm1ZgbIgWq1Xw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("v7vnexI1xUemaSmSkn0QNA"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("KnFHQkb9uUe98IweLYf9wg"), key!("5kd20hGcxUuTlY-6JYaudA"))
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
      HullSocket::mount(key!("4V6twrIUcEKE11tom-tDXQ"), Size::new(6, 12, 6), Fore, 1.0),
      HullSocket::mount(key!("yN7f-9tEXEKlBNskqP6EbA"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("Z2uDs34J-ESu64-i9YGnxQ"), Size::new(6, 4, 6), Down, 1.0),
      HullSocket::mount(key!("gb6_3kWBY0KlEWeex3ruDg"), Size::new(3, 2, 5), Right, 1.0),
      HullSocket::mount(key!("_y-Bd-EWjUeMIrmzG3UxvQ"), Size::new(3, 2, 5), Left, 1.0),
      HullSocket::compartment(key!("gDQghm5iik2syYcoJqUPFg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("vtme83CXg0qDA4_YK6NqwA"), Size::new(6, 3, 8), 1.0),
      HullSocket::compartment(key!("TTM8O-MUDky06__LwvSo8w"), Size::new(6, 1, 3), 1.0),
      HullSocket::compartment(key!("rGVFgJ8GyUePoRkz33c1sw"), Size::new(6, 1, 3), 1.0),
      HullSocket::compartment(key!("_hUvj899GkWP0bnjDN1ODw"), Size::new(6, 3, 8), 1.0),
      HullSocket::compartment(key!("N4KIOsNqeUGVAUrx5OW3DQ"), Size::new(6, 3, 8), 1.0),
      HullSocket::module(key!("9K_1ZNCgFESrvaoQMOH3yw"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("HNv6B-78FEuhV8YT9A6uZw"), Size::new(10, 5, 8), 1.0),
      HullSocket::module(key!("HvdEBsnpUUypPN4uW67paQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("iDdlfFwKkUO9HbumzymceA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("DaqWEX-8gEeIzYrhp8NIMw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("EvZ6JN5xV0aUlKEeeKxFTQ"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("CU_oraqJxEaB3dTzZW2MkA"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("gb6_3kWBY0KlEWeex3ruDg"), key!("_y-Bd-EWjUeMIrmzG3UxvQ"))
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
      HullSocket::mount(key!("iJzzxinSN0uJIjrt9M3aGA"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("lQ7GplJ1ikW0K_QhXK7GvA"), Size::new(6, 4, 6), Up, 1.0),
      HullSocket::mount(key!("60X-7VSlxUq6Qas-KwszMA"), Size::new(6, 4, 6), Down, 1.0),
      HullSocket::mount(key!("4v5e01-gEUiTNnQJevNpGg"), Size::new(3, 4, 3), Left, 1.0),
      HullSocket::mount(key!("o78jebL2nkezXdHJ6xA3WA"), Size::new(3, 4, 3), Right, 1.0),
      HullSocket::mount(key!("Gpm2KFKPCU-0ECKr0YsJAw"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount(key!("-lwFeyvDUECyC-AuVfATog"), Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount(key!("rI5k7ytZH0S7yQLZnFB_nQ"), Size::new(2, 2, 2), Up, 1.0),
      HullSocket::mount(key!("0J9pyeFp6U6KMEAXiuNHHw"), Size::new(2, 2, 2), Down, 1.0),
      HullSocket::mount(key!("mP5XoHZ3Wk-DsFk3YLAxzg"), Size::new(2, 2, 2), Left, 1.0),
      HullSocket::mount(key!("xe7LNTrStUuS8PMCn6KYsQ"), Size::new(2, 2, 2), Right, 1.0),
      HullSocket::compartment(key!("4NKBOsswyUGpVA-YP-T5ng"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("bzRgWNrBdE6BjUaGy8G1tg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("foWiXJZCEEKQx0LmO1IMnw"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("viC38BuOaEuyTlVDRdF0gg"), Size::new(6, 1, 8), 1.0),
      HullSocket::compartment(key!("GXIxYPfk8E2cbQnK0U1QNw"), Size::new(4, 1, 6), 1.0),
      HullSocket::compartment(key!("q5iHLt9VEU-166x4NJmr4Q"), Size::new(6, 1, 6), 1.0),
      HullSocket::compartment(key!("vXXZA02qQEyreCSFbtLJOA"), Size::new(6, 1, 6), 1.0),
      HullSocket::compartment(key!("S03aWjfX90epbCcytu9_xw"), Size::new(6, 1, 6), 1.0),
      HullSocket::compartment(key!("aImgD6jcq0GZXRi5MmyeWw"), Size::new(6, 1, 6), 1.0),
      HullSocket::module(key!("hjHxzsG5iUCkaG0eGd52bA"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("Fp6_NyEq1E-ZGeZxM9kjUg"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("-XwaOMN6eUSsztvywrmXVQ"), Size::new(8, 12, 10), 1.0),
      HullSocket::module(key!("26mxVeOdd0SB5ZSaG_hJ0w"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("c2zMLaokmU6GUJMYfS3N_Q"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("t3_3a8clbky2DQa0g_ODgg"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("73x1IW-ccUSI_0bIcWMFPw"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("LNhnTOaflUaLPxvcpEctGg"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("yPVEWDqw0EOqF2I5_u-ETA"), Size::new(2, 2, 2), 1.0),
      HullSocket::module(key!("FDuYw4ygU0ik6wQg6rW2Qg"), Size::new(2, 2, 2), 1.0)
    ],
    socket_symmetries: &[
      (key!("4v5e01-gEUiTNnQJevNpGg"), key!("o78jebL2nkezXdHJ6xA3WA")),
      (key!("mP5XoHZ3Wk-DsFk3YLAxzg"), key!("xe7LNTrStUuS8PMCn6KYsQ"))
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
      HullSocket::mount(key!("E9IMUD_Uz0661-nYgJh4Mg"), Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount(key!("auQ7ijGeX0evdikZQeBxTg"), Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount(key!("d3N-B_JnDUKAdVRBJ6nvFQ"), Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount(key!("xvNT1_DYcUuSpU6rrmL_4A"), Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount(key!("t0T4CSAPnEq4rvHAVl84-Q"), Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount(key!("grYBjxIS3E--lmrqenuK8A"), Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount(key!("8pltGexHAEyFL2ljOe0oMQ"), Size::new(6, 10, 6), Left, 1.0),
      HullSocket::mount(key!("tcQfj6XOHUWFSAx_W0cc1A"), Size::new(6, 10, 6), Right, 1.0),
      HullSocket::mount(key!("bxvsvg5znk6TqiIKq8H0kg"), Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount(key!("9EF9u0V1eUetleBnkQhHkA"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount(key!("I68tNng-K0yG-AoUx0HcSw"), Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount(key!("Sb5-VSuwlkWpT6ViRiJ3vg"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount(key!("6ZiODnoXqkaNWvaqyGjnwg"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount_unknown(key!("87PSgg9GREmGU9MIRAXXhg"), Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown(key!("VDQhZPo-rkWkzwozat27fw"), Size::new(3, 4, 3), 1.0),
      HullSocket::compartment(key!("y1Cs8LR0KUyCKUAJBIOyOg"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("fYoX2amTl0KrZurPKjU9gg"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("3Ky8PvdxqUKMRXZa0xLF2A"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("T9TN0ZYumUaCxcSRjO_qTw"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("T4d-8WnA_UiHSJh6PXiXpA"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("6LpDL27QuEW0PZCm35Nx5w"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("QIOFQpQSKESIdNzf_JEFyA"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("FVcPIcV_00eu62NQQK3oLg"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("1FrTJ5o850ynlq4dRDmMbA"), Size::new(8, 1, 8), 1.0),
      HullSocket::module(key!("vOzzHo4KfE60wZ3Q1s38GA"), Size::new(12, 12, 12), 1.0),
      HullSocket::module(key!("6evWdeLGT0uALe075-nhgw"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("JVCLf3RKKUOBEXaWWkVLMw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("JHJi2y5Xskytpw5-Qbu95A"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("yryHYXgMWEy969gRuaDO6w"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("M3mc7A4WOU-2fq5XgRSECA"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("S-rQo19r20K0L0PRer3yMw"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("MQyTgOLGCkyeljdvbkkFDA"), Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      (key!("E9IMUD_Uz0661-nYgJh4Mg"), key!("auQ7ijGeX0evdikZQeBxTg")),
      (key!("d3N-B_JnDUKAdVRBJ6nvFQ"), key!("xvNT1_DYcUuSpU6rrmL_4A")),
      (key!("t0T4CSAPnEq4rvHAVl84-Q"), key!("grYBjxIS3E--lmrqenuK8A")),
      (key!("8pltGexHAEyFL2ljOe0oMQ"), key!("tcQfj6XOHUWFSAx_W0cc1A")),
      (key!("Sb5-VSuwlkWpT6ViRiJ3vg"), key!("6ZiODnoXqkaNWvaqyGjnwg"))
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
      HullSocket::mount_unknown(key!("uLHDwjLFekuYaY3h0JwZoQ"), Size::new(20, 5, 30), 1.0),
      HullSocket::mount_unknown(key!("44SMwZSbRkOQyQmJ458Y1Q"), Size::new(20, 5, 30), 1.0),
      HullSocket::mount_unknown(key!("VkODw--0zk-K4FkajpPjwQ"), Size::new(20, 5, 20), 1.0),
      HullSocket::mount_unknown(key!("F-nHNZm4Z0WZMJakJAwlPw"), Size::new(20, 5, 20), 1.0),
      HullSocket::mount(key!("y70HHuLWd0uphO2H-hvZGQ"), Size::new(3, 4, 3), Up, 1.0),
      HullSocket::mount(key!("NjfVPFfJqUmkw-AR_2KBjg"), Size::new(3, 4, 3), Down, 1.0),
      HullSocket::mount_unknown(key!("TcS59cU8X0aEm3ds7Nqixg"), Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown(key!("3WEnDKibY0-WJvtq9-tmuA"), Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown(key!("6HbbAtLFGUOi-kq06ErQdA"), Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown(key!("bF4isjieO0m0PsOlE-zgkQ"), Size::new(3, 4, 3), 1.0),
      HullSocket::mount_unknown(key!("KuKTAiCM50uIV7XV1S8wrw"), Size::new(3, 4, 3), 1.0),
      HullSocket::compartment(key!("y1Cs8LR0KUyCKUAJBIOyOg"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("iWiBr9uCyUCBub-IIW2bZg"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("QhdtovcTk0WSpMAUsBIF6Q"), Size::new(8, 3, 8), 1.0),
      HullSocket::compartment(key!("T9TN0ZYumUaCxcSRjO_qTw"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("T4d-8WnA_UiHSJh6PXiXpA"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("hs-SaPpehUu8ZbwsuJSvXw"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("jhY0RlqxLkW69clFCGXELA"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("OE8qVccIYEGwfvJXsiFgoA"), Size::new(8, 1, 8), 1.0),
      HullSocket::compartment(key!("UAmAj39M3E2zPnVXF46kZA"), Size::new(8, 1, 8), 1.0),
      HullSocket::module(key!("yxnQxeuUHUO22pm-LQu4Cg"), Size::new(12, 12, 12), 1.0),
      HullSocket::module(key!("_uEQzHKRzEqqIqa1c8wJcw"), Size::new(6, 6, 6), 1.0),
      HullSocket::module(key!("M0ewvimLKU2vXoznZNDoYQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("cXpBal32p0-R2pA6hN056w"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("OY726V7rAUKDR92MQM7gnQ"), Size::new(3, 3, 3), 1.0),
      HullSocket::module(key!("SmUC0Jc9FEuCeKc_ZJMxIg"), Size::new(3, 3, 3), 1.0)
    ],
    socket_symmetries: &[
      (key!("uLHDwjLFekuYaY3h0JwZoQ"), key!("44SMwZSbRkOQyQmJ458Y1Q")),
      (key!("VkODw--0zk-K4FkajpPjwQ"), key!("F-nHNZm4Z0WZMJakJAwlPw")),
      (key!("y70HHuLWd0uphO2H-hvZGQ"), key!("NjfVPFfJqUmkw-AR_2KBjg"))
    ]
  };
}
