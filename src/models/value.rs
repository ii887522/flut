#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Value<'a> {
  Bool(bool),
  Uint(u32),
  Int(i32),
  Float(f32),
  Str(&'a str),
}

impl Value<'_> {
  pub fn is_for_singular(&self) -> bool {
    match self {
      Value::Bool(value) => *value,
      Value::Uint(value) => *value == 1,
      Value::Int(value) => value.abs() == 1,
      Value::Float(value) => value.abs() == 1.0,
      Value::Str(_) => false,
    }
  }

  pub fn is_for_plural(&self) -> bool {
    match self {
      Value::Bool(value) => !*value,
      Value::Uint(value) => *value != 1,
      Value::Int(value) => value.abs() != 1,
      Value::Float(value) => value.abs() != 1.0,
      Value::Str(_) => false,
    }
  }
}

impl<'a> From<Value<'a>> for String {
  fn from(value: Value<'a>) -> Self {
    match value {
      Value::Bool(value) => value.to_string(),
      Value::Uint(value) => value.to_string(),
      Value::Int(value) => value.to_string(),
      Value::Float(value) => value.to_string(),
      Value::Str(value) => value.to_string(),
    }
  }
}
