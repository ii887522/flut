#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

use proc_macro::TokenStream;
use quote::quote;
use std::fmt::{self, Display, Formatter};
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

enum LogLevel {
  Info,
  Warn,
  Error,
}

impl Display for LogLevel {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    match self {
      LogLevel::Info => fmt.write_str("INFO"),
      LogLevel::Warn => fmt.write_str("WARN"),
      LogLevel::Error => fmt.write_str("ERROR"),
    }
  }
}

fn log(level: LogLevel, input: TokenStream) -> TokenStream {
  let msg = syn::parse::<LitStr>(input).unwrap();
  let level = level.to_string();

  quote! {{
    let level = #level;
    let this_file = file!();
    let this_line = line!();
    let msg = format!(#msg);
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    println!("[{level}] {now} [{this_file}:{this_line}] {msg}");
  }}
  .into()
}
