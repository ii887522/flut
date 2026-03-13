use winit::event::{ElementState, MouseButton};

pub enum Event {
  MouseInput {
    input_state: ElementState,
    button: MouseButton,
  },
  CursorMoved {
    cursor_position: (f32, f32),
  },
  Click,
}
