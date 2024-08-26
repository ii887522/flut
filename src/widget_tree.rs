use crate::widgets::{Buildable, Widget};
use rayon::prelude::*;
use sdl2::event::Event;
use skia_safe::{Canvas, Rect};
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub(super) struct WidgetTree<'a> {
  widget_nodes: Vec<Option<WidgetNode<'a>>>,
  buildables: Vec<Option<Buildable<'a>>>,
  empty_widget_node_indices: Vec<u32>,
  empty_buildable_indices: Vec<u32>,
}

#[derive(Debug)]
struct WidgetNode<'a> {
  constraint: Rect,
  widget: Widget<'a>,
  child_indices: Vec<u32>,
  buildable_indices: Vec<u32>,
}

impl WidgetTree<'_> {
  pub(super) fn new<'a>(root: Option<Widget<'a>>, constraint: Rect) -> WidgetTree<'a> {
    let mut this = WidgetTree {
      widget_nodes: vec![],
      buildables: vec![],
      empty_widget_node_indices: vec![],
      empty_buildable_indices: vec![],
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
        Widget::Stateless(mut widget) => {
          widget_node.widget = widget.build(widget_node.constraint);

          if let Some(buildable_index) = self.empty_buildable_indices.pop() {
            widget_node.buildable_indices.push(buildable_index);
            self.buildables[buildable_index as usize] = Some(Buildable::Stateless(widget));
          } else {
            widget_node
              .buildable_indices
              .push(self.buildables.len() as _);

            self.buildables.push(Some(Buildable::Stateless(widget)));
          }

          widget_node_index_lifo_q.push(widget_node_index);
        }
        Widget::Stateful(widget) => {
          let state = widget.new_state();
          widget_node.widget = state.build(widget_node.constraint);

          if let Some(buildable_index) = self.empty_buildable_indices.pop() {
            widget_node.buildable_indices.push(buildable_index);
            self.buildables[buildable_index as usize] = Some(Buildable::Stateful(state));
          } else {
            widget_node
              .buildable_indices
              .push(self.buildables.len() as _);

            self.buildables.push(Some(Buildable::Stateful(state)));
          }

          widget_node_index_lifo_q.push(widget_node_index);
        }
        Widget::Stack(mut stack) => {
          for stack_child in stack.children.drain(..) {
            let stack_child_node = WidgetNode {
              constraint: Rect::from_xywh(
                stack_child.position.0,
                stack_child.position.1,
                stack_child.size.0,
                stack_child.size.1,
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
        Widget::Painter(_) => {
          // Painter widget can be drawn directly, no expand logic needed
        }
      }

      // Put the widget_node back into the tree so that this widget_node stay at the same position in the tree
      // as like not moved before
      self.widget_nodes[widget_node_index as usize] = Some(widget_node);
    }
  }

  pub(super) fn process_event(&mut self, event: &Event) {
    for buildable in self.buildables.iter_mut().flatten() {
      if let Buildable::Stateful(state) = buildable {
        state.process_event(event);
      }
    }
  }

  pub(super) fn update(&mut self, dt: f32) {
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
        if let Buildable::Stateful(state) =
          self.buildables[buildable_index as usize].as_mut().unwrap()
        {
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
            .empty_buildable_indices
            .reserve(buildable_indices.len());

          for buildable_index in buildable_indices {
            self.buildables[buildable_index as usize] = None;
            self.empty_buildable_indices.push(buildable_index);
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
        match self.buildables[buildable_index as usize].as_ref().unwrap() {
          Buildable::Stateless(widget) => {
            widget.pre_draw(canvas, widget_node.constraint);
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
            widget_node_index_lifo_q.par_extend(widget_node.child_indices.par_iter());
          }
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
          match self.buildables[buildable_index as usize].as_ref().unwrap() {
            Buildable::Stateless(widget) => {
              widget.post_draw(canvas, parent_widget_node.constraint);
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
