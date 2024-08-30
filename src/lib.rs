#![deny(elided_lifetimes_in_paths)]

pub mod app;
pub mod collections;
pub mod helpers;
mod widget_tree;
pub mod widgets;

pub use app::App;
use widget_tree::WidgetTree;
