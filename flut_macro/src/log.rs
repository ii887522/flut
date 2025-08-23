use proc_macro::TokenStream;
use quote::quote;
use std::fmt::{self, Display, Formatter};
use syn::LitStr;

pub(super) enum LogLevel {
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

pub(super) fn log(level: LogLevel, input: TokenStream) -> TokenStream {
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
