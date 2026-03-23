use winit::event::{ElementState, MouseButton};

#[derive(Clone, Copy)]
pub enum Event {
  MouseInput {
    input_state: ElementState,
    button: MouseButton,
  },
  MouseMoved {
    mouse_position: (f32, f32),
  },
  Click,
}
