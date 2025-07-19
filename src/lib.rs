#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod app;
mod collections;
mod consts;
pub mod models;
pub mod renderer;
mod vk;
mod write;

pub use app::App;
pub use renderer::Renderer;
use std::{mem, slice};
use write::Write;

const fn as_bytes<T>(item: &T) -> &[u8] {
  unsafe { slice::from_raw_parts(item as *const _ as *const _, mem::size_of::<T>()) }
}
