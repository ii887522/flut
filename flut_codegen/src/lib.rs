use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{collections::HashMap, fs};
use syn::LitStr;

fn convert_snake_case_to_pascal_case(word: &str) -> String {
  let prefix = if word.starts_with(|letter| char::is_ascii_digit(&letter)) {
    "_".to_string()
  } else {
    "".to_string()
  };

  prefix
    + &word
      .split('_')
      .map(|word| {
        let mut letters = word.chars();
        let first_letter = letters.next().unwrap();
        let first_letter = first_letter.to_ascii_uppercase();
        first_letter.to_string() + letters.as_str()
      })
      .collect::<String>()
}

#[proc_macro]
pub fn gen_icon_name_enum(input: TokenStream) -> TokenStream {
  let input = syn::parse::<LitStr>(input).unwrap();
  let codepoints_file_path = input.value();
  let codepoint_map = fs::read_to_string(codepoints_file_path).unwrap();

  let icon_names = codepoint_map
    .split_whitespace()
    .step_by(2)
    .map(|icon_name| format_ident!("{}", convert_snake_case_to_pascal_case(icon_name)));

  let codepoints = codepoint_map
    .split_whitespace()
    .skip(1)
    .step_by(2)
    .map(|codepoint| u16::from_str_radix(codepoint, 16).unwrap());

  let codepoint_map = codepoints.zip(icon_names).collect::<HashMap<_, _>>();
  let icon_names = codepoint_map.values();
  let codepoints = codepoint_map.keys();

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(u16)]
    pub enum IconName {
      #(#icon_names = #codepoints,)*
    }
  }
  .into()
}
