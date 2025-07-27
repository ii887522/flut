use crate::Context;

pub struct Shake {
  interval: f32,
  duration: f32,
  strength: f32,
  acc: f32,
  t: f32,
}

impl Shake {
  pub const fn new(interval: f32, duration: f32, strength: f32) -> Self {
    Self {
      interval,
      duration,
      strength,
      acc: 0.0,
      t: 0.0,
    }
  }

  pub fn update(self, dt: f32, context: &mut Context<'_>) -> Option<Self> {
    let new_t = self.t + dt;
    let new_acc = self.acc + dt;

    let new_t = if new_t >= self.interval {
      context.renderer.set_cam_position((
        (fastrand::f32() - 0.5) * self.strength,
        (fastrand::f32() - 0.5) * self.strength,
      ));

      new_t - self.interval
    } else {
      new_t
    };

    if new_acc >= self.duration {
      context.renderer.set_cam_position((0.0, 0.0));
      return None;
    }

    Some(Self {
      acc: new_acc,
      t: new_t,
      ..self
    })
  }
}
