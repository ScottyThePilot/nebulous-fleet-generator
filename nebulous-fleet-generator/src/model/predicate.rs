use crate::model::{DistanceRealm, ShipEquipmentSummary, MissileEquipmentSummary, PointDefenseType, ShipState, MissileState, WeaponFamily};
use crate::utils::{keyword, keyword_parse, keyword_match, symbol, Parseable, Symbol, Token};

use chumsky::prelude::*;
use nebulous_data::data::components::SigType;
use nebulous_data::data::hulls::HullKey;
use nebulous_data::data::missiles::bodies::MissileBodyKey;
use nebulous_data::data::missiles::seekers::{SeekerKind, SeekerMode};
use nebulous_data::data::missiles::{AuxiliaryKey, AvionicsKey, Maneuvers, WarheadKey};
use nebulous_data::data::MissileSize;
use nebulous_data::loadout::AvionicsConfigured;
use serde::de::{Deserialize, Deserializer};

use std::ops::Range;
use std::str::FromStr;



#[derive(Debug, Clone)]
pub enum ShipPredicate {
  Any(Box<[Self]>),
  All(Box<[Self]>),
  Not(Box<Self>),
  Tag(String),
  HullKey(HullKey),
  CostBudgetTotal(Range<usize>),
  CostBudgetSpare(Range<usize>),
  Equipment(ShipEquipmentPredicate)
}

impl ShipPredicate {
  pub fn test(&self, ship_state: &ShipState) -> bool {
    match self {
      Self::All(predicates) => predicates.iter().all(|predicate| predicate.test(ship_state)),
      Self::Any(predicates) => predicates.iter().any(|predicate| predicate.test(ship_state)),
      Self::Not(predicate) => !predicate.test(ship_state),
      Self::Tag(tag) => ship_state.tags.contains(tag),
      Self::HullKey(hull_key) => ship_state.loadout.hull_type == *hull_key,
      Self::CostBudgetTotal(cost_predicate) => cost_predicate.contains(&ship_state.cost_budget_total),
      Self::CostBudgetSpare(cost_predicate) => cost_predicate.contains(&ship_state.cost_budget_spare),
      Self::Equipment(equipment_predicate) => equipment_predicate.test(&ship_state.equipment_summary)
    }
  }
}

impl Parseable<Token> for ShipPredicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    recursive(|predicate| {
      let predicate_list = crate::utils::delimited_round_bracket_list(predicate.clone(), 1).map(Vec::into_boxed_slice);
      let predicate_single = crate::utils::delimited_by_round_brackets(predicate).map(Box::new);
      let range = keyword_parse::<usize>()
        .then_ignore(symbol(Symbol::Ellipsis))
        .then(keyword_parse::<usize>())
        .map(|(start, end)| Range { start, end });
      let hull_key = keyword_match(|keyword| match keyword {
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
        keyword("hull_key").then(symbol(Symbol::Slash))
          .ignore_then(hull_key).map(Self::HullKey),
        keyword("cost_budget_total").then(symbol(Symbol::Slash))
          .ignore_then(range.clone()).map(Self::CostBudgetSpare),
        keyword("cost_budget_spare").then(symbol(Symbol::Slash))
          .ignore_then(range.clone()).map(Self::CostBudgetTotal),
        keyword("equipment").then(symbol(Symbol::Slash))
          .ignore_then(ShipEquipmentPredicate::parser()).map(Self::Equipment),
      ))
    })
  }
}

impl FromStr for ShipPredicate {
  type Err = crate::utils::Errors;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::utils::run::<ShipPredicate>(s)
  }
}

impl<'de> Deserialize<'de> for ShipPredicate {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    String::deserialize(deserializer).and_then(|string| {
      string.parse::<Self>().map_err(serde::de::Error::custom)
    })
  }
}

#[derive(Debug, Clone)]
pub enum ShipEquipmentPredicate {
  Intelligence,
  Illuminator,
  DeceptionModule,
  MissileIdentification,
  FireControl(Option<SigType>),
  Sensor(Option<SigType>),
  Jammer(Option<SigType>),
  Weapon(WeaponFamilyPredicate)
}

impl ShipEquipmentPredicate {
  pub fn test(&self, equipment_summary: &ShipEquipmentSummary) -> bool {
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

impl Parseable<Token> for ShipEquipmentPredicate {
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

#[derive(Debug, Clone)]
pub enum MissilePredicate {
  Any(Box<[Self]>),
  All(Box<[Self]>),
  Not(Box<Self>),
  Tag(String),
  MissileBodyKey(MissileBodyKey),
  Cost(Range<usize>),
  Equipment(MissileEquipmentPredicate)
}

impl MissilePredicate {
  pub fn test(&self, missile_state: &MissileState) -> bool {
    match self {
      Self::All(predicates) => predicates.iter().all(|predicate| predicate.test(missile_state)),
      Self::Any(predicates) => predicates.iter().any(|predicate| predicate.test(missile_state)),
      Self::Not(predicate) => !predicate.test(missile_state),
      Self::Tag(tag) => missile_state.tags.contains(tag),
      Self::MissileBodyKey(body_key) => missile_state.loadout.body_key == *body_key,
      Self::Cost(cost_predicate) => cost_predicate.contains(&missile_state.cost),
      Self::Equipment(equipment_predicate) => equipment_predicate.test(&missile_state.equipment_summary)
    }
  }
}

#[derive(Debug, Clone)]
pub enum MissileEquipmentPredicate {
  Seeker(SeekerKind, SeekerMode),
  Auxiliary(AuxiliaryKey),
  Avionics(AvionicsKey, Option<AvionicsPredicate>),
  Warhead(WarheadKey)
}

impl MissileEquipmentPredicate {
  pub fn test(&self, equipment_summary: &MissileEquipmentSummary) -> bool {
    match self {
      Self::Seeker(seeker_kind, seeker_mode) => {
        equipment_summary.seekers.iter().any(|(sk, sm)| {
          sk == *seeker_kind && sm == *seeker_mode
        })
      },
      Self::Auxiliary(auxiliary_key) => {
        equipment_summary.auxiliary_components.contains(auxiliary_key)
      },
      Self::Avionics(avionics_key, avionics_predicate) => {
        equipment_summary.avionics.into_avionics_key() == *avionics_key &&
        avionics_predicate.as_ref().map_or(true, |avionics_predicate| {
          avionics_predicate.test(equipment_summary.avionics)
        })
      },
      Self::Warhead(warhead_key) => {
        equipment_summary.warhead == Some(*warhead_key)
      }
    }
  }
}

impl Parseable<Token> for MissileEquipmentPredicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    let predicate_seeker_mode = symbol(Symbol::Comma)
      .ignore_then(SeekerMode::parser()).or_not()
      .map(|seeker_mode| seeker_mode.unwrap_or(SeekerMode::Targeting));
    let predicate_seeker = crate::utils::delimited_by_round_brackets(SeekerKind::parser().then(predicate_seeker_mode))
      .map(|(seeker_kind, seeker_mode)| Self::Seeker(seeker_kind, seeker_mode));

    let predicate_avionics = AvionicsKey::parser();



    choice((
      keyword("seeker").then(symbol(Symbol::Slash))
        .ignore_then().map(),
      keyword("auxiliary").then(symbol(Symbol::Slash))
        .ignore_then().map(),
      keyword("avionics").then(symbol(Symbol::Slash))
        .ignore_then().map(),
      keyword("warhead").then(symbol(Symbol::Slash))
        .ignore_then().map(),
    ))
  }
}

#[derive(Debug, Clone)]
pub enum AvionicsPredicate {
  Any(Box<[Self]>),
  All(Box<[Self]>),
  Not(Box<Self>),
  HotLaunch,
  SelfDestructOnLost,
  Maneuvers(Maneuvers),
  DefensiveDoctrine
}

impl AvionicsPredicate {
  pub fn test(&self, avionics_configured: AvionicsConfigured) -> bool {
    match self {
      Self::All(predicates) => predicates.iter().all(|predicate| predicate.test(avionics_configured)),
      Self::Any(predicates) => predicates.iter().any(|predicate| predicate.test(avionics_configured)),
      Self::Not(predicate) => !predicate.test(avionics_configured),
      Self::HotLaunch => avionics_configured.hot_launch(),
      Self::SelfDestructOnLost => avionics_configured.self_destruct_on_lost(),
      Self::Maneuvers(maneuvers) => avionics_configured.maneuvers() == *maneuvers,
      Self::DefensiveDoctrine => avionics_configured.defensive_doctrine().is_some()
    }
  }
}

impl Parseable<Token> for AvionicsPredicate {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    recursive(|predicate| {
      let predicate_list = crate::utils::delimited_round_bracket_list(predicate.clone(), 1).map(Vec::into_boxed_slice);
      let predicate_single = crate::utils::delimited_by_round_brackets(predicate).map(Box::new);

      choice((
        keyword("any").ignore_then(predicate_list.clone()).map(Self::Any),
        keyword("all").ignore_then(predicate_list.clone()).map(Self::All),
        keyword("not").ignore_then(predicate_single).map(Self::Not),
        keyword("hot_launch").to(Self::HotLaunch),
        keyword("self_destruct_on_lost").to(Self::SelfDestructOnLost),
        keyword("maneuvers").then(symbol(Symbol::Slash))
          .ignore_then(Maneuvers::parser()).map(Self::Maneuvers),
        keyword("defensive_doctrine").to(Self::DefensiveDoctrine)
      ))
    })
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

impl Parseable<Token> for SeekerKind {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "command" => Some(Self::Command),
      "active_radar" => Some(Self::ActiveRadar),
      "semi_active_radar" => Some(Self::SemiActiveRadar),
      "anti_radiation" => Some(Self::AntiRadiation),
      "home_on_jam" => Some(Self::HomeOnJam),
      "electro_optical" => Some(Self::ElectroOptical),
      "wake_homing" => Some(Self::WakeHoming),
      _ => None
    })
  }
}

impl Parseable<Token> for SeekerMode {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "targeting" | "tgt" => Some(Self::Targeting),
      "validation" | "val" => Some(Self::Validation),
      _ => None
    })
  }
}

impl Parseable<Token> for AuxiliaryKey {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "cold_gas_bottle" => Some(Self::ColdGasBottle),
      "decoy_launcher" => Some(Self::DecoyLauncher),
      "cluster_decoy_launcher" => Some(Self::ClusterDecoyLauncher),
      "fast_startup_module" => Some(Self::FastStartupModule),
      "hardened_skin" => Some(Self::HardenedSkin),
      "radar_absorbent_coating" => Some(Self::RadarAbsorbentCoating),
      "self_screening_jammer" => Some(Self::SelfScreeningJammer),
      "boosted_self_screening_jammer" => Some(Self::BoostedSelfScreeningJammer),
      _ => None
    })
  }
}

impl Parseable<Token> for AvionicsKey {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "direct_guidance" => Some(Self::DirectGuidance),
      "cruise_guidance" => Some(Self::CruiseGuidance),
      _ => None
    })
  }
}

impl Parseable<Token> for WarheadKey {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "he_impact" => Some(Self::HEImpact),
      "he_kinetic_penetrator" => Some(Self::HEKineticPenetrator),
      "blast_fragmentation" => Some(Self::BlastFragmentation),
      "blast_fragmentation_el" => Some(Self::BlastFragmentationEL),
      _ => None
    })
  }
}

impl Parseable<Token> for Maneuvers {
  fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
    keyword_match(|keyword| match keyword {
      "none" => Some(Self::None),
      "weave" => Some(Self::Weave),
      "corkscrew" => Some(Self::Corkscrew),
      _ => None
    })
  }
}
