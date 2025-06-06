use std::collections::HashSet;
use vm_object_model::{AnyObjectRef, ObjectRef};

use crate::objects::ArrayRef;
use crate::reference_finder::object_iterator::GraphNode;
use crate::reference_finder::{
    method_return_path, method_return_paths, visit_objects, visitor_next_objects, ObjectVisitor,
    ReferencedObject, VisitorAction, VisitorState,
};
use std::fmt::Debug;
use std::hash::Hash;
use vm_bindings::{Smalltalk, StackOffset};

#[allow(non_snake_case)]
pub extern "C" fn primitiveReferenceFinderFindAllPaths() -> Result<(), anyhow::Error> {
    let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(2));

    let classes_to_ignore: Result<Vec<_>, _> =
        classes.iter().map(|each| each.as_object()).collect();
    let paths = ReferenceFinder::new(target_obj)
        .with_all_paths(true)
        .with_ignored_classes(classes_to_ignore?)
        .find(start_obj);
    method_return_paths(paths, classes)
}

#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindPath() -> Result<(), anyhow::Error> {
    let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(2));

    let classes_to_ignore: Result<Vec<_>, _> =
        classes.iter().map(|each| each.as_object()).collect();
    let path = ReferenceFinder::new(target_obj)
        .with_all_paths(false)
        .with_ignored_classes(classes_to_ignore?)
        .find(start_obj)
        .pop()
        .unwrap_or(vec![]);
    method_return_path(path, classes)
}

#[allow(non_snake_case)]
pub fn primitiveReferenceFinderGetNeighbours() -> Result<(), anyhow::Error> {
    let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    method_return_path(start_obj.neighbors().collect(), classes)
}

#[allow(non_snake_case)]
pub fn primitiveClassInstanceReferenceFinderFindAllPaths() -> Result<(), anyhow::Error> {
    let target_class = Smalltalk::stack_ref(StackOffset::new(3)).as_object()?;
    let start_obj = Smalltalk::stack_ref(StackOffset::new(2));
    let path_len = Smalltalk::stack_integer_value(StackOffset::new(1)) as usize;
    let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;

    let paths = ClassInstanceReferenceFinder::find_all_paths(start_obj, target_class, path_len);
    method_return_paths(paths, classes)
}

#[allow(non_snake_case)]
pub fn primitiveClassInstanceReferenceFinderFindPath() -> Result<(), anyhow::Error> {
    let target_class = Smalltalk::stack_ref(StackOffset::new(2)).as_object()?;
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    let classes = ArrayRef::try_from(Smalltalk::stack_ref(StackOffset::new(0)))?;

    let paths = ClassInstanceReferenceFinder::find_path(start_obj, target_class).unwrap_or(vec![]);
    method_return_path(paths, classes)
}

pub struct ReferenceFinder {
    target: AnyObjectRef,
    find_all_paths: bool,
    classes_to_ignore: HashSet<ObjectRef>,
    paths: Vec<Vec<ReferencedObject>>,
}

impl ReferenceFinder {
    pub fn new(target: AnyObjectRef) -> Self {
        Self {
            target,
            find_all_paths: false,
            paths: vec![],
            classes_to_ignore: Default::default(),
        }
    }

    pub fn find_path(start: AnyObjectRef, target: AnyObjectRef) -> Option<Vec<ReferencedObject>> {
        Self::new(target).find(start).pop()
    }

    pub fn find_all_paths(start: AnyObjectRef, target: AnyObjectRef) -> Vec<Vec<ReferencedObject>> {
        Self::new(target).with_all_paths(true).find(start)
    }

    pub fn with_all_paths(mut self, find_all_paths: bool) -> Self {
        self.find_all_paths = find_all_paths;
        self
    }

    pub fn with_ignored_classes<T: IntoIterator<Item = ObjectRef>>(mut self, classes: T) -> Self {
        self.classes_to_ignore.extend(classes);
        self
    }

    pub fn find(mut self, start: AnyObjectRef) -> Vec<Vec<ReferencedObject>> {
        visit_objects(start, &mut self);
        self.paths
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
        if !self.classes_to_ignore.is_empty() {
            if let Ok(object) = object.object().as_object() {
                if self
                    .classes_to_ignore
                    .contains(&Smalltalk::class_of_object(object))
                {
                    return VisitorAction::Skip;
                }
            }
        }

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
    find_all_paths: bool,
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
            find_all_paths: true,
            paths: Default::default(),
        };
        visit_objects(start, &mut finder);
        finder.paths.into_iter().collect()
    }

    pub fn find_path(start: AnyObjectRef, class: ObjectRef) -> Option<Vec<ReferencedObject>> {
        let mut finder = Self {
            class,
            path_len: 0,
            find_all_paths: false,
            paths: Default::default(),
        };
        visit_objects(start, &mut finder);
        finder
            .paths
            .into_iter()
            .collect::<Vec<Vec<ReferencedObject>>>()
            .pop()
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
                return if self.find_all_paths {
                    self.paths
                        .insert(state.path_with_limited(object, self.path_len));
                    VisitorAction::Skip
                } else {
                    self.paths.insert(state.path_with(object));
                    VisitorAction::Stop
                };
            }
        }
        VisitorAction::Continue
    }
}
