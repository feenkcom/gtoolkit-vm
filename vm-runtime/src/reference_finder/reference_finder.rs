use std::collections::HashSet;
use vm_object_model::{AnyObjectRef, ObjectRef};

use crate::objects::Array;
use crate::reference_finder::object_iterator::GraphNode;
use crate::reference_finder::{
    visit_objects, visitor_next_objects, ObjectVisitor, VisitorAction, VisitorState,
};
use std::fmt::Debug;
use std::hash::Hash;
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindAllPaths() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(1));

    let paths = ReferenceFinder::find_all_paths(start_obj, target_obj);
    method_return_paths(paths);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindPath() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(1));

    let path = ReferenceFinder::find_path(start_obj, target_obj).unwrap_or(vec![]);

    let mut array = Array::new(path.len()).unwrap();
    for (index, each) in path.iter().enumerate() {
        array.insert(index, each.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(array.as_ptr()));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderGetNeighbours() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));

    let mut neighbors = start_obj.neighbors();

    let neighbors: Vec<AnyObjectRef> = neighbors.collect();
    let mut array = Array::new(neighbors.len()).unwrap();
    for (index, each) in neighbors.iter().enumerate() {
        array.insert(index, each.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(array.as_ptr()));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveClassInstanceReferenceFinderFindAllPaths() {
    let target_class = Smalltalk::stack_ref(StackOffset::new(2))
        .as_object()
        .unwrap();
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    let path_len = Smalltalk::stack_integer_value(StackOffset::new(0)) as usize;

    let paths = ClassInstanceReferenceFinder::find_all_paths(start_obj, target_class, path_len);
    method_return_paths(paths);
}

fn method_return_paths(paths: Vec<Vec<AnyObjectRef>>) {
    let mut paths_array = Array::new(paths.len()).unwrap();
    for (path_index, path) in paths.iter().enumerate() {
        let mut path_array = Array::new(path.len()).unwrap();
        for (obj_index, obj) in path.iter().enumerate() {
            path_array.insert(obj_index, obj.clone());
        }
        paths_array.insert(path_index, path_array.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(paths_array.as_ptr()));
}

pub struct ReferenceFinder {
    target: AnyObjectRef,
    find_all_paths: bool,
    paths: Vec<Vec<AnyObjectRef>>,
}

impl ReferenceFinder {
    pub fn find_path(start: AnyObjectRef, target: AnyObjectRef) -> Option<Vec<AnyObjectRef>> {
        let mut finder = Self {
            target,
            find_all_paths: false,
            paths: vec![],
        };
        visit_objects(start, &mut finder);
        finder.paths.pop()
    }

    pub fn find_all_paths(start: AnyObjectRef, target: AnyObjectRef) -> Vec<Vec<AnyObjectRef>> {
        let mut finder = Self {
            target,
            find_all_paths: true,
            paths: vec![],
        };
        visit_objects(start, &mut finder);
        finder.paths
    }
}

impl ObjectVisitor for ReferenceFinder {
    fn next_objects(object: AnyObjectRef) -> impl Iterator<Item = AnyObjectRef> {
        visitor_next_objects(object)
            .filter(|each| !each.is_immediate())
            .filter(|each| !each.amount_of_indexable_units() != 0)
    }

    fn visit_referenced_object(
        &mut self,
        object: AnyObjectRef,
        state: &VisitorState,
    ) -> VisitorAction {
        if object == self.target {
            self.paths.push(state.path_with(object));

            return if self.find_all_paths {
                VisitorAction::Skip
            } else {
                VisitorAction::Stop
            };
        }
        VisitorAction::Continue
    }
}

pub struct ClassInstanceReferenceFinder {
    class: ObjectRef,
    path_len: usize,
    paths: HashSet<Vec<AnyObjectRef>>,
}

impl ClassInstanceReferenceFinder {
    pub fn find_all_paths(
        start: AnyObjectRef,
        class: ObjectRef,
        path_len: usize,
    ) -> Vec<Vec<AnyObjectRef>> {
        let mut finder = Self {
            class,
            path_len,
            paths: Default::default(),
        };
        visit_objects(start, &mut finder);
        finder.paths.into_iter().collect()
    }
}

impl ObjectVisitor for ClassInstanceReferenceFinder {
    fn next_objects(object: AnyObjectRef) -> impl Iterator<Item = AnyObjectRef> {
        visitor_next_objects(object).filter(|each| !each.is_immediate())
    }

    fn visit_referenced_object(
        &mut self,
        object: AnyObjectRef,
        state: &VisitorState,
    ) -> VisitorAction {
        if let Ok(this_object) = object.as_object() {
            let class = Smalltalk::class_of_object(this_object);
            if class == self.class {
                self.paths
                    .insert(state.path_with_limited(object, self.path_len));
            }
        }
        VisitorAction::Continue
    }
}
