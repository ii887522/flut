#![deny(elided_lifetimes_in_paths)]

pub mod app;
pub mod boot;
pub mod collections;
pub mod helpers;
pub mod models;
mod widget_tree;
pub mod widgets;

pub use app::App;
