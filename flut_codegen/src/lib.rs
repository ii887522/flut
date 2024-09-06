use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use syn::LitStr;

#[proc_macro]
pub fn gen_icon_names(input: TokenStream) -> TokenStream {
  let input = syn::parse::<LitStr>(input).unwrap();
  let codepoint_file_path = input.value();
  let codepoint_data = fs::read_to_string(codepoint_file_path).unwrap();

  let icon_names = codepoint_data
    .split_whitespace()
    .step_by(2)
    .map(|icon_name| {
      format_ident!(
        "{}{}",
        if icon_name.starts_with(|c: char| c.is_ascii_digit()) {
          "_"
        } else {
          ""
        },
        icon_name.to_uppercase()
      )
    });

  let codepoints = codepoint_data
    .split_whitespace()
    .skip(1)
    .step_by(2)
    .map(|codepoint| u16::from_str_radix(codepoint, 16).unwrap());

  quote! {
    #(pub const #icon_names: u16 = #codepoints;)*
  }
  .into()
}
