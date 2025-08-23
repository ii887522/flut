#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod log;

use crate::log::{LogLevel, log};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use rustc_hash::FxHashMap;
use std::fs;
use syn::LitStr;

#[proc_macro]
pub fn info(input: TokenStream) -> TokenStream {
  log(LogLevel::Info, input)
}

#[proc_macro]
pub fn warn(input: TokenStream) -> TokenStream {
  log(LogLevel::Warn, input)
}

#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
  log(LogLevel::Error, input)
}

#[proc_macro]
pub fn gen_icon_name_enum(input: TokenStream) -> TokenStream {
  let codepoints_path = syn::parse::<LitStr>(input).unwrap();
  let icon_name_to_codepoint = fs::read_to_string(codepoints_path.value()).unwrap();

  let icon_names = icon_name_to_codepoint
    .split_whitespace()
    .step_by(2)
    .map(|icon_name| {
      if icon_name.starts_with(|ch: char| ch.is_ascii_digit()) {
        format_ident!("_{}", snake_to_pascal_case(icon_name))
      } else {
        format_ident!("{}", snake_to_pascal_case(icon_name))
      }
    });

  let codepoints = icon_name_to_codepoint
    .split_whitespace()
    .skip(1)
    .step_by(2)
    .map(|codepoint| u16::from_str_radix(codepoint, 16).unwrap());

  let codepoint_to_icon_name = FxHashMap::from_iter(codepoints.zip(icon_names));
  let codepoints = codepoint_to_icon_name.keys();
  let icon_names = codepoint_to_icon_name.values();

  quote! {
    #[repr(u16)]
    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub enum IconName {
      #(#icon_names = #codepoints,)*
    }
  }
  .into()
}

fn snake_to_pascal_case(s: &str) -> String {
  s.split('_')
    .map(|s| s[0..1].to_uppercase() + &s[1..])
    .collect()
}
