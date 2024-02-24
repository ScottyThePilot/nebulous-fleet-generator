#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  unreachable_pub
)]

extern crate itertools;
#[macro_use]
extern crate thiserror;
pub extern crate xml;

#[macro_use]
mod utils;
pub mod data;
pub mod format;

pub use crate::utils::Size;
