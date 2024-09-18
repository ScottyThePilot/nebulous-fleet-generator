use crate::data::components::{ComponentKey, ComponentVariant, SigType};
use crate::data::hulls::HullKey;
use crate::data::hulls::config::Variant;
use crate::data::missiles::{AvionicsKey, Maneuvers, WarheadKey};
use crate::data::missiles::seekers::{SeekerKind, SeekerStrategy};
use crate::data::missiles::bodies::MissileBodyKey;
use crate::data::munitions::{MunitionFamily, MunitionKey, WeaponRole};
use crate::data::MissileSize;
use crate::format::{ComponentData, Color, HullSocket, MunitionOrMissileKey, MagazineSaveData, MissileTemplate, MissileTemplateContents, MissileSocket, Ship};
use crate::format::key::Key;

use indexmap::IndexMap;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::Extend;
use std::str::FromStr;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLoadout {
  pub hull_type: HullKey,
  pub hull_config: Option<[Variant; 3]>,
  pub sockets: Box<[Option<ShipLoadoutSocket>]>
}

impl ShipLoadout {
  pub fn from_ship(ship: &Ship) -> Option<Self> {
    let hull = ship.hull_type.hull();
    let hull_config = ship.hull_config.as_ref().zip(hull.config_template)
      .and_then(|(hull_config, config_template)| config_template.get_variants(hull_config));

    let mut component_map = ship.socket_map.iter()
      .map(|hull_socket| (hull_socket.key, ShipLoadoutSocket::from_hull_socket(hull_socket)))
      .collect::<HashMap<Key, ShipLoadoutSocket>>();
    let sockets = hull.sockets.iter()
      .map(|hull_socket| component_map.remove(&hull_socket.save_key))
      .collect::<Box<[Option<ShipLoadoutSocket>]>>();

    Some(ShipLoadout {
      hull_type: ship.hull_type,
      hull_config,
      sockets
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLoadoutSocket {
  component_key: ComponentKey,
  variant: Option<ShipLoadoutSocketVariant>
}

impl ShipLoadoutSocket {
  pub fn from_hull_socket(hull_socket: &HullSocket) -> Self {
    let component_key = hull_socket.component_name;
    let variant = hull_socket.component_data.as_ref()
      .map(ShipLoadoutSocketVariant::from_component_data);
    ShipLoadoutSocket { component_key, variant }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShipLoadoutSocketVariant {
  DeceptionComponent {
    identity_option: usize
  },
  MagazineComponent {
    magazine_contents: IndexMap<MunitionOrMissileKey, usize>
  }
}

impl ShipLoadoutSocketVariant {
  pub fn from_component_data(component_data: &ComponentData) -> Self {
    match *component_data {
      ComponentData::BulkMagazineData { ref load } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(load) }
      },
      ComponentData::CellLauncherData { ref missile_load } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(missile_load) }
      },
      ComponentData::ResizableCellLauncherData { ref missile_load, .. } => {
        Self::MagazineComponent { magazine_contents: get_magazine_contents(missile_load) }
      },
      ComponentData::DeceptionComponentData { identity_option } => {
        Self::DeceptionComponent { identity_option }
      }
    }
  }
}

fn get_magazine_contents(load: &[MagazineSaveData]) -> IndexMap<MunitionOrMissileKey, usize> {
  let mut magazine_contents = IndexMap::new();
  for &MagazineSaveData { ref munition_key, quantity, .. } in load.iter() {
    *magazine_contents.entry(munition_key.clone()).or_default() += quantity;
  };

  magazine_contents
}
