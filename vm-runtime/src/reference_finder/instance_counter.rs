use crate::objects::{Array, Association};
use crate::reference_finder::{
    visit_unique_objects, visitor_next_objects, ObjectVisitor, ReferencedObject,
    VisitorAction, VisitorState,
};
use std::collections::HashMap;
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Immediate, ObjectRef};

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveInstanceCounterCountAll() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(1));
    let association_class = Smalltalk::stack_ref(StackOffset::new(0))
        .as_object()
        .unwrap();

    let instances = InstanceCounter::count_instances(start_obj);

    let mut paths_array = Array::new(instances.len()).unwrap();
    for (index, (class, amount)) in instances.into_iter().enumerate() {
        let mut association = Association::new(association_class).unwrap();
        association.set_key(class);
        association.set_value(Immediate::new_u64(amount as u64));

        paths_array.insert(index, association);
    }

    Smalltalk::method_return_value(ObjectPointer::from(paths_array.as_ptr()));
}

pub struct InstanceCounter {
    classes_map: HashMap<ObjectRef, usize>,
}

impl InstanceCounter {
    pub fn count_instances(start: AnyObjectRef) -> HashMap<ObjectRef, usize> {
        let mut counter = Self {
            classes_map: Default::default(),
        };
        visit_unique_objects(start, &mut counter);
        counter.classes_map
    }
}

impl ObjectVisitor for InstanceCounter {
    fn next_objects(object: ReferencedObject) -> impl Iterator<Item = ReferencedObject> {
        visitor_next_objects(object).filter(|each| !each.object().is_immediate())
    }

    fn visit_referenced_object(
        &mut self,
        object: ReferencedObject,
        _state: &VisitorState,
    ) -> VisitorAction {
        if let Ok(object) = object.object().as_object() {
            let class = Smalltalk::class_of_object(object);
            let entry = self.classes_map.entry(class).or_insert(0);
            *entry += 1;
        }
        VisitorAction::Continue
    }
}
