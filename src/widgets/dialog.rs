use crate::{
  Context,
  models::{Container, Glass, Rect},
};

// Settings
const SCALE_PER_SECOND: f32 = 4.0;
const MAX_GLASS_ALPHA: f32 = 0.5;
const DIALOG_SIZE: (f32, f32) = (512.0, 256.0);

pub struct Dialog {
  glass: Glass,
  container: Container,
}

impl Dialog {
  pub fn new(context: &mut Context<'_>, color: (u8, u8, u8, u8)) -> Self {
    let glass = Glass {
      size: (context.app_size.0 as _, context.app_size.1 as _),
      alpha: 0.0,
      drawable_id: u32::MAX,
    };

    let container = Container {
      position: (
        context.app_size.0 as f32 * 0.5,
        context.app_size.1 as f32 * 0.5,
      ),
      size: (0.0, 0.0),
      color,
      drawable_id: u32::MAX,
    };

    let glass = Glass {
      drawable_id: context.renderer.add_rect(Rect::from(glass)),
      ..glass
    };

    let container = Container {
      drawable_id: context.renderer.add_rect(Rect::from(container)),
      ..container
    };

    Self { glass, container }
  }

  pub fn update(self, dt: f32, context: &mut Context<'_>) -> Self {
    let glass = Glass {
      alpha: (self.glass.alpha + dt * SCALE_PER_SECOND * MAX_GLASS_ALPHA).min(MAX_GLASS_ALPHA),
      ..self.glass
    };

    let new_container_size = (
      (self.container.size.0 + dt * SCALE_PER_SECOND * DIALOG_SIZE.0).min(DIALOG_SIZE.0),
      (self.container.size.1 + dt * SCALE_PER_SECOND * DIALOG_SIZE.1).min(DIALOG_SIZE.1),
    );

    let container = Container {
      position: (
        (context.app_size.0 as f32 - new_container_size.0) * 0.5,
        (context.app_size.1 as f32 - new_container_size.1) * 0.5,
      ),
      size: new_container_size,
      ..self.container
    };

    if glass != self.glass {
      context
        .renderer
        .update_rect(self.glass.drawable_id, Rect::from(glass));
    }

    if container != self.container {
      context
        .renderer
        .update_rect(self.container.drawable_id, Rect::from(container));
    }

    Self { glass, container }
  }
}
