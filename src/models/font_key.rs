use font_kit::{
  family_name::FamilyName,
  properties::{Properties, Stretch, Weight},
};
use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
};

#[derive(Clone, PartialEq)]
pub enum FontKey {
  Family {
    font_family: Cow<'static, [FamilyName]>,
    font_props: Properties,
  },
  Path(Cow<'static, str>),
}

impl Hash for FontKey {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    match *self {
      Self::Family {
        ref font_family,
        font_props,
      } => {
        let Properties {
          style,
          weight: Weight(weight),
          stretch: Stretch(stretch),
        } = font_props;

        font_family.hash(state);
        style.hash(state);
        weight.to_bits().hash(state);
        stretch.to_bits().hash(state);
      }
      Self::Path(ref font_path) => font_path.hash(state),
    }
  }
}

impl Eq for FontKey {}
