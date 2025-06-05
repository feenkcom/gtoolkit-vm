use std::collections::HashSet;
use std::error::Error;
use vm_object_model::{AnyObjectRef, ObjectRef};

use crate::objects::{Array, ArrayRef};
use crate::reference_finder::object_iterator::GraphNode;
use crate::reference_finder::{
    convert_referenced_object_paths, method_return_path, method_return_paths, visit_objects,
    visitor_next_objects, ObjectVisitor, ReferencedObject, VisitorAction, VisitorState,
};
use anyhow::anyhow;
use std::fmt::Debug;
use std::hash::Hash;
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindAllPaths() {
    fn find_all_path() -> Result<(), anyhow::Error> {
        let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;
        let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
        let target_obj = Smalltalk::stack_ref(StackOffset::new(2));

        let paths = ReferenceFinder::find_all_paths(start_obj, target_obj);
        let paths = convert_referenced_object_paths(paths, classes)?;

        Smalltalk::method_return_value(ObjectPointer::from(paths.as_ptr()));
        Ok(())
    }

    find_all_path().unwrap_or_else(|error| error!("{}", error));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindPath() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(1));

    let path = ReferenceFinder::find_path(start_obj, target_obj).unwrap_or(vec![]);
    method_return_path(path);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderGetNeighbours() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    method_return_path(start_obj.neighbors().collect());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveClassInstanceReferenceFinderFindAllPaths() {
    fn find_all_path() -> Result<(), anyhow::Error> {
        let target_class = Smalltalk::stack_ref(StackOffset::new(3)).as_object()?;
        let start_obj = Smalltalk::stack_ref(StackOffset::new(2));
        let path_len = Smalltalk::stack_integer_value(StackOffset::new(1)) as usize;
        let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;

        let paths = ClassInstanceReferenceFinder::find_all_paths(start_obj, target_class, path_len);
        let paths = convert_referenced_object_paths(paths, classes)?;

        Smalltalk::method_return_value(ObjectPointer::from(paths.as_ptr()));
        Ok(())
    }

    find_all_path().unwrap_or_else(|error| error!("{}", error));
}

pub struct ReferenceFinder {
    target: AnyObjectRef,
    find_all_paths: bool,
    paths: Vec<Vec<ReferencedObject>>,
}

impl ReferenceFinder {
    pub fn find_path(start: AnyObjectRef, target: AnyObjectRef) -> Option<Vec<ReferencedObject>> {
        let mut finder = Self {
            target,
            find_all_paths: false,
            paths: vec![],
        };
        visit_objects(start, &mut finder);
        finder.paths.pop()
    }

    pub fn find_all_paths(start: AnyObjectRef, target: AnyObjectRef) -> Vec<Vec<ReferencedObject>> {
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
    fn next_objects(object: ReferencedObject) -> impl Iterator<Item = ReferencedObject> {
        visitor_next_objects(object)
            .filter(|each| !each.object().is_immediate())
            .filter(|each| !each.object().amount_of_indexable_units() != 0)
    }

    fn visit_referenced_object(
        &mut self,
        object: ReferencedObject,
        state: &VisitorState,
    ) -> VisitorAction {
        if object.object() == self.target {
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
    paths: HashSet<Vec<ReferencedObject>>,
}

impl ClassInstanceReferenceFinder {
    pub fn find_all_paths(
        start: AnyObjectRef,
        class: ObjectRef,
        path_len: usize,
    ) -> Vec<Vec<ReferencedObject>> {
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
    fn next_objects(object: ReferencedObject) -> impl Iterator<Item = ReferencedObject> {
        visitor_next_objects(object).filter(|each| !each.object().is_immediate())
    }

    fn visit_referenced_object(
        &mut self,
        object: ReferencedObject,
        state: &VisitorState,
    ) -> VisitorAction {
        if let Ok(this_object) = object.object().as_object() {
            let class = Smalltalk::class_of_object(this_object);
            if class == self.class {
                self.paths
                    .insert(state.path_with_limited(object, self.path_len));
            }
        }
        VisitorAction::Continue
    }
}
