use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Route<'a> {
  pub(crate) path: &'a str,
  pub(crate) qs_params: HashMap<&'a str, &'a str>,
}

impl<'a> Route<'a> {
  pub(crate) fn from_relative_url(relative_url: &'a str) -> Self {
    let mut url_parts = relative_url.split('?');
    let path = url_parts.next().unwrap_or_default();

    let qs_params = url_parts
      .next()
      .map(|qs_params| {
        qs_params
          .split('&')
          .map(|qs_param| {
            let mut param_parts = qs_param.split('=');
            let key = param_parts.next().unwrap_or_default();
            let value = param_parts.next().unwrap_or_default();
            (key, value)
          })
          .collect::<HashMap<_, _>>()
      })
      .unwrap_or_default();

    Self { path, qs_params }
  }
}
