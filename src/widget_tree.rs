use crate::{
  boot::context,
  widgets::{Buildable, Widget},
};
use rayon::prelude::*;
use sdl2::{event::Event, mouse::MouseButton, EventPump};
use skia_safe::{Canvas, Contains, Point, Rect};
use std::{
  collections::{HashMap, VecDeque},
  fmt::Debug,
  sync::atomic::Ordering,
};

#[derive(Debug)]
pub(super) struct WidgetTree<'a> {
  app_size: (f32, f32),
  widget_nodes: Vec<Option<WidgetNode<'a>>>,
  buildable_nodes: Vec<Option<BuildableNode<'a>>>,
  empty_widget_node_indices: Vec<u32>,
  empty_buildable_node_indices: Vec<u32>,
}

#[derive(Debug)]
struct WidgetNode<'a> {
  constraint: Rect,
  widget: Widget<'a>,
  child_indices: Vec<u32>,
  buildable_indices: Vec<u32>,
}

#[derive(Debug)]
struct BuildableNode<'a> {
  buildable: Buildable<'a>,
  is_mouse_on_this: bool,
  downed_mouse_button: MouseButton,
}

impl WidgetTree<'_> {
  pub(super) fn new<'a>(
    root: Option<Widget<'a>>,
    constraint: Rect,
    app_size: (f32, f32),
  ) -> WidgetTree<'a> {
    let mut this = WidgetTree {
      app_size,
      widget_nodes: vec![],
      buildable_nodes: vec![],
      empty_widget_node_indices: vec![],
      empty_buildable_node_indices: vec![],
    };

    if let Some(root) = root {
      this.widget_nodes.push(Some(WidgetNode {
        widget: root,
        child_indices: vec![],
        buildable_indices: vec![],
        constraint,
      }));

      // Expand the root widget
      this.build(0);
    }

    this
  }

  fn build(&mut self, widget_node_index: u32) {
    let mut widget_node_index_lifo_q = vec![widget_node_index];

    while let Some(widget_node_index) = widget_node_index_lifo_q.pop() {
      // We shouldn't take this widget_node out of the tree. But to avoid multiple mutable borrows in this loop,
      // we take out at the beginning and put it back at the end of the loop
      let mut widget_node = self.widget_nodes[widget_node_index as usize]
        .take()
        .unwrap();

      match widget_node.widget {
        Widget::Stateless(widget) => {
          widget_node.widget = widget.lock().unwrap().build(widget_node.constraint);

          if let Some(buildable_index) = self.empty_buildable_node_indices.pop() {
            widget_node.buildable_indices.push(buildable_index);

            self.buildable_nodes[buildable_index as usize] = Some(BuildableNode {
              buildable: Buildable::Stateless(widget),
              is_mouse_on_this: false,
              downed_mouse_button: MouseButton::Unknown,
            });
          } else {
            widget_node
              .buildable_indices
              .push(self.buildable_nodes.len() as _);

            self.buildable_nodes.push(Some(BuildableNode {
              buildable: Buildable::Stateless(widget),
              is_mouse_on_this: false,
              downed_mouse_button: MouseButton::Unknown,
            }));
          }

          widget_node_index_lifo_q.push(widget_node_index);
        }
        Widget::Stateful(widget) => {
          let mut state = widget.lock().unwrap().new_state();
          widget_node.widget = state.build(widget_node.constraint);

          if let Some(buildable_index) = self.empty_buildable_node_indices.pop() {
            widget_node.buildable_indices.push(buildable_index);

            self.buildable_nodes[buildable_index as usize] = Some(BuildableNode {
              buildable: Buildable::Stateful(state),
              is_mouse_on_this: false,
              downed_mouse_button: MouseButton::Unknown,
            });
          } else {
            widget_node
              .buildable_indices
              .push(self.buildable_nodes.len() as _);

            self.buildable_nodes.push(Some(BuildableNode {
              buildable: Buildable::Stateful(state),
              is_mouse_on_this: false,
              downed_mouse_button: MouseButton::Unknown,
            }));
          }

          widget_node_index_lifo_q.push(widget_node_index);
        }
        Widget::Stack(stack) => {
          for stack_child in stack.lock().unwrap().children.drain(..) {
            let stack_child_position = stack_child.get_position();
            let stack_child_size = stack_child.get_size();

            let stack_child_node = WidgetNode {
              constraint: Rect::from_xywh(
                stack_child_position.0,
                stack_child_position.1,
                stack_child_size.0,
                stack_child_size.1,
              ),
              widget: Widget::StackChild(Box::new(stack_child)),
              child_indices: vec![],
              buildable_indices: vec![],
            };

            if let Some(widget_node_index) = self.empty_widget_node_indices.pop() {
              widget_node.child_indices.push(widget_node_index);
              widget_node_index_lifo_q.push(widget_node_index);
              self.widget_nodes[widget_node_index as usize] = Some(stack_child_node);
            } else {
              let widget_node_index = self.widget_nodes.len() as _;
              widget_node.child_indices.push(widget_node_index);
              widget_node_index_lifo_q.push(widget_node_index);
              self.widget_nodes.push(Some(stack_child_node));
            }
          }

          widget_node.widget = Widget::Stack(stack);
        }
        Widget::StackChild(mut stack_child) => {
          if let Some(stack_child) = stack_child.child.take() {
            let stack_child_node = WidgetNode {
              widget: stack_child,
              child_indices: vec![],
              buildable_indices: vec![],
              constraint: widget_node.constraint,
            };

            if let Some(widget_node_index) = self.empty_widget_node_indices.pop() {
              widget_node.child_indices.push(widget_node_index);
              widget_node_index_lifo_q.push(widget_node_index);
              self.widget_nodes[widget_node_index as usize] = Some(stack_child_node);
            } else {
              let widget_node_index = self.widget_nodes.len() as _;
              widget_node.child_indices.push(widget_node_index);
              widget_node_index_lifo_q.push(widget_node_index);
              self.widget_nodes.push(Some(stack_child_node));
            }
          }

          widget_node.widget = Widget::StackChild(stack_child);
        }
        Widget::Painter(widget) => {
          // Painter widget can be drawn directly, no expand logic needed
          widget_node.widget = Widget::Painter(widget);
        }
      }

      // Put the widget_node back into the tree so that this widget_node stay at the same position in the tree
      // as like not moved before
      self.widget_nodes[widget_node_index as usize] = Some(widget_node);
    }
  }

  pub(super) fn process_event(&mut self, event: &Event) {
    let mut widget_node_index_fifo_q = VecDeque::new();

    if !self.widget_nodes.is_empty() {
      widget_node_index_fifo_q.push_back(0);
    }

    while let Some(widget_node_index) = widget_node_index_fifo_q.pop_front() {
      let widget_node = self.widget_nodes[widget_node_index as usize]
        .as_ref()
        .unwrap();

      let mut is_mouse_on_widget = false;
      let mut is_consume_event = false;

      match event {
        Event::MouseMotion { x, y, .. }
        | Event::MouseButtonDown { x, y, .. }
        | Event::MouseButtonUp { x, y, .. } => {
          if widget_node.constraint.contains(Point::new(
            *x as f32 * context::DRAWABLE_SIZE.0.load(Ordering::Relaxed) / self.app_size.0,
            *y as f32 * context::DRAWABLE_SIZE.1.load(Ordering::Relaxed) / self.app_size.1,
          )) {
            is_mouse_on_widget = true;
          }
        }
        _ => {}
      }

      for &buildable_index in &widget_node.buildable_indices {
        let buildable_node = self.buildable_nodes[buildable_index as usize]
          .as_mut()
          .unwrap();

        is_consume_event |= match event {
          Event::MouseMotion { x, y, .. } => {
            buildable_node.on_mouse_move((*x as _, *y as _), is_mouse_on_widget)
          }
          Event::MouseButtonDown {
            mouse_btn, x, y, ..
          } => buildable_node.on_mouse_down((*x as _, *y as _), *mouse_btn, is_mouse_on_widget),
          Event::MouseButtonUp {
            mouse_btn, x, y, ..
          } => buildable_node.on_mouse_up((*x as _, *y as _), *mouse_btn),
          event => buildable_node.process_event(event),
        };
      }

      if !is_consume_event {
        // Traverse the expanded widget tree in breadth-first order
        widget_node_index_fifo_q.par_extend(widget_node.child_indices.par_iter());
      }
    }
  }

  pub(super) fn update(&mut self, dt: f32, event_pump: &EventPump) {
    let mut widget_node_index_fifo_q = VecDeque::new();

    if !self.widget_nodes.is_empty() {
      widget_node_index_fifo_q.push_back(0);
    }

    while let Some(widget_node_index) = widget_node_index_fifo_q.pop_front() {
      // We shouldn't take this widget_node out of the tree. But to avoid multiple mutable borrows in this loop,
      // we take out at the beginning and put it back at the end of the loop
      let mut widget_node = self.widget_nodes[widget_node_index as usize]
        .take()
        .unwrap();

      let mut is_widget_rebuilt = false;

      for (index, &buildable_index) in widget_node.buildable_indices.iter().enumerate() {
        let buildable_node = self.buildable_nodes[buildable_index as usize]
          .as_mut()
          .unwrap();

        if let Buildable::Stateful(state) = &mut buildable_node.buildable {
          if !state.update(dt) {
            // Widget state not changed, thus can be reused
            continue;
          }

          // Widget state changed, need to rebuild
          widget_node.widget = state.build(widget_node.constraint);
          let mut child_indices = widget_node.child_indices.par_drain(..).collect::<Vec<_>>();

          let mut buildable_indices = widget_node
            .buildable_indices
            .par_drain(index + 1..)
            .collect::<Vec<_>>();

          while let Some(child_index) = child_indices.pop() {
            let widget_node = self.widget_nodes[child_index as usize].take().unwrap();
            self.empty_widget_node_indices.push(child_index);
            child_indices.par_extend(widget_node.child_indices);
            buildable_indices.par_extend(widget_node.buildable_indices);
          }

          self
            .empty_buildable_node_indices
            .reserve(buildable_indices.len());

          for buildable_index in buildable_indices {
            let mut buildable_node = self.buildable_nodes[buildable_index as usize]
              .take()
              .unwrap();

            // This widget will be removed and if mouse cursor still hover on this, it is considered as cursor leaving
            // this widget (mouse_out)
            buildable_node.on_mouse_out(event_pump, self.app_size);

            self.empty_buildable_node_indices.push(buildable_index);
          }

          is_widget_rebuilt = true;
          break;
        }
      }

      if !is_widget_rebuilt {
        // Traverse the expanded widget tree in breadth-first order
        widget_node_index_fifo_q.par_extend(widget_node.child_indices.par_iter());
      }

      // Put the widget_node back into the tree so that this widget_node stay at the same position in the tree
      // as like not moved before
      self.widget_nodes[widget_node_index as usize] = Some(widget_node);

      if is_widget_rebuilt {
        self.build(widget_node_index);
      }
    }
  }

  pub(super) fn draw(&self, canvas: &Canvas) {
    let mut widget_node_index_lifo_q = vec![];
    let mut last_child_to_parent = HashMap::new();

    if !self.widget_nodes.is_empty() {
      widget_node_index_lifo_q.push(0);
    }

    while let Some(widget_node_index) = widget_node_index_lifo_q.pop() {
      let widget_node = self.widget_nodes[widget_node_index as usize]
        .as_ref()
        .unwrap();

      for &buildable_index in &widget_node.buildable_indices {
        let buildable_node = self.buildable_nodes[buildable_index as usize]
          .as_ref()
          .unwrap();

        match &buildable_node.buildable {
          Buildable::Stateless(widget) => {
            widget
              .lock()
              .unwrap()
              .pre_draw(canvas, widget_node.constraint);
          }
          Buildable::Stateful(state) => {
            state.pre_draw(canvas, widget_node.constraint);
          }
        }
      }

      match &widget_node.widget {
        Widget::Stack(_) => {
          if let Some(&last_child_index) = widget_node.child_indices.last() {
            last_child_to_parent.insert(last_child_index, widget_node_index);

            // Traverse the expanded widget tree in depth-first order
            widget_node_index_lifo_q.par_extend(widget_node.child_indices.par_iter().rev());
          }

          // Skip the below post_draw() code as there are still more stack_child children yet to be drawn
          continue;
        }
        Widget::StackChild(_) => {
          if let Some(&first_child_index) = widget_node.child_indices.first() {
            if let Some(parent_index) = last_child_to_parent.remove(&widget_node_index) {
              last_child_to_parent.insert(first_child_index, parent_index);
            }

            widget_node_index_lifo_q.push(first_child_index);
          }
        }
        Widget::Painter(widget) => widget.draw(canvas, widget_node.constraint),
        widget => {
          // By right after expanded all widgets, should no longer have buildable widgets like StatelessWidget,
          // StatefulWidget, else something wrong in the build logic
          panic!("Cannot draw widget: {widget:?}");
        }
      }

      let mut last_child_index = widget_node_index;

      while let Some(parent_index) = last_child_to_parent.remove(&last_child_index) {
        let parent_widget_node = self.widget_nodes[parent_index as usize].as_ref().unwrap();

        for &buildable_index in parent_widget_node.buildable_indices.iter().rev() {
          let buildable_node = self.buildable_nodes[buildable_index as usize]
            .as_ref()
            .unwrap();

          match &buildable_node.buildable {
            Buildable::Stateless(widget) => {
              widget
                .lock()
                .unwrap()
                .post_draw(canvas, parent_widget_node.constraint);
            }
            Buildable::Stateful(state) => {
              state.post_draw(canvas, parent_widget_node.constraint);
            }
          }
        }

        last_child_index = parent_index;
      }
    }
  }
}

impl BuildableNode<'_> {
  fn on_mouse_move(&mut self, mouse_position: (f32, f32), is_mouse_on_this: bool) -> bool {
    if let Buildable::Stateful(state) = &mut self.buildable {
      let is_consume_event = if !self.is_mouse_on_this && is_mouse_on_this {
        // Have to use | instead of || because || will short-circuit meaning that if state.on_mouse_over() consume event,
        // state.on_mouse_hover() will not get called which is not expected.
        //
        // When event is consumed, all children of this widget will not process the event. But all associated
        // state.on_xxx() of this widget will still get called regardless of event consumption.
        state.on_mouse_over(mouse_position) | state.on_mouse_hover(mouse_position)
      } else if self.is_mouse_on_this && !is_mouse_on_this {
        self.downed_mouse_button = MouseButton::Unknown;
        state.on_mouse_out(mouse_position)
      } else if is_mouse_on_this {
        state.on_mouse_hover(mouse_position)
      } else {
        false
      };

      self.is_mouse_on_this = is_mouse_on_this;
      return is_consume_event;
    }

    false
  }

  // This method is called when this widget is removed and mouse cursor still hover on this
  fn on_mouse_out(&mut self, event_pump: &EventPump, app_size: (f32, f32)) {
    if !self.is_mouse_on_this {
      return;
    }

    if let Buildable::Stateful(state) = &mut self.buildable {
      let mouse_state = event_pump.mouse_state();

      state.on_mouse_out((
        mouse_state.x() as f32 * context::DRAWABLE_SIZE.0.load(Ordering::Relaxed) / app_size.0,
        mouse_state.y() as f32 * context::DRAWABLE_SIZE.1.load(Ordering::Relaxed) / app_size.1,
      ));

      self.is_mouse_on_this = false;
    }
  }

  fn on_mouse_down(
    &mut self,
    mouse_position: (f32, f32),
    mouse_button: MouseButton,
    is_mouse_on_this: bool,
  ) -> bool {
    if !is_mouse_on_this {
      return false;
    }

    if let Buildable::Stateful(state) = &mut self.buildable {
      self.downed_mouse_button = mouse_button;
      return state.on_mouse_down(mouse_position, mouse_button);
    }

    false
  }

  fn on_mouse_up(&mut self, mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != self.downed_mouse_button {
      return false;
    }

    if let Buildable::Stateful(state) = &mut self.buildable {
      self.downed_mouse_button = MouseButton::Unknown;
      return state.on_mouse_up(mouse_position, mouse_button);
    }

    false
  }

  fn process_event(&mut self, event: &Event) -> bool {
    if let Buildable::Stateful(state) = &mut self.buildable {
      return state.process_event(event);
    }

    false
  }
}
