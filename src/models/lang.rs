use super::Origin;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Lang {
  #[default]
  En,
  Id,
  ZhCn,
  ZhTw,
}

impl Lang {
  pub fn from_font_family(font_family: &str) -> Self {
    #[cfg(target_os = "macos")]
    match font_family {
      "Arial" => Lang::En,
      "Heiti SC" => Lang::ZhCn,
      "Heiti TC" => Lang::ZhTw,
      _ => Lang::En,
    }

    #[cfg(not(target_os = "macos"))]
    match font_family {
      "Arial" => Lang::En,
      "SimHei" => Lang::ZhCn,
      _ => Lang::En,
    }
  }

  pub const fn get_default_font_family(&self) -> &'static str {
    #[cfg(target_os = "macos")]
    match self {
      Lang::En | Lang::Id => "Arial",
      Lang::ZhCn => "Heiti SC",
      Lang::ZhTw => "Heiti TC",
    }

    #[cfg(not(target_os = "macos"))]
    match self {
      Lang::En | Lang::Id => "Arial",
      Lang::ZhCn | Lang::ZhTw => "SimHei",
    }
  }

  pub const fn get_text_origin(&self) -> Origin {
    match self {
      Lang::En | Lang::Id => Origin::TopLeft,
      Lang::ZhCn | Lang::ZhTw => Origin::Left,
    }
  }
}

impl From<&str> for Lang {
  fn from(lang: &str) -> Self {
    match lang {
      "zh-CN" => Lang::ZhCn,
      "zh-TW" => Lang::ZhTw,
      lang => match lang.split('-').next().unwrap() {
        "en" => Lang::En,
        "id" => Lang::Id,
        "zh" => Lang::ZhCn,
        _ => Lang::En,
      },
    }
  }
}

impl From<String> for Lang {
  fn from(lang: String) -> Self {
    lang.as_str().into()
  }
}

impl From<&String> for Lang {
  fn from(lang: &String) -> Self {
    lang.as_str().into()
  }
}
