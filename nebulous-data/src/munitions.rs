use super::MissileSize;

use std::str::FromStr;



#[derive(Debug, Clone, Copy)]
pub struct Munition {
  pub name: &'static str,
  pub save_key: &'static str,
  pub role: WeaponRole,
  pub family: MunitionFamily,
  pub point_cost: u32,
  pub point_division: u32,
  pub storage_volume: f32,
  pub flight_speed: f32,
  pub max_range: f32,
  pub variant: MunitionVariant
}

#[derive(Debug, Clone, Copy)]
pub enum MunitionVariant {
  Shell {
    damage: MunitionDamage
  },
  Missile {
    size: MissileSize,
    damage: MunitionDamage
  },
  MissileOther {
    size: MissileSize
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MunitionDamage {
  pub armor_penetration: f32,
  pub component_damage: f32,
  pub overpenetration_damage_multiplier: f32,
  pub max_penetration_depth: Option<f32>,
  pub can_ricochet: bool
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WeaponRole {
  Offensive,
  Defensive,
  DualPurpose,
  Utility,
  Decoy
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MunitionFamily {
  BallisticMagnetic15mm,
  BallisticChemical20mm,
  BallisticChemical50mmFlak,
  BallisticChemical100mm,
  BallisticChemical120mm,
  BallisticChemical250mm,
  BallisticMagnetic300mmRailgun,
  BallisticMagnetic400mmPlasma,
  BallisticChemical450mm,
  BallisticMagnetic500mmMassDriver,
  BallisticChemical600mm,
  StandardMissile,
  ContainerMissile,
  LoiteringMine,
  UnguidedRocket,
  Infinite
}

impl MunitionFamily {
  pub fn keys(self) -> impl Iterator<Item = MunitionKey> + DoubleEndedIterator + Clone {
    MunitionKey::VALUES.iter().copied().filter(move |&key| key.munition().family == self)
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MunitionKey {
  A15mmSandshot,
  A20mmSlug,
  A100mmAP,
  A100mmGrape,
  A100mmHE,
  A100mmHEHC,
  A120mmAP,
  A120mmHE,
  A120mmHERPF,
  A250mmAP,
  A250mmHE,
  A250mmHERPF,
  A300mmAPRailSabot,
  A400mmPlasmaAmpoule,
  A450mmAP,
  A450mmHE,
  A500mmFracturingBlock,
  A600mmBomb,
  A600mmHESH,
  CM4D1DecoyContainerClipper,
  CM4D2DecoyContainerLineShip,
  CM4MMineContainer,
  CM4R12RocketContainer,
  CM4R6RocketContainer,
  EA12ChaffDecoy,
  EA20FlareDecoy,
  EA99ActiveDecoy,
  FlakRound,
  M30MattockMine,
  M30NMattockCooperativeMine,
  M50AugerSprintMine,
  R2PiranhaRocket
}

impl MunitionKey {
  pub const fn save_key(self) -> &'static str {
    self.munition().save_key
  }

  pub const fn munition(self) -> &'static Munition {
    use self::list::*;

    match self {
      Self::A15mmSandshot => &A15MM_SANDSHOT,
      Self::A20mmSlug => &A20MM_SLUG,
      Self::A100mmAP => &A100MM_AP,
      Self::A100mmGrape => &A100MM_GRAPE,
      Self::A100mmHE => &A100MM_HE,
      Self::A100mmHEHC => &A100MM_HEHC,
      Self::A120mmAP => &A120MM_AP,
      Self::A120mmHE => &A120MM_HE,
      Self::A120mmHERPF => &A120MM_HERPF,
      Self::A250mmAP => &A250MM_AP,
      Self::A250mmHE => &A250MM_HE,
      Self::A250mmHERPF => &A250MM_HERPF,
      Self::A300mmAPRailSabot => &A300MM_AP_RAIL_SABOT,
      Self::A400mmPlasmaAmpoule => &A400MM_PLASMA_AMPOULE,
      Self::A450mmAP => &A450MM_AP,
      Self::A450mmHE => &A450MM_HE,
      Self::A500mmFracturingBlock => &A500MM_FRACTURING_BLOCK,
      Self::A600mmBomb => &A600MM_BOMB,
      Self::A600mmHESH => &A600MM_HESH,
      Self::CM4D1DecoyContainerClipper => &CM4D1_DECOY_CONTAINER_CLIPPER,
      Self::CM4D2DecoyContainerLineShip => &CM4D2_DECOY_CONTAINER_LINE_SHIP,
      Self::CM4MMineContainer => &CM4M_MINE_CONTAINER,
      Self::CM4R12RocketContainer => &CM4R12_ROCKET_CONTAINER,
      Self::CM4R6RocketContainer => &CM4R6_ROCKET_CONTAINER,
      Self::EA12ChaffDecoy => &EA12_CHAFF_DECOY,
      Self::EA20FlareDecoy => &EA20_FLARE_DECOY,
      Self::EA99ActiveDecoy => &EA99_ACTIVE_DECOY,
      Self::FlakRound => &FLAK_ROUND,
      Self::M30MattockMine => &M30_MATTOCK_MINE,
      Self::M30NMattockCooperativeMine => &M30N_MATTOCK_COOPERATIVE_MINE,
      Self::M50AugerSprintMine => &M50_AUGER_SPRINT_MINE,
      Self::R2PiranhaRocket => &R2_PIRANHA_ROCKET
    }
  }

  pub const VALUES: &'static [Self] = &[
    Self::A15mmSandshot,
    Self::A20mmSlug,
    Self::A100mmAP,
    Self::A100mmGrape,
    Self::A100mmHE,
    Self::A100mmHEHC,
    Self::A120mmAP,
    Self::A120mmHE,
    Self::A120mmHERPF,
    Self::A250mmAP,
    Self::A250mmHE,
    Self::A250mmHERPF,
    Self::A300mmAPRailSabot,
    Self::A400mmPlasmaAmpoule,
    Self::A450mmAP,
    Self::A450mmHE,
    Self::A500mmFracturingBlock,
    Self::A600mmBomb,
    Self::A600mmHESH,
    Self::CM4D1DecoyContainerClipper,
    Self::CM4D2DecoyContainerLineShip,
    Self::CM4MMineContainer,
    Self::CM4R12RocketContainer,
    Self::CM4R6RocketContainer,
    Self::EA12ChaffDecoy,
    Self::EA20FlareDecoy,
    Self::EA99ActiveDecoy,
    Self::FlakRound,
    Self::M30MattockMine,
    Self::M30NMattockCooperativeMine,
    Self::M50AugerSprintMine,
    Self::R2PiranhaRocket
  ];
}

impl FromStr for MunitionKey {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    MunitionKey::VALUES.iter().copied()
      .find(|munition_key| munition_key.save_key() == s)
      .ok_or(())
  }
}



pub mod list {
  use super::*;

  pub const A15MM_SANDSHOT: Munition = Munition {
    name: "15mm Sandshot",
    save_key: "Stock/15mm Sandshot",
    role: WeaponRole::Defensive,
    family: MunitionFamily::BallisticMagnetic15mm,
    point_cost: 1,
    point_division: 50,
    storage_volume: 0.0025,
    flight_speed: 2500.0,
    max_range: 8000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 3.0,
        component_damage: 75.0,
        overpenetration_damage_multiplier: 0.15,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A20MM_SLUG: Munition = Munition {
    name: "20mm Slug",
    save_key: "Stock/20mm Slug",
    role: WeaponRole::Defensive,
    family: MunitionFamily::BallisticChemical20mm,
    point_cost: 1,
    point_division: 2000,
    storage_volume: 0.0025,
    flight_speed: 700.0,
    max_range: 1750.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 3.0,
        component_damage: 15.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(0.5),
        can_ricochet: true
      }
    }
  };

  pub const A100MM_AP: Munition = Munition {
    name: "100mm AP Shell",
    save_key: "Stock/100mm AP Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical100mm,
    point_cost: 1,
    point_division: 250,
    storage_volume: 0.05,
    flight_speed: 900.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 45.0,
        component_damage: 25.0,
        overpenetration_damage_multiplier: 0.1,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A100MM_GRAPE: Munition = Munition {
    name: "100mm Grape",
    save_key: "Stock/100mm Grapeshot",
    role: WeaponRole::DualPurpose,
    family: MunitionFamily::BallisticChemical100mm,
    point_cost: 1,
    point_division: 250,
    storage_volume: 0.05,
    flight_speed: 1100.0,
    max_range: 7997.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 16.0,
        component_damage: 25.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A100MM_HE: Munition = Munition {
    name: "100mm HE Shell",
    save_key: "Stock/100mm HE Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical100mm,
    point_cost: 1,
    point_division: 250,
    storage_volume: 0.05,
    flight_speed: 900.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 30.0,
        component_damage: 45.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(1.5),
        can_ricochet: true
      }
    }
  };

  pub const A100MM_HEHC: Munition = Munition {
    name: "100mm HE-HC Shell",
    save_key: "Stock/100mm HE-HC Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical100mm,
    point_cost: 1,
    point_division: 250,
    storage_volume: 0.05,
    flight_speed: 900.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 8.0,
        component_damage: 70.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(3.0),
        can_ricochet: true
      }
    }
  };

  pub const A120MM_AP: Munition = Munition {
    name: "120mm AP Shell",
    save_key: "Stock/120mm AP Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical120mm,
    point_cost: 1,
    point_division: 100,
    storage_volume: 0.05,
    flight_speed: 800.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 45.0,
        component_damage: 30.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A120MM_HE: Munition = Munition {
    name: "120mm HE Shell",
    save_key: "Stock/120mm HE Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical120mm,
    point_cost: 1,
    point_division: 100,
    storage_volume: 0.05,
    flight_speed: 800.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 30.0,
        component_damage: 50.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(1.5),
        can_ricochet: true
      }
    }
  };

  pub const A120MM_HERPF: Munition = Munition {
    name: "120mm HE-RPF Shell",
    save_key: "Stock/120mm HE-RPF Shell",
    role: WeaponRole::DualPurpose,
    family: MunitionFamily::BallisticChemical120mm,
    point_cost: 1,
    point_division: 100,
    storage_volume: 0.05,
    flight_speed: 800.0,
    max_range: 7200.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 10.0,
        component_damage: 8.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: Some(1.0),
        can_ricochet: true
      }
    }
  };

  pub const A250MM_AP: Munition = Munition {
    name: "250mm AP Shell",
    save_key: "Stock/250mm AP Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical250mm,
    point_cost: 1,
    point_division: 50,
    storage_volume: 0.15,
    flight_speed: 800.0,
    max_range: 8000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 60.0,
        component_damage: 70.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A250MM_HE: Munition = Munition {
    name: "250mm HE Shell",
    save_key: "Stock/250mm HE Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical250mm,
    point_cost: 1,
    point_division: 50,
    storage_volume: 0.15,
    flight_speed: 800.0,
    max_range: 8000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 40.0,
        component_damage: 80.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(2.5),
        can_ricochet: true
      }
    }
  };

  pub const A250MM_HERPF: Munition = Munition {
    name: "250mm HE-RPF Shell",
    save_key: "Stock/250mm HE-RPF Shell",
    role: WeaponRole::DualPurpose,
    family: MunitionFamily::BallisticChemical250mm,
    point_cost: 1,
    point_division: 50,
    storage_volume: 0.15,
    flight_speed: 800.0,
    max_range: 8000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 20.0,
        component_damage: 16.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: Some(1.0),
        can_ricochet: true
      }
    }
  };

  pub const A300MM_AP_RAIL_SABOT: Munition = Munition {
    name: "300mm AP Rail Sabot",
    save_key: "Stock/300mm AP Rail Sabot",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticMagnetic300mmRailgun,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.75,
    flight_speed: 2000.0,
    max_range: 21000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 200.0,
        component_damage: 80.0,
        overpenetration_damage_multiplier: 0.5,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A400MM_PLASMA_AMPOULE: Munition = Munition {
    name: "400mm Plasma Ampoule",
    save_key: "Stock/400mm Plasma Ampoule",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticMagnetic400mmPlasma,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.8,
    flight_speed: 600.0,
    max_range: 8400.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 60.0,
        component_damage: 60.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(0.0),
        can_ricochet: true
      }
    }
  };

  pub const A450MM_AP: Munition = Munition {
    name: "450mm AP Shell",
    save_key: "Stock/450mm AP Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical450mm,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.8,
    flight_speed: 750.0,
    max_range: 11250.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 110.0,
        component_damage: 100.0,
        overpenetration_damage_multiplier: 0.3,
        max_penetration_depth: None,
        can_ricochet: true
      }
    }
  };

  pub const A450MM_HE: Munition = Munition {
    name: "450mm HE Shell",
    save_key: "Stock/450mm HE Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical450mm,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.8,
    flight_speed: 750.0,
    max_range: 11250.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 65.0,
        component_damage: 150.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(3.0),
        can_ricochet: true
      }
    }
  };

  pub const A500MM_FRACTURING_BLOCK: Munition = Munition {
    name: "500mm Fracturing Block",
    save_key: "Stock/500mm Fracturing Block",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticMagnetic500mmMassDriver,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.75,
    flight_speed: 2000.0,
    max_range: 21000.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 120.0,
        component_damage: 400.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(3.0),
        can_ricochet: true
      }
    }
  };

  pub const A600MM_BOMB: Munition = Munition {
    name: "600mm Bomb Shell",
    save_key: "Stock/600mm Bomb Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical600mm,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.8,
    flight_speed: 700.0,
    max_range: 9800.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 30.0,
        component_damage: 40.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(5.0),
        can_ricochet: true
      }
    }
  };

  pub const A600MM_HESH: Munition = Munition {
    name: "600mm HE-SH Shell",
    save_key: "Stock/600mm HE Shell",
    role: WeaponRole::Offensive,
    family: MunitionFamily::BallisticChemical600mm,
    point_cost: 1,
    point_division: 25,
    storage_volume: 0.8,
    flight_speed: 700.0,
    max_range: 9800.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 75.0,
        component_damage: 300.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(5.0),
        can_ricochet: true
      }
    }
  };

  pub const CM4D1_DECOY_CONTAINER_CLIPPER: Munition = Munition {
    name: "CM-4D1 Decoy Container (Clipper)",
    save_key: "Stock/Decoy Container (Clipper)",
    role: WeaponRole::Utility,
    family: MunitionFamily::ContainerMissile,
    point_cost: 4,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 54.0,
    max_range: 21600.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size3
    }
  };

  pub const CM4D2_DECOY_CONTAINER_LINE_SHIP: Munition = Munition {
    name: "CM-4D2 Decoy Container (Line Ship)",
    save_key: "Stock/Decoy Container (Line Ship)",
    role: WeaponRole::Utility,
    family: MunitionFamily::ContainerMissile,
    point_cost: 6,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 24.0,
    max_range: 15600.001,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size3
    }
  };

  pub const CM4M_MINE_CONTAINER: Munition = Munition {
    name: "CM-4M Mine Container",
    save_key: "Stock/Mine Container",
    role: WeaponRole::Utility,
    family: MunitionFamily::ContainerMissile,
    point_cost: 15,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 175.0,
    max_range: 11900.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size3
    }
  };

  pub const CM4R12_ROCKET_CONTAINER: Munition = Munition {
    name: "CM-4R12 Rocket Container",
    save_key: "Stock/Rocket Container 12",
    role: WeaponRole::Offensive,
    family: MunitionFamily::ContainerMissile,
    point_cost: 35,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 175.0,
    max_range: 17500.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size3
    }
  };

  pub const CM4R6_ROCKET_CONTAINER: Munition = Munition {
    name: "CM-4R6 Rocket Container",
    save_key: "Stock/Rocket Container",
    role: WeaponRole::Offensive,
    family: MunitionFamily::ContainerMissile,
    point_cost: 10,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 175.0,
    max_range: 17500.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size3
    }
  };

  pub const EA12_CHAFF_DECOY: Munition = Munition {
    name: "EA12 Chaff Decoy",
    save_key: "Stock/EA12 Chaff Decoy",
    role: WeaponRole::Decoy,
    family: MunitionFamily::StandardMissile,
    point_cost: 1,
    point_division: 1,
    storage_volume: 4.0,
    flight_speed: 70.0,
    max_range: 70.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size1
    }
  };

  pub const EA20_FLARE_DECOY: Munition = Munition {
    name: "EA20 Flare Decoy",
    save_key: "Stock/EA20 Flare Decoy",
    role: WeaponRole::Decoy,
    family: MunitionFamily::StandardMissile,
    point_cost: 1,
    point_division: 1,
    storage_volume: 4.0,
    flight_speed: 75.0,
    max_range: 75.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size1
    }
  };

  pub const EA99_ACTIVE_DECOY: Munition = Munition {
    name: "EA99 Active Decoy",
    save_key: "Stock/EA99 Active Decoy",
    role: WeaponRole::Decoy,
    family: MunitionFamily::StandardMissile,
    point_cost: 8,
    point_division: 1,
    storage_volume: 4.0,
    flight_speed: 20.0,
    max_range: 600.0,
    variant: MunitionVariant::MissileOther {
      size: MissileSize::Size1
    }
  };

  pub const FLAK_ROUND: Munition = Munition {
    name: "Flak Round",
    save_key: "Stock/Flak Round",
    role: WeaponRole::Defensive,
    family: MunitionFamily::BallisticChemical50mmFlak,
    point_cost: 1,
    point_division: 75,
    storage_volume: 0.01,
    flight_speed: 650.0,
    max_range: 2002.0,
    variant: MunitionVariant::Shell {
      damage: MunitionDamage {
        armor_penetration: 1.0,
        component_damage: 15.0,
        overpenetration_damage_multiplier: 0.2,
        max_penetration_depth: Some(0.2),
        can_ricochet: true
      }
    }
  };

  pub const M30_MATTOCK_MINE: Munition = Munition {
    name: "M-30 'Mattock' Mine",
    save_key: "Stock/S3 Mine",
    role: WeaponRole::Utility,
    family: MunitionFamily::LoiteringMine,
    point_cost: 6,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 250.0,
    max_range: 3750.0,
    variant: MunitionVariant::Missile {
      size: MissileSize::Size3,
      damage: MunitionDamage {
        armor_penetration: 200.0,
        component_damage: 5000.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: None,
        can_ricochet: false
      }
    }
  };

  pub const M30N_MATTOCK_COOPERATIVE_MINE: Munition = Munition {
    name: "M-30-N 'Mattock' Cooperative Mine",
    save_key: "Stock/S3 Net Mine",
    role: WeaponRole::Utility,
    family: MunitionFamily::LoiteringMine,
    point_cost: 6,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 250.0,
    max_range: 3750.0,
    variant: MunitionVariant::Missile {
      size: MissileSize::Size3,
      damage: MunitionDamage {
        armor_penetration: 200.0,
        component_damage: 5000.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: None,
        can_ricochet: false
      }
    }
  };

  pub const M50_AUGER_SPRINT_MINE: Munition = Munition {
    name: "M-50 'Auger' Sprint Mine",
    save_key: "Stock/S3 Sprint Mine",
    role: WeaponRole::Utility,
    family: MunitionFamily::LoiteringMine,
    point_cost: 10,
    point_division: 1,
    storage_volume: 30.0,
    flight_speed: 700.0,
    max_range: 3850.0,
    variant: MunitionVariant::Missile {
      size: MissileSize::Size3,
      damage: MunitionDamage {
        armor_penetration: 200.0,
        component_damage: 5000.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: None,
        can_ricochet: false
      }
    }
  };

  pub const R2_PIRANHA_ROCKET: Munition = Munition {
    name: "R-2 'Piranha' Rocket",
    save_key: "Stock/S1 Rocket",
    role: WeaponRole::Offensive,
    family: MunitionFamily::UnguidedRocket,
    point_cost: 3,
    point_division: 1,
    storage_volume: 6.0,
    flight_speed: 350.0,
    max_range: 7000.0,
    variant: MunitionVariant::Missile {
      size: MissileSize::Size1,
      damage: MunitionDamage {
        armor_penetration: 38.0,
        component_damage: 850.0,
        overpenetration_damage_multiplier: 1.0,
        max_penetration_depth: None,
        can_ricochet: false
      }
    }
  };
}
