use std::ffi::CString;

pub(crate) struct StringSlice {
  c_strings: Vec<CString>,
  raw_strings: Vec<*const i8>,
}

impl From<&[&str]> for StringSlice {
  fn from(slice: &[&str]) -> Self {
    let c_strings = slice
      .iter()
      .map(|&string| CString::new(string).unwrap())
      .collect::<Vec<_>>();

    let raw_strings = c_strings.iter().map(|c_string| c_string.as_ptr()).collect();

    Self {
      c_strings,
      raw_strings,
    }
  }
}

impl StringSlice {
  pub(crate) fn len(&self) -> usize {
    self.c_strings.len()
  }

  pub(crate) fn as_ptr(&self) -> *const *const i8 {
    self.raw_strings.as_ptr()
  }
}
