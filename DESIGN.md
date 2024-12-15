```rs
App {
  title: "Worm",
  size: (660, 720),
  favicon_file_path: "assets/worm/images/favicon.png",
  child: GamePage,
}

App {
  title: "Worm",
  size: (660, 720),
  favicon_file_path: "assets/worm/images/favicon.png",
  child: Grid {
    col_count: 41,
    row_count: 41,
    gap: 2.0,
    builder: |index| {
      RectWidget {
        color: Color::DARK_GRAY,
      }
    },
  },
}

App {
  title: "Worm",
  size: (660, 720),
  favicon_file_path: "assets/worm/images/favicon.png",
  child: Stack {
    children: vec![
      StackChild {
        position: (x, y),
        size: (w, h),
        child: RectWidget {
          color: Color::DARK_GRAY,
        },
      },
      StackChild {
        position: (x, y),
        size: (w, h),
        child: RectWidget {
          color: Color::DARK_GRAY,
        },
      },
      StackChild {
        position: (x, y),
        size: (w, h),
        child: RectWidget {
          color: Color::DARK_GRAY,
        },
      },
    ],
  },
}

impl<'a> BuilderWidget<'a> for GamePage {
  fn process_event(&mut self, event: Event) {
    todo!();
  }

  fn update(&mut self, dt: f32) -> bool {
    todo!();
    false
  }

  fn pre_draw(&self, canvas: &Canvas) {
    todo!();
  }

  fn build(&self) -> Widget<'a> {
    Grid {
      col_count: 41,
      row_count: 41,
      gap: 2.0,
      builder: |index| {
        RectWidget {
          color: Color::DARK_GRAY,
        }
      },
    }
  }

  fn post_draw(&self, canvas: &Canvas) {
    todo!();
  }
}

impl<'a> BuilderWidget<'a> for Grid {
  fn process_event(&mut self, event: Event) {
    todo!();
  }

  fn update(&mut self, dt: f32) -> bool {
    todo!();
    false
  }

  fn pre_draw(&self, canvas: &Canvas) {
    todo!();
  }

  fn build(&self) -> Widget<'a> {
    Stack {
      children: vec![
        StackChild {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
        StackChild {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
        StackChild {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
      ],
    }
  }

  fn post_draw(&self, canvas: &Canvas) {
    todo!();
  }
}

impl PainterWidget for RectWidget {
  fn draw(&self, canvas: &Canvas) {
    todo!()
  }
}

WidgetTree<Building> {
  buildable_nodes: vec![],
  free_buildable_indices: vec![],
  expandable_nodes: vec![
    BuilderNode {
      buildable_indices: vec![],
      parent: None,
      widget: GamePage,
    },
  ],
  stack_nodes: vec![],
  free_stack_indices: vec![],
  drawable_nodes: vec![],
  free_drawable_indices: vec![],
}
// Can be accessed by build()

WidgetTree<Building> {
  buildable_nodes: vec![
    BuildableNode {
      is_mouse_over: false,
      mouse_downed_btn: MouseButton::Unknown,
      widget: GamePage,
    },
  ],
  free_buildable_indices: vec![],
  expandable_nodes: vec![
    BuilderNode {
      buildable_indices: vec![0],
      parent: None,
      widget: Grid {
        col_count: 41,
        row_count: 41,
        gap: 2.0,
        builder: |index| {
          RectWidget {
            color: Color::DARK_GRAY,
          }
        },
      },
    },
  ],
  stack_nodes: vec![],
  free_stack_indices: vec![],
  drawable_nodes: vec![],
  free_drawable_indices: vec![],
}
// Can be accessed by build()

WidgetTree<Building> {
  buildable_nodes: vec![
    BuildableNode {
      is_mouse_over: false,
      mouse_downed_btn: MouseButton::Unknown,
      widget: GamePage,
    },
    BuildableNode {
      is_mouse_over: false,
      mouse_downed_btn: MouseButton::Unknown,
      widget: Grid,
    },
  ],
  free_buildable_indices: vec![],
  expandable_nodes: vec![
    StackNode {
      buildable_indices: vec![0, 1],
      parent: None,
      children: vec![
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: RectWidget {
            color: Color::DARK_GRAY,
          },
        },
      ],
    },
  ],
  stack_nodes: vec![],
  free_stack_indices: vec![],
  drawable_nodes: vec![],
  free_drawable_indices: vec![],
}
// Can be accessed by build()

WidgetTree<Built> {
  buildable_nodes: vec![
    BuildableNode {
      is_mouse_over: false,
      mouse_downed_btn: MouseButton::Unknown,
      widget: GamePage,
    },
    BuildableNode {
      is_mouse_over: false,
      mouse_downed_btn: MouseButton::Unknown,
      widget: Grid,
    },
  ],
  free_buildable_indices: vec![],
  expandable_nodes: (),
  stack_nodes: vec![
    StackNode {
      buildable_indices: vec![0, 1],
      parent: None,
      children: vec![
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: Painter(0),
        },
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: Painter(1),
        },
        StackChildNode {
          position: (x, y),
          size: (w, h),
          child: Painter(2),
        },
      ],
    },
  ],
  free_stack_indices: vec![],
  painter_nodes: vec![
    PainterNode {
      buildable_indices: vec![],
      parent: (0, 0),
      widget: RectWidget {
        color: Color::DARK_GRAY,
      },
    },
    PainterNode {
      buildable_indices: vec![],
      parent: (0, 1),
      widget: RectWidget {
        color: Color::DARK_GRAY,
      },
    },
    PainterNode {
      buildable_indices: vec![],
      parent: (0, 2),
      widget: RectWidget {
        color: Color::DARK_GRAY,
      },
    },
  ],
  free_drawable_indices: vec![],
}
// Can be accessed by process_event(event)
// Can be accessed by update(dt)
// Can be accessed by draw(canvas)
```
