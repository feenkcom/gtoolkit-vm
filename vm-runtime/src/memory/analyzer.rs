use crate::assign_field;
use crate::memory::{EdenMemorySpace, OldMemorySpace, PastMemorySpace};
use crate::objects::{ArrayRef, Association};
use fxhash::FxHashMap;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use strum::EnumCount;
use strum_macros::EnumCount;
use vec_map::VecMap;
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef};

#[derive(Debug)]
pub struct MemoryAnalyzer {
    total_amount_of_objects: usize,
    tallies: VecMap<ClassTally>,
}

#[derive(Debug)]
pub struct ClassTally {
    class: ObjectRef,
    amount_of_objects: usize,
    total_byte_size: usize,
    details_per_space: [ClassTallyPerSpace; SpaceType::COUNT],
}

#[derive(Debug)]
pub struct ClassTallyPerSpace {
    space_type: SpaceType,
    amount_of_objects: usize,
    total_byte_size: usize,
    object_byte_sizes: FxHashMap<usize, usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, ToPrimitive, EnumCount)]
#[repr(u8)]
pub enum SpaceType {
    Eden,
    Past,
    Old,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct ClassTallyObject {
    this: Object,
    class: ObjectRef,
    amount_of_objects: Immediate,
    total_byte_size: Immediate,
    details_per_space: AnyObjectRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct ClassTallyPerSpaceObject {
    this: Object,
    space_type: Immediate,
    amount_of_objects: Immediate,
    total_byte_size: Immediate,
    byte_size_details: ArrayRef,
}

impl MemoryAnalyzer {
    pub fn sorted_tallies_by_size(&self) -> Vec<&ClassTally> {
        let mut tallies = self.tallies.values().collect::<Vec<_>>();
        tallies.sort_by(|a, b| b.total_byte_size.cmp(&a.total_byte_size));
        tallies
    }
}

impl ClassTally {
    pub fn new(class: ObjectRef) -> Self {
        Self {
            class,
            amount_of_objects: 0,
            total_byte_size: 0,
            details_per_space: [
                ClassTallyPerSpace::new(SpaceType::Eden),
                ClassTallyPerSpace::new(SpaceType::Past),
                ClassTallyPerSpace::new(SpaceType::Old),
            ],
        }
    }

    pub fn total_byte_size(&self) -> usize {
        self.total_byte_size
    }
}

impl ClassTallyPerSpace {
    pub fn new(space_type: SpaceType) -> Self {
        Self {
            space_type,
            amount_of_objects: 0,
            total_byte_size: 0,
            object_byte_sizes: Default::default(),
        }
    }

    pub fn sorted_size_details_by_size(&self) -> Vec<(usize, usize)> {
        let mut tallies = self
            .object_byte_sizes
            .iter()
            .map(|each| (*each.0, *each.1))
            .collect::<Vec<_>>();
        tallies.sort_by(|a, b| (b.0 * b.1).cmp(&(a.0 * a.1)));
        tallies
    }
}

impl ClassTallyObject {
    pub fn set_class(&mut self, class: ObjectRef) {
        assign_field!(self, self.class, class);
    }

    pub fn set_amount_of_objects(&mut self, amount_of_objects: usize) {
        self.amount_of_objects = Immediate::new_u64(amount_of_objects as u64);
    }

    pub fn set_total_byte_size(&mut self, total_byte_size: usize) {
        self.total_byte_size = Immediate::new_u64(total_byte_size as u64);
    }

    pub fn set_details_per_space(&mut self, details: impl Into<AnyObjectRef>) {
        assign_field!(self, self.details_per_space, details.into());
    }
}

impl ClassTallyPerSpaceObject {
    pub fn set_space_type(&mut self, space_type: SpaceType) {
        self.space_type = Immediate::new_u64(space_type.to_u64().unwrap());
    }

    pub fn set_amount_of_objects(&mut self, amount_of_objects: usize) {
        self.amount_of_objects = Immediate::new_u64(amount_of_objects as u64);
    }

    pub fn set_total_byte_size(&mut self, total_byte_size: usize) {
        self.total_byte_size = Immediate::new_u64(total_byte_size as u64);
    }

    pub fn set_byte_size_details(&mut self, details: ArrayRef) {
        assign_field!(self, self.byte_size_details, details);
    }
}

impl MemoryAnalyzer {
    pub fn new() -> Self {
        Self {
            total_amount_of_objects: 0,
            tallies: Default::default(),
        }
    }

    pub fn process_objects(
        &mut self,
        objects: impl Iterator<Item = ObjectRef>,
        space_type: SpaceType,
    ) {
        for object in objects {
            let class_index = object.header().class_index();
            let object_size = Smalltalk::byte_size(object);

            let tally = self
                .tallies
                .entry(class_index as usize)
                .or_insert_with(|| ClassTally::new(Smalltalk::class_of_object(object)));
            tally.amount_of_objects += 1;
            tally.total_byte_size += object_size;

            let details = &mut tally.details_per_space[space_type.to_u64().unwrap() as usize];
            details.amount_of_objects += 1;
            details.total_byte_size += object_size;

            let size_entry = details.object_byte_sizes.entry(object_size).or_insert(0);
            *size_entry += 1;

            self.total_amount_of_objects += 1;
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAnalyzeObjectMemory() -> Result<(), vm_object_model::Error> {
    let class_tally_class = Smalltalk::get_method_argument(0).as_object()?;
    let association_class = Smalltalk::get_method_argument(1).as_object()?;
    let class_tally_per_space_class = Smalltalk::get_method_argument(2).as_object()?;
    let include_per_space_details =
        Smalltalk::get_method_argument(3).as_object()? == Smalltalk::bool_object(true);

    let mut analyzer = MemoryAnalyzer::new();
    analyzer.process_objects(EdenMemorySpace::new().objects(), SpaceType::Eden);
    analyzer.process_objects(PastMemorySpace::new().objects(), SpaceType::Past);
    analyzer.process_objects(OldMemorySpace::new().objects(), SpaceType::Old);

    let mut tallies_array = Smalltalk::instantiate_indexable::<ArrayRef>(
        Smalltalk::class_array(),
        analyzer.tallies.len(),
    )?;
    for (index, tally) in analyzer.sorted_tallies_by_size().into_iter().enumerate() {
        let details_per_space = if include_per_space_details {
            tallies_per_space_to_array(
                &tally.details_per_space,
                association_class,
                class_tally_per_space_class,
            )?
            .into()
        } else {
            Smalltalk::nil_object()
        };

        let mut class_tally_object =
            Smalltalk::instantiate::<ClassTallyObjectRef>(class_tally_class)?;
        class_tally_object.set_class(tally.class);
        class_tally_object.set_amount_of_objects(tally.amount_of_objects);
        class_tally_object.set_total_byte_size(tally.total_byte_size());
        class_tally_object.set_details_per_space(details_per_space);

        tallies_array.insert(index, class_tally_object);
    }

    Smalltalk::method_return(tallies_array);
    Ok(())
}

fn tallies_per_space_to_array(
    details: &[ClassTallyPerSpace],
    association_class: ObjectRef,
    tally_per_space_class: ObjectRef,
) -> Result<ArrayRef, vm_object_model::Error> {
    let mut details_array =
        Smalltalk::instantiate_indexable::<ArrayRef>(Smalltalk::class_array(), details.len())?;

    for (index, tally) in details.iter().enumerate() {
        details_array.insert(
            index,
            tally_per_space_to_object(tally, association_class, tally_per_space_class)?,
        );
    }

    Ok(details_array)
}

fn tally_per_space_to_object(
    details: &ClassTallyPerSpace,
    association_class: ObjectRef,
    tally_per_space_class: ObjectRef,
) -> Result<ClassTallyPerSpaceObjectRef, vm_object_model::Error> {
    let mut class_tally_object =
        Smalltalk::instantiate::<ClassTallyPerSpaceObjectRef>(tally_per_space_class)?;

    let object_byte_sizes =
        object_byte_sizes_to_array(&details.sorted_size_details_by_size(), association_class)?;

    class_tally_object.set_space_type(details.space_type);
    class_tally_object.set_amount_of_objects(details.amount_of_objects);
    class_tally_object.set_total_byte_size(details.total_byte_size);
    class_tally_object.set_byte_size_details(object_byte_sizes);

    Ok(class_tally_object)
}

fn object_byte_sizes_to_array(
    sizes: &[(usize, usize)],
    association_class: ObjectRef,
) -> Result<ArrayRef, vm_object_model::Error> {
    let mut sizes_array =
        Smalltalk::instantiate_indexable::<ArrayRef>(Smalltalk::class_array(), sizes.len())?;
    for (index, (size, amount)) in sizes.iter().enumerate() {
        let mut association = Association::new(association_class)?;
        association.set_key(Immediate::new_u64(*size as u64));
        association.set_value(Immediate::new_u64(*amount as u64));
        sizes_array.insert(index, association);
    }
    Ok(sizes_array)
}
