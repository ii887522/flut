use crate::{
  collections::SparseVec,
  widgets::{BuilderWidget, PainterWidget, Stack, StackChild, Widget},
};
use atomic_refcell::AtomicRefCell;
use rayon::prelude::*;
use sdl2::{event::Event, mouse::MouseButton};
use skia_safe::{Canvas, Contains, Point, Rect};
use std::{mem, sync::Arc};

pub(super) trait StackChildChild {}
impl StackChildChild for Widget<'_> {}
impl StackChildChild for DrawableIndex {}

pub(super) trait WidgetTreeState<'a> {
  type ExpandableNodes;
}

impl<'a> WidgetTreeState<'a> for Building {
  type ExpandableNodes = Vec<ExpandableNode<'a>>;
}

impl WidgetTreeState<'_> for Built {
  type ExpandableNodes = ();
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Building;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Built;

struct BuildableNode<'a> {
  mouse_downed_btn: MouseButton,
  is_mouse_over: bool,
  widget: Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>,
}

pub(super) struct BuilderNode<'a> {
  buildable_indices: Vec<u32>,
  parent: Option<(u32, u32)>,
  widget: Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>,
}

pub(super) struct StackNode<Child: StackChildChild> {
  buildable_indices: Vec<u32>,
  parent: Option<(u32, u32)>,
  children: Vec<StackChildNode<Child>>,
}

struct StackChildNode<Child: StackChildChild> {
  position: (f32, f32),
  size: (f32, f32),
  child: Child,
}

impl<'a> From<StackChild<'a>> for StackChildNode<Widget<'a>> {
  fn from(stack_child: StackChild<'a>) -> Self {
    Self {
      position: stack_child.position,
      size: stack_child.size,
      child: stack_child.child,
    }
  }
}

impl<'a> From<&StackChild<'a>> for StackChildNode<Widget<'a>> {
  fn from(stack_child: &StackChild<'a>) -> Self {
    StackChild::clone(stack_child).into()
  }
}

impl<'a> From<StackChildNode<Widget<'a>>> for StackChildNode<DrawableIndex> {
  fn from(stack_child: StackChildNode<Widget<'a>>) -> Self {
    Self {
      position: stack_child.position,
      size: stack_child.size,
      child: DrawableIndex::invalid(),
    }
  }
}

struct PainterNode<'a> {
  buildable_indices: Vec<u32>,
  parent: Option<(u32, u32)>,
  widget: Arc<dyn PainterWidget + 'a + Send + Sync>,
}

impl<'a> From<Arc<dyn PainterWidget + 'a + Send + Sync>> for PainterNode<'a> {
  fn from(widget: Arc<dyn PainterWidget + 'a + Send + Sync>) -> Self {
    Self {
      buildable_indices: vec![],
      parent: None,
      widget,
    }
  }
}

pub(super) enum ExpandableNode<'a> {
  Builder(BuilderNode<'a>),
  Stack(StackNode<Widget<'a>>),
}

impl<'a> From<Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>> for ExpandableNode<'a> {
  fn from(widget: Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>) -> Self {
    Self::Builder(BuilderNode {
      buildable_indices: vec![],
      parent: None,
      widget,
    })
  }
}

impl<'a> From<Arc<Stack<'a>>> for ExpandableNode<'a> {
  fn from(stack: Arc<Stack<'a>>) -> Self {
    Self::Stack(StackNode {
      buildable_indices: vec![],
      parent: None,
      children: stack
        .children
        .par_iter()
        .map(StackChildNode::from)
        .collect(),
    })
  }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DrawableIndex {
  Stack(u32),
  Painter(u32),
}

impl DrawableIndex {
  const fn invalid() -> Self {
    Self::Painter(u32::MAX)
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ConstrainedDrawableIndex {
  index: DrawableIndex,
  constraint: Rect,
}

pub(super) struct WidgetTree<'a, State: WidgetTreeState<'a>> {
  buildable_nodes: SparseVec<BuildableNode<'a>>,
  expandable_nodes: State::ExpandableNodes,
  stack_nodes: SparseVec<StackNode<DrawableIndex>>,
  painter_nodes: SparseVec<PainterNode<'a>>,
}

impl<'a> From<Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>>
  for WidgetTree<'a, Building>
{
  fn from(widget: Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>) -> Self {
    Self {
      buildable_nodes: SparseVec::new(),
      expandable_nodes: vec![ExpandableNode::from(widget)],
      stack_nodes: SparseVec::new(),
      painter_nodes: SparseVec::new(),
    }
  }
}

impl<'a> From<Arc<Stack<'a>>> for WidgetTree<'a, Building> {
  fn from(stack: Arc<Stack<'a>>) -> Self {
    Self {
      buildable_nodes: SparseVec::new(),
      expandable_nodes: vec![ExpandableNode::from(stack)],
      stack_nodes: SparseVec::new(),
      painter_nodes: SparseVec::new(),
    }
  }
}

impl<'a> From<Arc<dyn PainterWidget + 'a + Send + Sync>> for WidgetTree<'a, Built> {
  fn from(widget: Arc<dyn PainterWidget + 'a + Send + Sync>) -> Self {
    Self {
      buildable_nodes: SparseVec::new(),
      expandable_nodes: (),
      stack_nodes: SparseVec::new(),
      painter_nodes: SparseVec::from(PainterNode::from(widget)),
    }
  }
}

impl<'a, State: WidgetTreeState<'a>> WidgetTree<'a, State> {
  fn get_constraint(&self, parent: Option<(u32, u32)>) -> Option<Rect> {
    parent.map(|(parent_stack_index, parent_stack_child_index)| {
      let parent_stack_child =
        &self.stack_nodes[parent_stack_index].children[parent_stack_child_index as usize];

      Rect::from_xywh(
        parent_stack_child.position.0,
        parent_stack_child.position.1,
        parent_stack_child.size.0,
        parent_stack_child.size.1,
      )
    })
  }
}

impl<'a> WidgetTree<'a, Building> {
  pub(super) fn build(mut self, root_constraint: Rect) -> WidgetTree<'a, Built> {
    while let Some(expandable_node) = self.expandable_nodes.pop() {
      match expandable_node {
        ExpandableNode::Builder(builder_node) => {
          let constraint = self
            .get_constraint(builder_node.parent)
            .unwrap_or(root_constraint);

          let widget = builder_node.widget;
          let child = widget.borrow_mut().build(constraint);

          // todo: Query mouse input to restore mouse_downed_btn and is_mouse_over
          let buildable_node = BuildableNode {
            mouse_downed_btn: MouseButton::Unknown,
            is_mouse_over: false,
            widget,
          };

          let buildable_index = self.buildable_nodes.push(buildable_node);
          let mut buildable_indices = builder_node.buildable_indices;
          buildable_indices.push(buildable_index);

          match child {
            Widget::Builder(widget) => {
              let expandable_node = ExpandableNode::Builder(BuilderNode {
                buildable_indices,
                parent: builder_node.parent,
                widget,
              });

              self.expandable_nodes.push(expandable_node);
            }

            Widget::Stack(stack) => {
              let expandable_node = ExpandableNode::Stack(StackNode {
                buildable_indices,
                parent: builder_node.parent,
                children: stack
                  .children
                  .par_iter()
                  .map(StackChildNode::from)
                  .collect(),
              });

              self.expandable_nodes.push(expandable_node);
            }

            Widget::Painter(widget) => {
              let painter_node = PainterNode {
                buildable_indices,
                parent: builder_node.parent,
                widget,
              };

              let painter_index = self.painter_nodes.push(painter_node);

              if let Some((stack_index, stack_child_index)) = builder_node.parent {
                self.stack_nodes[stack_index].children[stack_child_index as usize].child =
                  DrawableIndex::Painter(painter_index);
              }
            }
          }
        }

        ExpandableNode::Stack(stack_node) => {
          let drawable_stack_node = StackNode {
            buildable_indices: stack_node.buildable_indices,
            parent: stack_node.parent,
            children: stack_node
              .children
              .par_iter()
              .map(|stack_child| StackChildNode {
                position: stack_child.position,
                size: stack_child.size,
                child: DrawableIndex::invalid(),
              })
              .collect(),
          };

          let drawable_stack_index = self.stack_nodes.push(drawable_stack_node);

          if let Some((stack_index, stack_child_index)) = stack_node.parent {
            self.stack_nodes[stack_index].children[stack_child_index as usize].child =
              DrawableIndex::Stack(drawable_stack_index);
          }

          for (stack_child_index, stack_child) in stack_node.children.into_iter().enumerate() {
            match stack_child.child {
              Widget::Builder(widget) => {
                let expandable_node = ExpandableNode::Builder(BuilderNode {
                  buildable_indices: vec![],
                  parent: Some((drawable_stack_index, stack_child_index as _)),
                  widget,
                });

                self.expandable_nodes.push(expandable_node);
              }

              Widget::Stack(stack) => {
                let expandable_node = ExpandableNode::Stack(StackNode {
                  buildable_indices: vec![],
                  parent: Some((drawable_stack_index, stack_child_index as _)),
                  children: stack
                    .children
                    .par_iter()
                    .map(StackChildNode::from)
                    .collect(),
                });

                self.expandable_nodes.push(expandable_node);
              }

              Widget::Painter(widget) => {
                let painter_node = PainterNode {
                  buildable_indices: vec![],
                  parent: Some((drawable_stack_index, stack_child_index as _)),
                  widget,
                };

                let painter_index = self.painter_nodes.push(painter_node);

                self.stack_nodes[drawable_stack_index].children[stack_child_index].child =
                  DrawableIndex::Painter(painter_index);
              }
            }
          }
        }
      }
    }

    WidgetTree {
      buildable_nodes: self.buildable_nodes,
      expandable_nodes: (),
      stack_nodes: self.stack_nodes,
      painter_nodes: self.painter_nodes,
    }
  }
}

impl<'a> WidgetTree<'a, Built> {
  pub(super) fn new(root: Widget<'a>, constraint: Rect) -> Self {
    match root {
      Widget::Builder(root) => WidgetTree::from(root).build(constraint),
      Widget::Painter(root) => WidgetTree::from(root),
      Widget::Stack(root) => WidgetTree::from(root).build(constraint),
    }
  }

  fn get_root_drawable_index(&self) -> DrawableIndex {
    if self.stack_nodes.is_empty() {
      DrawableIndex::Painter(0)
    } else {
      DrawableIndex::Stack(0)
    }
  }

  pub(super) fn process_event(
    &mut self,
    event: &Event,
    root_constraint: Rect,
    app_size: (f32, f32),
  ) {
    let mut drawable_index_lifo_q = vec![ConstrainedDrawableIndex {
      index: self.get_root_drawable_index(),
      constraint: root_constraint,
    }];

    while let Some(drawable_index) = drawable_index_lifo_q.pop() {
      let is_mouse_moved = if let &Event::MouseMotion { x, y, .. } = event {
        // Dialog size is zero but actually cover the whole app, will consider mouse always move on the dialog
        drawable_index.constraint.size().is_empty()
          || drawable_index.constraint.contains(Point::new(
            x as f32 * root_constraint.width() / app_size.0,
            y as f32 * root_constraint.height() / app_size.1,
          ))
      } else {
        false
      };

      match drawable_index.index {
        DrawableIndex::Stack(stack_index) => {
          // Temporarily take out self.stack_nodes so that we no need borrow it then we can mutably borrow self to
          // call self.drawable_process_event()
          let mut stack_nodes = mem::take(&mut self.stack_nodes);

          stack_nodes.replace_with_and_return(stack_index, |stack_node| {
            match *event {
              Event::MouseButtonDown {
                mouse_btn, x, y, ..
              } => {
                self.drawable_on_mouse_down(
                  mouse_btn,
                  (
                    x as f32 * root_constraint.width() / app_size.0,
                    y as f32 * root_constraint.height() / app_size.1,
                  ),
                  &stack_node.buildable_indices,
                );
              }

              Event::MouseButtonUp {
                mouse_btn, x, y, ..
              } => {
                self.drawable_on_mouse_up(
                  mouse_btn,
                  (
                    x as f32 * root_constraint.width() / app_size.0,
                    y as f32 * root_constraint.height() / app_size.1,
                  ),
                  &stack_node.buildable_indices,
                );
              }

              Event::MouseMotion { x, y, .. } => {
                if is_mouse_moved {
                  self.drawable_on_mouse_over(
                    (
                      x as f32 * root_constraint.width() / app_size.0,
                      y as f32 * root_constraint.height() / app_size.1,
                    ),
                    &stack_node.buildable_indices,
                  );
                } else {
                  self.drawable_on_mouse_out(
                    (
                      x as f32 * root_constraint.width() / app_size.0,
                      y as f32 * root_constraint.height() / app_size.1,
                    ),
                    &stack_node.buildable_indices,
                  );
                }
              }

              _ => {
                self.drawable_process_event(event, &stack_node.buildable_indices);
              }
            }

            let drawable_index_q =
              stack_node
                .children
                .par_iter()
                .map(|stack_child| ConstrainedDrawableIndex {
                  index: stack_child.child,
                  constraint: Rect::from_xywh(
                    stack_child.position.0,
                    stack_child.position.1,
                    stack_child.size.0,
                    stack_child.size.1,
                  ),
                });

            drawable_index_lifo_q.par_extend(drawable_index_q);
            (Some(stack_node), ())
          });

          // Put back self.stack_nodes after we are done working on it
          self.stack_nodes = stack_nodes;
        }

        DrawableIndex::Painter(painter_index) => {
          // Temporarily take out self.painter_nodes so that we no need borrow it then we can mutably borrow self to
          // call self.update_drawable()
          let mut painter_nodes = mem::take(&mut self.painter_nodes);

          painter_nodes.replace_with_and_return(painter_index, |painter_node| {
            match *event {
              Event::MouseButtonDown {
                mouse_btn, x, y, ..
              } => {
                self.drawable_on_mouse_down(
                  mouse_btn,
                  (
                    x as f32 * root_constraint.width() / app_size.0,
                    y as f32 * root_constraint.height() / app_size.1,
                  ),
                  &painter_node.buildable_indices,
                );
              }

              Event::MouseButtonUp {
                mouse_btn, x, y, ..
              } => {
                self.drawable_on_mouse_up(
                  mouse_btn,
                  (
                    x as f32 * root_constraint.width() / app_size.0,
                    y as f32 * root_constraint.height() / app_size.1,
                  ),
                  &painter_node.buildable_indices,
                );
              }

              Event::MouseMotion { x, y, .. } => {
                if is_mouse_moved {
                  self.drawable_on_mouse_over(
                    (
                      x as f32 * root_constraint.width() / app_size.0,
                      y as f32 * root_constraint.height() / app_size.1,
                    ),
                    &painter_node.buildable_indices,
                  );
                } else {
                  self.drawable_on_mouse_out(
                    (
                      x as f32 * root_constraint.width() / app_size.0,
                      y as f32 * root_constraint.height() / app_size.1,
                    ),
                    &painter_node.buildable_indices,
                  );
                }
              }

              _ => {
                self.drawable_process_event(event, &painter_node.buildable_indices);
              }
            }

            (Some(painter_node), ())
          });

          // Put back self.painter_nodes after we are done working on it
          self.painter_nodes = painter_nodes;
        }
      }
    }
  }

  fn drawable_on_mouse_down(
    &mut self,
    mouse_btn: MouseButton,
    mouse_position: (f32, f32),
    buildable_indices: &[u32],
  ) {
    for &buildable_index in buildable_indices {
      let buildable_node = &mut self.buildable_nodes[buildable_index];

      if !buildable_node.is_mouse_over {
        break;
      }

      buildable_node.mouse_downed_btn = mouse_btn;
      let mut builder_widget = buildable_node.widget.borrow_mut();
      builder_widget.on_mouse_down(mouse_btn, mouse_position);
    }
  }

  fn drawable_on_mouse_up(
    &mut self,
    mouse_btn: MouseButton,
    mouse_position: (f32, f32),
    buildable_indices: &[u32],
  ) {
    for &buildable_index in buildable_indices {
      let buildable_node = &mut self.buildable_nodes[buildable_index];

      if !buildable_node.is_mouse_over || buildable_node.mouse_downed_btn != mouse_btn {
        break;
      }

      buildable_node.mouse_downed_btn = MouseButton::Unknown;
      let mut builder_widget = buildable_node.widget.borrow_mut();
      builder_widget.on_mouse_up(mouse_btn, mouse_position);
    }
  }

  fn drawable_on_mouse_over(&mut self, mouse_position: (f32, f32), buildable_indices: &[u32]) {
    for &buildable_index in buildable_indices {
      let buildable_node = &mut self.buildable_nodes[buildable_index];

      if buildable_node.is_mouse_over {
        break;
      }

      buildable_node.is_mouse_over = true;
      let mut builder_widget = buildable_node.widget.borrow_mut();
      builder_widget.on_mouse_over(mouse_position);
    }
  }

  fn drawable_on_mouse_out(&mut self, mouse_position: (f32, f32), buildable_indices: &[u32]) {
    for &buildable_index in buildable_indices {
      let buildable_node = &mut self.buildable_nodes[buildable_index];

      if !buildable_node.is_mouse_over {
        break;
      }

      buildable_node.is_mouse_over = false;
      buildable_node.mouse_downed_btn = MouseButton::Unknown;
      let mut builder_widget = buildable_node.widget.borrow_mut();
      builder_widget.on_mouse_out(mouse_position);
    }
  }

  fn drawable_process_event(&mut self, event: &Event, buildable_indices: &[u32]) {
    for &buildable_index in buildable_indices {
      let buildable_node = &self.buildable_nodes[buildable_index];
      let mut builder_widget = buildable_node.widget.borrow_mut();
      builder_widget.process_event(event);
    }
  }

  pub(super) fn update(mut self, dt: f32) -> WidgetTree<'a, Building> {
    let mut drawable_index_lifo_q = vec![self.get_root_drawable_index()];
    let mut expandable_nodes = vec![];

    while let Some(drawable_index) = drawable_index_lifo_q.pop() {
      match drawable_index {
        DrawableIndex::Stack(stack_index) => {
          // Temporarily take out self.stack_nodes so that we no need borrow it then we can mutably borrow self to
          // call self.update_drawable()
          let mut stack_nodes = mem::take(&mut self.stack_nodes);

          let invalid_drawable_index_q =
            stack_nodes.replace_with_and_return(stack_index, |mut stack_node| {
              if let Some(expandable_node) =
                self.update_drawable(dt, &mut stack_node.buildable_indices, stack_node.parent)
              {
                // State changed, invalidate all stack children
                expandable_nodes.push(expandable_node);

                // Invalidate all stack children after stack_nodes are available because self.invalidate() will
                // invalidate from self.stack_nodes
                let drawable_index_q = stack_node
                  .children
                  .into_par_iter()
                  .map(|stack_child| stack_child.child);

                (None, Some(drawable_index_q))
              } else {
                // No state changes, proceed to update each stack child
                // Traverse the widget tree in depth-first order to update each stack child
                let drawable_index_q = stack_node
                  .children
                  .par_iter()
                  .map(|stack_child| stack_child.child);

                drawable_index_lifo_q.par_extend(drawable_index_q);
                (Some(stack_node), None)
              }
            });

          // Put back self.stack_nodes after we are done working on it
          self.stack_nodes = stack_nodes;

          if let Some(invalid_drawable_index_q) = invalid_drawable_index_q {
            // Invalidate all stack children
            self.invalidate(invalid_drawable_index_q);
          }
        }

        DrawableIndex::Painter(painter_index) => {
          // Temporarily take out self.painter_nodes so that we no need borrow it then we can mutably borrow self to
          // call self.update_drawable()
          let mut painter_nodes = mem::take(&mut self.painter_nodes);

          painter_nodes.replace_with_and_return(painter_index, |mut painter_node| {
            if let Some(expandable_node) =
              self.update_drawable(dt, &mut painter_node.buildable_indices, painter_node.parent)
            {
              expandable_nodes.push(expandable_node);
              (None, ())
            } else {
              (Some(painter_node), ())
            }
          });

          // Put back self.painter_nodes after we are done working on it
          self.painter_nodes = painter_nodes;
        }
      }
    }

    WidgetTree {
      buildable_nodes: self.buildable_nodes,
      expandable_nodes,
      stack_nodes: self.stack_nodes,
      painter_nodes: self.painter_nodes,
    }
  }

  fn update_drawable(
    &mut self,
    dt: f32,
    buildable_indices: &mut Vec<u32>,
    parent: Option<(u32, u32)>,
  ) -> Option<ExpandableNode<'a>> {
    let dirty_buildable_index_index = buildable_indices.iter().enumerate().find_map(
      |(buildable_index_index, &buildable_index)| {
        let mut buildable_node = self.buildable_nodes[buildable_index].widget.borrow_mut();

        if buildable_node.update(dt) {
          Some(buildable_index_index)
        } else {
          None
        }
      },
    )?;

    // This is the dirty_buildable that will be rebuilt later in WidgetTree Building state
    let dirty_buildable_index = buildable_indices.swap_remove(dirty_buildable_index_index);
    let dirty_buildable_node = self.buildable_nodes.take(dirty_buildable_index);

    // The rest of buildables that are children of dirty_buildable are no use anymore since we are going to recreate
    // them anyway. It is ok to drop them
    for buildable_index in buildable_indices.drain(dirty_buildable_index_index..) {
      self.buildable_nodes.take(buildable_index);
    }

    let builder_node = ExpandableNode::Builder(BuilderNode {
      // Safe to take out buildable_indices because the drawable is no use anymore since it is going to be recreated
      // by dirty_buildable anyway.
      buildable_indices: mem::take(buildable_indices),

      parent,
      widget: dirty_buildable_node.widget,
    });

    Some(builder_node)
  }

  fn invalidate(&mut self, drawable_indices: impl ParallelIterator<Item = DrawableIndex>) {
    let mut drawable_index_lifo_q = drawable_indices.collect::<Vec<_>>();

    while let Some(drawable_index) = drawable_index_lifo_q.pop() {
      match drawable_index {
        DrawableIndex::Stack(stack_index) => {
          let stack_node = self.stack_nodes.take(stack_index);

          for &buildable_index in &stack_node.buildable_indices {
            self.buildable_nodes.take(buildable_index);
          }

          // Traverse the widget tree in depth-first order to invalidate each stack child
          let drawable_index_q = stack_node
            .children
            .into_par_iter()
            .map(|stack_child| stack_child.child);

          drawable_index_lifo_q.par_extend(drawable_index_q);
        }

        DrawableIndex::Painter(painter_index) => {
          let painter_node = self.painter_nodes.take(painter_index);

          for &buildable_index in &painter_node.buildable_indices {
            self.buildable_nodes.take(buildable_index);
          }
        }
      }
    }
  }

  pub(super) fn draw(&self, canvas: &Canvas, root_constraint: Rect) {
    let mut drawable_index_lifo_q = vec![self.get_root_drawable_index()];

    while let Some(drawable_index) = drawable_index_lifo_q.pop() {
      match drawable_index {
        DrawableIndex::Stack(stack_index) => {
          let stack_node = &self.stack_nodes[stack_index];

          let constraint = self
            .get_constraint(stack_node.parent)
            .unwrap_or(root_constraint);

          for &buildable_index in &stack_node.buildable_indices {
            let builder_widget = self.buildable_nodes[buildable_index].widget.borrow();
            builder_widget.pre_draw(canvas, constraint);
          }

          if stack_node.children.is_empty() {
            for &buildable_index in stack_node.buildable_indices.iter().rev() {
              let builder_widget = self.buildable_nodes[buildable_index].widget.borrow();
              builder_widget.post_draw(canvas, constraint);
            }

            self.post_draw(canvas, root_constraint, stack_node.parent);
            continue;
          }

          // Traverse the widget tree in depth-first order to draw each stack child
          let drawable_index_q = stack_node
            .children
            .par_iter()
            .map(|stack_child| stack_child.child)
            .rev(); // Reverse the drawable_index_q so that each stack child is drawn in the order of their declaration

          drawable_index_lifo_q.par_extend(drawable_index_q);
        }

        DrawableIndex::Painter(painter_index) => {
          let painter_node = &self.painter_nodes[painter_index];

          let constraint = self
            .get_constraint(painter_node.parent)
            .unwrap_or(root_constraint);

          for &buildable_index in &painter_node.buildable_indices {
            let builder_widget = self.buildable_nodes[buildable_index].widget.borrow();
            builder_widget.pre_draw(canvas, constraint);
          }

          painter_node.widget.draw(canvas, constraint);

          for &buildable_index in painter_node.buildable_indices.iter().rev() {
            let builder_widget = self.buildable_nodes[buildable_index].widget.borrow();
            builder_widget.post_draw(canvas, constraint);
          }

          self.post_draw(canvas, root_constraint, painter_node.parent);
        }
      }
    }
  }

  fn post_draw(&self, canvas: &Canvas, root_constraint: Rect, mut parent: Option<(u32, u32)>) {
    while let Some((parent_stack_index, parent_stack_child_index)) = parent.take() {
      let parent_stack_node = &self.stack_nodes[parent_stack_index];

      if parent_stack_child_index < parent_stack_node.children.len() as u32 - 1 {
        continue;
      }

      let constraint = self
        .get_constraint(parent_stack_node.parent)
        .unwrap_or(root_constraint);

      for &buildable_index in parent_stack_node.buildable_indices.iter().rev() {
        let builder_widget = self.buildable_nodes[buildable_index].widget.borrow();
        builder_widget.post_draw(canvas, constraint);
      }

      parent = parent_stack_node.parent;
    }
  }
}
