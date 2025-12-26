pub(super) mod playing;
pub(super) mod preparing;
pub(super) mod shaking;

pub(super) use playing::Playing;
pub(super) use preparing::Preparing;
pub(super) use shaking::Shaking;

pub(super) enum State {
  Preparing(Preparing),
  Playing(Playing),
  Shaking(Shaking),
  Pending,
}
