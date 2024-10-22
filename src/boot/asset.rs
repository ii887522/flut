use super::context;
use crate::models::AssetReq;
use skia_safe::{Data, Image};
use std::sync::mpsc::Receiver;

pub(crate) fn main(rx: Receiver<AssetReq>) {
  for req in rx {
    match req {
      AssetReq::LoadImage(file_path) => {
        if context::IMAGES.get(file_path).is_some() {
          continue;
        }

        let image_data = Data::from_filename(file_path).unwrap();
        let image = Image::from_encoded(image_data).unwrap();
        context::IMAGES.insert(file_path, image);
      }
    }
  }
}
