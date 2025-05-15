use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

#[proc_macro]
pub fn gen_icon_name_enum(_input: TokenStream) -> TokenStream {
  let codepoints_file =
    fs::read_to_string("flut_macro/assets/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].codepoints")
      .unwrap();

  let codepoints = codepoints_file
    .split_whitespace()
    .skip(1)
    .step_by(2)
    .map(|codepoint| u16::from_str_radix(codepoint, 16).unwrap());

  let icon_names = codepoints_file
    .split_whitespace()
    .step_by(2)
    .map(|icon_name| format_ident!("{}", snake_case_to_camel_case(icon_name)));

  let codepoint_to_icon_name = codepoints.zip(icon_names).collect::<BTreeMap<_, _>>();
  let codepoints = codepoint_to_icon_name.keys();
  let icon_names = codepoint_to_icon_name.values();

  quote! {
    #[derive(Clone, Copy)]
    #[repr(u16)]
    pub enum IconName {
      #(#icon_names = #codepoints,)*
    }
  }
  .into()
}

fn snake_case_to_camel_case(s: &str) -> String {
  let result = s
    .split('_')
    .map(|word| word[0..1].to_uppercase() + &word[1..])
    .collect::<String>();

  let first_char = result.chars().next().unwrap();

  if first_char.is_ascii_digit() {
    format!("_{result}")
  } else {
    result
  }
}
