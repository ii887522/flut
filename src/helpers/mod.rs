pub mod animation;
pub mod animation_count;
pub mod clock;
pub mod consts;
pub mod i18n;
pub mod shake_animation_sm;

pub use animation::Animation;
pub use animation_count::AnimationCount;
pub use clock::Clock;
pub use i18n::I18n;
pub use shake_animation_sm::ShakeAnimationSM;

use crate::{boot::context, models::TextStyle};
use skia_safe::Font;

pub(super) fn new_font(style: TextStyle) -> Font {
  context::TEXT_TYPEFACES.with_borrow_mut(|text_typefaces| {
    let typeface = text_typefaces
      .entry(format!(
        "FontFamily#{}#Weight#{}#Width#{}#Slant#{:?}",
        style.font_family,
        *style.font_style.weight(),
        *style.font_style.width(),
        style.font_style.slant()
      ))
      .or_insert_with(|| {
        context::FONT_MGR.with(|font_mgr| {
          font_mgr
            .match_family_style(style.font_family, style.font_style)
            .unwrap()
        })
      });

    Font::new(&*typeface, style.font_size)
  })
}
