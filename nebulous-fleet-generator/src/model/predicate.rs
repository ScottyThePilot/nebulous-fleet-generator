use crate::model::{DistanceRealm, EquipmentSummary, PointDefenseType, ShipState, WeaponFamily};
use crate::utils::{keyword, keyword_parse, keyword_match, symbol, Parseable, Symbol, Token};

use chumsky::prelude::*;
use nebulous_data::data::components::SigType;
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::MissileSize;

use std::ops::Range;
use std::str::FromStr;



#[derive(Debug, Clone)]
pub enum Predicate {
  Any(Box<[Self]>),
  All(Box<[Self]>),
  Not(Box<Self>),
  HullType(HullKey),
  CostBudgetTotal(Range<usize>),
  CostBudgetSpare(Range<usize>),
  Equipment(EquipmentPredicate)
}

impl Predicate {
  pub fn test(&self, ship_data: &ShipState) -> bool {
    match self {
      Self::All(predicates) => predicates.iter().all(|predicate| predicate.test(ship_data)),
      Self::Any(predicates) => predicates.iter().any(|predicate| predicate.test(ship_data)),
      Self::Not(predicate) => !predicate.test(ship_data),
      Self::CostBudgetTotal(cost_predicate) => cost_predicate.contains(&ship_data.cost_budget_total),
      Self::CostBudgetSpare(cost_predicate) => cost_predicate.contains(&ship_data.cost_budget_spare),
      Self::HullType(hull_key) => ship_data.hull_type == *hull_key,
      Self::Equipment(equipment_predicate) => equipment_predicate.test(&ship_data.equipment_summary)
    }
  }
}

impl Parseable<Token> for Predicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    recursive(|predicate| {
      let predicate_list = predicate.clone()
        .separated_by(symbol(Symbol::Comma)).allow_trailing().at_least(1)
        .delimited_by(symbol(Symbol::RoundBracketOpen), symbol(Symbol::RoundBracketClose))
        .map(Vec::into_boxed_slice);
      let predicate_single = predicate
        .delimited_by(symbol(Symbol::RoundBracketOpen), symbol(Symbol::RoundBracketClose))
        .map(Box::new);
      let range = keyword_parse::<usize>()
        .then_ignore(symbol(Symbol::Ellipsis))
        .then(keyword_parse::<usize>())
        .map(|(start, end)| Range { start, end });
      let hull_type = keyword_match(|keyword| match keyword {
        "sprinter" => Some(HullKey::SprinterCorvette),
        "raines" => Some(HullKey::RainesFrigate),
        "keystone" => Some(HullKey::KeystoneDestroyer),
        "vauxhall" => Some(HullKey::VauxhallLightCruiser),
        "axford" => Some(HullKey::AxfordHeavyCruiser),
        "solomon" => Some(HullKey::SolomonBattleship),
        "ferryman" | "shuttle" => Some(HullKey::FerrymanClipper),
        "draugr" | "tugboat" => Some(HullKey::DraugrClipper),
        "flathead" | "cargo_feeder" => Some(HullKey::FlatheadMonitor),
        "ocello" => Some(HullKey::OcelloCommandCruiser),
        "marauder" | "bulk_freighter" => Some(HullKey::MarauderLineShip),
        "moorline" | "container_liner" => Some(HullKey::MoorlineLineShip),
        _ => None
      });

      choice((
        keyword("any").ignore_then(predicate_list.clone()).map(Self::Any),
        keyword("all").ignore_then(predicate_list.clone()).map(Self::All),
        keyword("not").ignore_then(predicate_single).map(Self::Not),
        keyword("hull_type").then(symbol(Symbol::Slash))
          .ignore_then(hull_type).map(Self::HullType),
        keyword("cost_budget_total").then(symbol(Symbol::Slash))
          .ignore_then(range.clone()).map(Self::CostBudgetSpare),
        keyword("cost_budget_spare").then(symbol(Symbol::Slash))
          .ignore_then(range.clone()).map(Self::CostBudgetTotal),
        keyword("equipment").then(symbol(Symbol::Slash))
          .ignore_then(EquipmentPredicate::parser()).map(Self::Equipment),
      ))
    })
  }
}

impl FromStr for Predicate {
  type Err = crate::utils::Errors;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::utils::run::<Predicate>(s)
  }
}

#[derive(Debug, Clone)]
pub enum EquipmentPredicate {
  Intelligence,
  Illuminator,
  DeceptionModule,
  MissileIdentification,
  FireControl(Option<SigType>),
  Sensor(Option<SigType>),
  Jammer(Option<SigType>),
  Weapon(WeaponFamilyPredicate)
}

impl EquipmentPredicate {
  pub fn test(&self, equipment_summary: &EquipmentSummary) -> bool {
    match self {
      Self::Intelligence => equipment_summary.has_intelligence,
      Self::Illuminator => equipment_summary.has_illuminator,
      Self::DeceptionModule => equipment_summary.has_deception_module,
      Self::MissileIdentification => equipment_summary.has_missile_identification,
      Self::FireControl(sig_type) => sig_type.map_or(true, |sig_type| equipment_summary.fire_control.contains(&sig_type)),
      Self::Sensor(sig_type) => sig_type.map_or(true, |sig_type| equipment_summary.sensors.contains(&sig_type)),
      Self::Jammer(sig_type) => sig_type.map_or(true, |sig_type| equipment_summary.jamming.contains(&sig_type)),
      Self::Weapon(weapon_family_predicate) => equipment_summary.weapons.iter().any(|weapon_family| {
        weapon_family_predicate.test(weapon_family)
      })
    }
  }
}

impl Parseable<Token> for EquipmentPredicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    let sig_type_predicate = symbol(Symbol::Slash)
      .ignore_then(keyword_parse::<SigType>()).or_not().boxed();

    choice((
      keyword("intelligence").to(Self::Intelligence),
      keyword("illuminator").to(Self::Illuminator),
      keyword("deception_module").to(Self::DeceptionModule),
      keyword("missile_identification").to(Self::MissileIdentification),
      keyword("fire_control").ignore_then(sig_type_predicate.clone()).map(Self::FireControl),
      keyword("sensor").ignore_then(sig_type_predicate.clone()).map(Self::Sensor),
      keyword("jammer").ignore_then(sig_type_predicate.clone()).map(Self::Jammer),
      keyword("weapon").then(symbol(Symbol::Slash))
        .ignore_then(WeaponFamilyPredicate::parser()).map(Self::Weapon)
    ))
  }
}

#[derive(Debug, Clone)]
pub enum WeaponFamilyPredicate {
  EnergyBeam(Option<MultiPredicate<DistanceRealm>>),
  EnergyPlasma(Option<MultiPredicate<DistanceRealm>>),
  EnergyRailgun(Option<MultiPredicate<DistanceRealm>>),
  Ballistic(Option<MultiPredicate<DistanceRealm>>),
  PointDefense(Option<PointDefenseType>),
  StandardMissile(Option<MissileSize>),
  ContainerMissile,
  LoiteringMine,
  UnguidedRocket
}

impl WeaponFamilyPredicate {
  pub fn test(&self, weapon_family: &WeaponFamily) -> bool {
    match (self, weapon_family) {
      (Self::EnergyBeam(distance_realm_predicate), WeaponFamily::EnergyBeam(distance_realm)) |
      (Self::EnergyPlasma(distance_realm_predicate), WeaponFamily::EnergyPlasma(distance_realm)) |
      (Self::EnergyRailgun(distance_realm_predicate), WeaponFamily::EnergyRailgun(distance_realm)) |
      (Self::Ballistic(distance_realm_predicate), WeaponFamily::Ballistic(distance_realm)) => {
        distance_realm_predicate.as_ref().map_or(true, |p| p.contains(distance_realm))
      },
      (Self::PointDefense(point_defense_type_predicate), WeaponFamily::PointDefense(point_defense_type)) => {
        point_defense_type_predicate.as_ref().map_or(true, |p| p == point_defense_type)
      },
      (Self::StandardMissile(missile_size_predicate), WeaponFamily::StandardMissile(missile_size)) => {
        missile_size_predicate.as_ref().map_or(true, |p| p == missile_size)
      },
      (Self::ContainerMissile, WeaponFamily::ContainerMissile) => true,
      (Self::LoiteringMine, WeaponFamily::LoiteringMine) => true,
      (Self::UnguidedRocket, WeaponFamily::UnguidedRocket) => true,
      _ => false
    }
  }
}

impl Parseable<Token> for WeaponFamilyPredicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    let distance_realm_predicate = symbol(Symbol::Slash)
      .ignore_then(<MultiPredicate<DistanceRealm>>::parser()).or_not().boxed();
    let point_defense_type_predicate = symbol(Symbol::Slash)
      .ignore_then(PointDefenseType::parser()).or_not();
    let missile_size_predicate = symbol(Symbol::Slash)
      .ignore_then(keyword_parse::<MissileSize>()).or_not();

    choice((
      keyword("energy_beam").ignore_then(distance_realm_predicate.clone()).map(Self::EnergyBeam),
      keyword("energy_plasma").ignore_then(distance_realm_predicate.clone()).map(Self::EnergyPlasma),
      keyword("energy_rail_gun").ignore_then(distance_realm_predicate.clone()).map(Self::EnergyRailgun),
      keyword("ballistic").ignore_then(distance_realm_predicate.clone()).map(Self::Ballistic),
      keyword("point_defense").ignore_then(point_defense_type_predicate).map(Self::PointDefense),
      keyword("standard_missile").ignore_then(missile_size_predicate).map(Self::StandardMissile),
      keyword("container_missile").to(Self::ContainerMissile),
      keyword("loitering_mine").to(Self::LoiteringMine),
      keyword("unguided_rocket").to(Self::UnguidedRocket)
    ))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MultiPredicate<T> {
  InRange(T, T),
  AnyOf(Box<[T]>),
  Only(T)
}

impl<T: PartialOrd + PartialEq> MultiPredicate<T> {
  pub fn contains(&self, value: &T) -> bool {
    match self {
      Self::InRange(predicate_start, predicate_end) => predicate_start <= value && value <= predicate_end,
      Self::AnyOf(predicate_values) => predicate_values.contains(value),
      Self::Only(predicate_value) => predicate_value == value
    }
  }
}

impl<T: Parseable<Token> + 'static> Parseable<Token> for MultiPredicate<T> {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    let t = T::parser().boxed();

    choice((
      t.clone().then_ignore(symbol(Symbol::Ellipsis)).then(t.clone())
        .map(|(start, end)| Self::InRange(start, end)),
      t.clone().separated_by(symbol(Symbol::Comma)).allow_trailing().at_least(1)
        .delimited_by(symbol(Symbol::SquareBracketOpen), symbol(Symbol::SquareBracketClose))
        .map(|list| Self::AnyOf(list.into_boxed_slice())),
      t.clone()
        .map(Self::Only)
    ))
  }
}

impl Parseable<Token> for DistanceRealm {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_parse::<DistanceRealm>()
  }
}

impl Parseable<Token> for PointDefenseType {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_parse::<PointDefenseType>()
  }
}
