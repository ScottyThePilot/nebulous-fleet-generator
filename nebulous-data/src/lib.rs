#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  unreachable_pub
)]

pub extern crate bytemuck;
extern crate itertools;
pub extern crate nebulous_xml as xml;
#[macro_use]
extern crate thiserror;

#[macro_use]
pub mod utils;
pub mod data;
pub mod format;

pub use crate::utils::Size;

pub mod prelude {
  #[doc(no_inline)]
  pub use bytemuck::Contiguous;
  pub use crate::utils::{ContiguousExt, Lerp, lerp, lerp2, Size};
}
