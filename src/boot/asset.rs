use super::context;
use crate::models::AssetTask;
use skia_safe::{Data, Image};
use std::sync::mpsc::Receiver;

pub(crate) fn main(rx: Receiver<AssetTask>) {
  for task in rx {
    match task {
      AssetTask::Load(file_path) => {
        let images = context::IMAGES.read().unwrap();

        if images.get(file_path).is_some() {
          continue;
        }

        drop(images);
        let image_data = Data::from_filename(file_path).unwrap();
        let image = Image::from_encoded(image_data).unwrap();
        let mut images = context::IMAGES.write().unwrap();
        images.insert(file_path, image);
      }
    }
  }
}
