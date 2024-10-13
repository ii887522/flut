pub mod animation;
pub mod animation_count;
pub mod clock;
pub mod i18n;
pub mod shake_animation_sm;

pub use animation::Animation;
pub use animation_count::AnimationCount;
pub use clock::Clock;
pub use i18n::I18n;
pub use shake_animation_sm::ShakeAnimationSM;

use crate::boot::context;
use skia_safe::{Font, FontStyle};

pub(super) fn new_font(font_family: &'static str, font_style: FontStyle, font_size: f32) -> Font {
  context::TEXT_TYPEFACES.with_borrow_mut(|text_typefaces| {
    let typeface = text_typefaces
      .entry(format!(
        "FontFamily#{}#Weight#{}#Width#{}#Slant#{:?}",
        font_family,
        *font_style.weight(),
        *font_style.width(),
        font_style.slant()
      ))
      .or_insert_with(|| {
        context::FONT_MGR.with(|font_mgr| {
          font_mgr
            .match_family_style(font_family, font_style)
            .unwrap()
        })
      });

    Font::new(&*typeface, font_size)
  })
}
