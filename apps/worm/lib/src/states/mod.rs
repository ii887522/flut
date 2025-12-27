pub(super) mod dialog_shown;
pub(super) mod playing;
pub(super) mod preparing;
pub(super) mod shaking;
pub(super) mod showing_dialog;

pub(super) use dialog_shown::DialogShown;
pub(super) use playing::Playing;
pub(super) use preparing::Preparing;
pub(super) use shaking::Shaking;
pub(super) use showing_dialog::ShowingDialog;

pub(super) enum State {
  Preparing(Preparing),
  Playing(Playing),
  Shaking(Shaking),
  ShowingDialog(ShowingDialog),
  DialogShown(DialogShown),
  Pending,
}
