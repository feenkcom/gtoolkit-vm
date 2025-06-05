use crate::reference_finder::object_iterator::GraphNode;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use vm_object_model::AnyObjectRef;

#[inline]
pub fn visit_referenced_object<V>(
    visitor: &mut V,
    object: AnyObjectRef,
    state: &VisitorState,
) -> VisitorAction
where
    V: ObjectVisitor,
{
    visitor.visit_object(object, state)
}

#[inline]
pub fn visitor_next_objects(object: AnyObjectRef) -> impl Iterator<Item = AnyObjectRef> {
    object.neighbors()
}

pub trait ObjectVisitor {
    #[inline]
    fn next_objects(object: AnyObjectRef) -> impl Iterator<Item = AnyObjectRef> {
        visitor_next_objects(object)
    }

    #[inline]
    fn visit_object(&mut self, _object: AnyObjectRef, _state: &VisitorState) -> VisitorAction {
        VisitorAction::Continue
    }

    #[inline]
    fn visit_referenced_object(
        &mut self,
        object: AnyObjectRef,
        state: &VisitorState,
    ) -> VisitorAction
    where
        Self: Sized,
    {
        visit_referenced_object(self, object, state)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VisitorAction {
    Continue,
    Skip,
    Stop,
}

pub fn visit_objects<T: ObjectVisitor>(start: AnyObjectRef, visitor: &mut T) {
    let mut visited: HashSet<AnyObjectRef> = HashSet::new();

    let root = VisitorState {
        node: start,
        parent: None,
    };

    match visitor.visit_referenced_object(start, &root) {
        VisitorAction::Continue => {}
        VisitorAction::Skip => {
            return;
        }
        VisitorAction::Stop => {
            return;
        }
    }

    let mut buffer = vec![];
    let mut objects_left = vec![];
    objects_left.push(Rc::new(root));
    visited.insert(start);

    loop {
        buffer.clear();
        let objects = std::mem::replace(&mut objects_left, buffer);
        for state in &objects {
            match visitor.visit_object(state.node, state) {
                VisitorAction::Continue => {}
                VisitorAction::Skip => {
                    continue;
                }
                VisitorAction::Stop => {
                    return;
                }
            }
            for neighbor in T::next_objects(state.node) {
                match visitor.visit_referenced_object(neighbor, state) {
                    VisitorAction::Continue => {
                        if visited.insert(neighbor) {
                            objects_left.push(Rc::new(VisitorState {
                                node: neighbor,
                                parent: Some(state.clone()),
                            }));
                        }
                    }
                    VisitorAction::Skip => {
                        continue;
                    }
                    VisitorAction::Stop => {
                        return;
                    }
                }
            }
        }
        buffer = objects;
        if objects_left.is_empty() {
            break;
        }
    }
}

pub fn visit_unique_objects<T: ObjectVisitor>(start: AnyObjectRef, visitor: &mut T) {
    let mut visited: HashSet<AnyObjectRef> = HashSet::new();

    let root = VisitorState {
        node: start,
        parent: None,
    };

    match visitor.visit_referenced_object(start, &root) {
        VisitorAction::Continue => {}
        VisitorAction::Skip => {
            return;
        }
        VisitorAction::Stop => {
            return;
        }
    }

    let mut buffer = vec![];
    let mut objects_left = vec![];
    objects_left.push(Rc::new(root));
    visited.insert(start);

    loop {
        buffer.clear();
        let objects = std::mem::replace(&mut objects_left, buffer);
        for state in &objects {
            match visitor.visit_object(state.node, state) {
                VisitorAction::Continue => {}
                VisitorAction::Skip => {
                    continue;
                }
                VisitorAction::Stop => {
                    return;
                }
            }
            for neighbor in T::next_objects(state.node) {
                if visited.insert(neighbor) {
                    match visitor.visit_referenced_object(neighbor, state) {
                        VisitorAction::Continue => {
                            objects_left.push(Rc::new(VisitorState {
                                node: neighbor,
                                parent: Some(state.clone()),
                            }));
                        }
                        VisitorAction::Skip => {
                            continue;
                        }
                        VisitorAction::Stop => {
                            return;
                        }
                    }
                }
            }
        }
        buffer = objects;
        if objects_left.is_empty() {
            break;
        }
    }
}

#[derive(Clone, Debug)]
pub struct VisitorState {
    node: AnyObjectRef,
    parent: Option<Rc<VisitorState>>,
}

impl VisitorState {
    pub fn path(&self) -> Vec<AnyObjectRef> {
        reconstruct_path(self, vec![])
    }

    pub fn path_with(&self, object: AnyObjectRef) -> Vec<AnyObjectRef> {
        reconstruct_path(self, vec![object])
    }

    pub fn path_with_limited(&self, object: AnyObjectRef, len: usize) -> Vec<AnyObjectRef> {
        reconstruct_path_limited(self, vec![object], len)
    }
}

/// Reconstruct path from backlinks
fn reconstruct_path(mut frame: &VisitorState, mut path: Vec<AnyObjectRef>) -> Vec<AnyObjectRef> {
    loop {
        path.push(frame.node.clone());
        match &frame.parent {
            Some(parent) => frame = parent,
            None => break,
        }
    }
    path.reverse();
    path
}

fn reconstruct_path_limited(
    mut frame: &VisitorState,
    mut path: Vec<AnyObjectRef>,
    length: usize,
) -> Vec<AnyObjectRef> {
    loop {
        if path.len() >= length {
            break;
        }

        path.push(frame.node.clone());
        match &frame.parent {
            Some(parent) => frame = parent,
            None => break,
        }
    }
    path.reverse();
    path
}
