```rs
struct GamePage;

impl StatefulWidget for GamePage {
  fn build(&self, constraint: Rect) -> Widget {
    Column {
      children: [
        Spacing {
          height: 16,
        },
        Text {
          text: "6",
        },
        Spacing {
          height: 16,
        },
        Grid {
          col_count: 5,
          row_count: 5,
          gap: 2,
          builder: |i| {
            RectWidget {
              color: Color::White,
            },
          },
        },
      ],
    }
  }
}

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: GamePage,
      child_indices: [],
      buildable_indices: [],
    }),
  ],
  buildables: [],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Column {
        children: [
          Spacing {
            height: 16,
          },
          Text {
            text: "6",
          },
          Spacing {
            height: 16,
          },
          Grid {
            col_count: 5,
            row_count: 5,
            gap: 2,
            builder: |i| {
              RectWidget {
                color: Color::White,
              },
            },
          },
        ],
      },
      child_indices: [],
      buildable_indices: [0],
    }),
  ],
  buildables: [
    Some(GamePageState),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [
          StackChild {
            position: (0, 0),
            size: (100, 16),
            child: Some(Spacing {
              height: 16,
            }),
          },
          StackChild {
            position: (0, 16),
            size: (100, 32),
            child: Some(Text {
              text: "6",
            }),
          },
          StackChild {
            position: (0, 48),
            size: (100, 16),
            child: Some(Spacing {
              height: 16,
            }),
          },
          StackChild {
            position: (0, 64),
            size: (100, 100),
            child: Some(Grid {
              col_count: 5,
              row_count: 5,
              gap: 2,
              builder: |i| {
                RectWidget {
                  color: Color::White,
                },
              },
            }),
          },
        ],
      },
      child_indices: [],
      buildable_indices: [0, 1],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [],
      },
      child_indices: [1, 2, 3, 4],
      buildable_indices: [0, 1],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 16),
        child: Some(Spacing {
          height: 16,
        }),
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 16),
        size: (100, 32),
        child: Some(Text {
          text: "6",
        }),
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 48),
        size: (100, 16),
        child: Some(Spacing {
          height: 16,
        }),
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 64),
        size: (100, 100),
        child: Some(Grid {
          col_count: 5,
          row_count: 5,
          gap: 2,
          builder: |i| {
            RectWidget {
              color: Color::White,
            },
          },
        }),
      },
      child_indices: [],
      buildable_indices: [],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [],
      },
      child_indices: [1, 2, 3, 4],
      buildable_indices: [0, 1],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 16),
        child: None,
      },
      child_indices: [5],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 16),
        size: (100, 32),
        child: None,
      },
      child_indices: [6],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 48),
        size: (100, 16),
        child: None,
      },
      child_indices: [7],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 64),
        size: (100, 100),
        child: None,
      },
      child_indices: [8],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Text {
        text: "6",
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Grid {
        col_count: 5,
        row_count: 5,
        gap: 2,
        builder: |i| {
          RectWidget {
            color: Color::White,
          },
        },
      },
      child_indices: [],
      buildable_indices: [],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [],
      },
      child_indices: [1, 2, 3, 4],
      buildable_indices: [0, 1],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 16),
        child: None,
      },
      child_indices: [5],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 16),
        size: (100, 32),
        child: None,
      },
      child_indices: [6],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 48),
        size: (100, 16),
        child: None,
      },
      child_indices: [7],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 64),
        size: (100, 100),
        child: None,
      },
      child_indices: [8],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Text {
        text: "6",
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Stack {
        children: [
          StackChild {
            position: (0, 0),
            size: (100, 100),
            child: RectWidget {
              color: Color::White,
            },
          },
          StackChild {
            position: (0, 0),
            size: (100, 100),
            child: RectWidget {
              color: Color::White,
            },
          },
          StackChild {
            position: (0, 0),
            size: (100, 100),
            child: RectWidget {
              color: Color::White,
            },
          },
        ],
      },
      child_indices: [],
      buildable_indices: [2],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
    Some(Grid),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [],
      },
      child_indices: [1, 2, 3, 4],
      buildable_indices: [0, 1],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 16),
        child: None,
      },
      child_indices: [5],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 16),
        size: (100, 32),
        child: None,
      },
      child_indices: [6],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 48),
        size: (100, 16),
        child: None,
      },
      child_indices: [7],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 64),
        size: (100, 100),
        child: None,
      },
      child_indices: [8],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Text {
        text: "6",
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Stack {
        children: [],
      },
      child_indices: [9, 10, 11],
      buildable_indices: [2],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: RectWidget {
          color: Color::White,
        },
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: RectWidget {
          color: Color::White,
        },
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: RectWidget {
          color: Color::White,
        },
      },
      child_indices: [],
      buildable_indices: [],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
    Some(Grid),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

  | (build)
  v

WidgetTree {
  widget_nodes: [
    Some(WidgetNode {  <-- root
      widget: Stack {
        children: [],
      },
      child_indices: [1, 2, 3, 4],
      buildable_indices: [0, 1],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 16),
        child: None,
      },
      child_indices: [5],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 16),
        size: (100, 32),
        child: None,
      },
      child_indices: [6],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 48),
        size: (100, 16),
        child: None,
      },
      child_indices: [7],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 64),
        size: (100, 100),
        child: None,
      },
      child_indices: [8],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Text {
        text: "6",
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Spacing {
        height: 16,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: Stack {
        children: [],
      },
      child_indices: [9, 10, 11],
      buildable_indices: [2],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: None,
      },
      child_indices: [12],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: None,
      },
      child_indices: [13],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: StackChild {
        position: (0, 0),
        size: (100, 100),
        child: None,
      },
      child_indices: [14],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: RectWidget {
        color: Color::White,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: RectWidget {
        color: Color::White,
      },
      child_indices: [],
      buildable_indices: [],
    }),
    Some(WidgetNode {
      widget: RectWidget {
        color: Color::White,
      },
      child_indices: [],
      buildable_indices: [],
    }),
  ],
  buildables: [
    Some(GamePageState),
    Some(Column),
    Some(Grid),
  ],
  empty_widget_node_indices: [],
  empty_buildable_indices: [],
}

init() {
  // Insert WidgetNode that contains root widget into widget_nodes
  // build(constraint)
}

build(constraint) {
  // maybe_buildable_widget_node_indices = [0]
  //
  // Take each widget at maybe_buildable_widget_node_indices from widget_nodes
  //   If the widget is StatelessWidget:
  //     ...
  //
  //   If the widget is StatefulWidget:
  //     state = widget.new_state()
  //     Insert StateNode that contains state into state_nodes
  //     new_widget = state.build(constraint)
  //     Insert new_widget into the widget node
  //     Set the state node widget_index
  //     Insert the widget into buildable_widgets
  //     Insert widget index into buildable_widget_indices
  //     Insert widget index into maybe_buildable_widget_node_indices
  //
  //   If the widget is PainterWidget:
  //     ...
  //
  //   If the widget is Stack:
  //     ...
  //
  //   If the widget is StackChild:
  //     ...
}

process_event(event) {

}

update(dt) {

}

draw(canvas, constraint) {

}
```
