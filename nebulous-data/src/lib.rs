extern crate itertools;
#[macro_use]
extern crate thiserror;
pub extern crate xml;

#[macro_use]
mod utils;
pub mod data;
pub mod format;

pub use crate::utils::Size;
pub use xml::uuid::Uuid;
