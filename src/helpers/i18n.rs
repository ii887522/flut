use crate::models::{Locale, Value};
use optarg2chain::optarg_impl;
use regex::{Captures, Regex};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug)]
pub struct I18n {
  placeholder_regex: Regex,
  current_locale: Locale,
  messages: HashMap<Locale, HashMap<&'static str, &'static str>>,
}

#[optarg_impl]
impl I18n {
  #[optarg_method(I18nNewBuilder, call)]
  pub fn new(
    #[optarg_default] current_locale: Option<Locale>,
    messages: HashMap<Locale, HashMap<&'static str, &'static str>>,
  ) -> Self {
    let placeholder_regex = Regex::new(r"(\{\w+\})|(\{\w+:\w+\|\w+\})").unwrap();

    let current_locale = current_locale.unwrap_or_else(|| {
      sys_locale::get_locale()
        .unwrap_or("en-US".to_string())
        .into()
    });

    Self {
      placeholder_regex,
      current_locale,
      messages,
    }
  }

  pub const fn get_default_font_family(&self) -> &'static str {
    self.current_locale.get_default_font_family()
  }

  #[optarg_method(I18nTBuilder, call)]
  pub fn t<'a, 'b, 'c, 'd, 'e, 'f>(
    &'a self,
    message_key: &'b str,
    #[optarg_default] message_args: &'c [(&'d str, Value<'e>)],
  ) -> Cow<'f, str> {
    let message_args = message_args
      .iter()
      .map(|(k, v)| (*k, *v))
      .collect::<HashMap<_, _>>();

    self.placeholder_regex.replace_all(
      self.messages[&self.current_locale][message_key],
      |captures: &Captures<'_>| {
        if let Some(pluralization_placeholder) = captures.get(2) {
          let placeholder = pluralization_placeholder.as_str();
          let mut words = placeholder.split_terminator(&['{', ':', '|', '}'][..]);

          // split_terminator() did not skip the leading empty string, so manually skip it then we can get the
          // message_arg_key in the next words.next() invocation
          words.next().unwrap();

          let message_arg_key = words.next().unwrap();
          let message_arg_value = message_args[message_arg_key];
          let singular_word = words.next().unwrap();
          let plural_word = words.next().unwrap();

          return if message_arg_value.is_for_plural() {
            plural_word
          } else {
            singular_word
          }
          .to_string();
        }

        if let Some(simple_placeholder) = captures.get(1) {
          let placeholder = simple_placeholder.as_str();
          let message_arg_key = &placeholder[1..placeholder.len() - 1];
          let message_arg_value = message_args[message_arg_key];
          return message_arg_value.into();
        }

        unreachable!("No explicit capture group found");
      },
    )
  }
}
