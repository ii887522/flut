use super::Origin;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Locale {
  #[default]
  En,
  Id,
  ZhCn,
  ZhTw,
}

impl Locale {
  pub fn from_font_family(font_family: &str) -> Self {
    match font_family {
      "Arial" => Locale::En,
      "SimHei" => Locale::ZhCn,
      _ => Locale::En,
    }
  }

  pub const fn get_default_font_family(&self) -> &'static str {
    match self {
      Locale::En | Locale::Id => "Arial",
      Locale::ZhCn | Locale::ZhTw => "SimHei",
    }
  }

  pub const fn get_text_origin(&self) -> Origin {
    match self {
      Locale::En | Locale::Id => Origin::TopLeft,
      Locale::ZhCn | Locale::ZhTw => Origin::Left,
    }
  }
}

impl From<&str> for Locale {
  fn from(locale: &str) -> Self {
    match locale {
      "zh-CN" => Locale::ZhCn,
      "zh-TW" => Locale::ZhTw,
      locale => match locale.split('-').next().unwrap() {
        "en" => Locale::En,
        "id" => Locale::Id,
        "zh" => Locale::ZhCn,
        _ => Locale::En,
      },
    }
  }
}

impl From<String> for Locale {
  fn from(locale: String) -> Self {
    locale.as_str().into()
  }
}

impl From<&String> for Locale {
  fn from(locale: &String) -> Self {
    locale.as_str().into()
  }
}
