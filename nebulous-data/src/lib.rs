#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  unreachable_pub
)]

pub extern crate bytemuck;
extern crate float_ord;
extern crate indexmap;
extern crate itertools;
pub extern crate nebulous_xml as xml;
#[cfg(feature = "rand")]
extern crate rand;
#[cfg(feature = "serde")]
extern crate serde;
#[macro_use]
extern crate thiserror;
pub extern crate uuid;

#[macro_export]
macro_rules! key {
  ($expr:expr) => ($crate::format::key::Key::from_str_unchecked($expr));
}

#[macro_use]
pub mod utils;
pub mod data;
pub mod format;
pub mod loadout;

pub use crate::format::key::Key;
pub use crate::utils::Size;

pub mod prelude {
  #[doc(no_inline)]
  pub use bytemuck::Contiguous;
  pub use uuid::Uuid;
  pub use crate::format::key::Key;
  pub use crate::utils::{ContiguousExt, Lerp, lerp, lerp2, Size};
}
