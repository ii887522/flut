#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod app;
pub mod collections;
pub mod context;
pub mod models;
mod pipelines;
pub mod renderers;
pub mod utils;
pub mod widgets;

pub use app::App;
pub use context::Context;
pub use sdl3::*;
