use steamlocate::SteamDir;

use std::path::{Path, PathBuf};



pub const NEBULOUS_STEAM_APPID: u32 = 887570;

pub fn get_local_nebulous_dir() -> Option<PathBuf> {
  let mut steam_dir = SteamDir::locate()?;
  let nebulous_app = steam_dir.app(&NEBULOUS_STEAM_APPID)?;
  Some(nebulous_app.path.to_owned())
}
