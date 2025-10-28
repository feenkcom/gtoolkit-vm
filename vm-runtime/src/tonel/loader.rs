use crate::assign_field;
use crate::objects::{Array, ArrayRef, ByteStringRef};
use crate::vm;
use std::path::PathBuf;
use std::ops::{Deref, DerefMut};
use thiserror::Error;
use tonel::MethodType;
use tonel_loader::{
    DependencyKind, DependencyReason, LoadInstruction, LoadPrecondition, MethodOwnerKind,
    build_load_plan,
};
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Object, ObjectRef, RawObjectPointer};

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelLoadPlan {
    this: Object,
    package_name: AnyObjectRef,
    instructions: ArrayRef,
    preconditions: ArrayRef,
}

impl TonelLoadPlan {
    pub fn set_package_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.package_name, value);
    }

    pub fn set_instructions(&mut self, value: ArrayRef) {
        assign_field!(self, self.instructions, value);
    }

    pub fn set_preconditions(&mut self, value: ArrayRef) {
        assign_field!(self, self.preconditions, value);
    }
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelLoadInstruction {
    this: Object,
    kind: AnyObjectRef,
    name: AnyObjectRef,
    path: AnyObjectRef,
    detail: AnyObjectRef,
}

impl TonelLoadInstruction {
    pub fn set_kind(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.kind, value);
    }

    pub fn set_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.name, value);
    }

    pub fn set_path(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.path, value);
    }

    pub fn set_detail(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.detail, value);
    }
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelMethodDefinition {
    this: Object,
    selector: AnyObjectRef,
    owner_name: AnyObjectRef,
    owner_kind: AnyObjectRef,
    class_name: AnyObjectRef,
    method_type: AnyObjectRef,
    category: AnyObjectRef,
    source: AnyObjectRef,
    header: AnyObjectRef,
    body: AnyObjectRef,
    source_path: AnyObjectRef,
}

impl TonelMethodDefinition {
    pub fn set_selector(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.selector, value);
    }

    pub fn set_owner_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.owner_name, value);
    }

    pub fn set_owner_kind(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.owner_kind, value);
    }

    pub fn set_class_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.class_name, value);
    }

    pub fn set_method_type(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.method_type, value);
    }

    pub fn set_category(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.category, value);
    }

    pub fn set_source(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.source, value);
    }

    pub fn set_header(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.header, value);
    }

    pub fn set_body(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.body, value);
    }

    pub fn set_source_path(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.source_path, value);
    }
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelLoadPrecondition {
    this: vm_object_model::Object,
    required_name: AnyObjectRef,
    required_kind: AnyObjectRef,
    reason: AnyObjectRef,
    dependent_kind: AnyObjectRef,
    dependent_name: AnyObjectRef,
    dependent_source_path: AnyObjectRef,
}

impl TonelLoadPrecondition {
    pub fn set_required_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.required_name, value);
    }

    pub fn set_required_kind(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.required_kind, value);
    }

    pub fn set_reason(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.reason, value);
    }

    pub fn set_dependent_kind(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.dependent_kind, value);
    }

    pub fn set_dependent_name(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.dependent_name, value);
    }

    pub fn set_dependent_source_path(&mut self, value: AnyObjectRef) {
        assign_field!(self, self.dependent_source_path, value);
    }
}

#[derive(Debug, Error)]
pub enum TonelPrimitiveError {
    #[error("Wrong number of arguments: {0}")]
    WrongNumberOfArguments(usize),
    #[error("Expected a ByteString for the tonel package path")]
    ExpectedByteString,
    #[error("Tonel loader error")]
    Loader(#[from] tonel_loader::LoaderError),
    #[error("Object model error")]
    ObjectModel(#[from] vm_object_model::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveTonelBuildLoadPlan() -> Result<(), TonelPrimitiveError> {
    const EXPECTED_ARGUMENTS: usize = 5;
    let argument_count = Smalltalk::method_argument_count();
    if argument_count != EXPECTED_ARGUMENTS {
        return Err(TonelPrimitiveError::WrongNumberOfArguments(
            argument_count,
        ));
    }

    let proxy = vm().proxy();

    let path_argument =
        Smalltalk::stack_object_value_unchecked(StackOffset::new(4));
    let path_object =
        AnyObjectRef::from(RawObjectPointer::from(path_argument.as_i64()));
    let path_byte_string =
        ByteStringRef::try_from(path_object).map_err(|_| TonelPrimitiveError::ExpectedByteString)?;
    let package_path = PathBuf::from(path_byte_string.as_str());

    let plan_class = class_argument(StackOffset::new(3))?;
    let instruction_class = class_argument(StackOffset::new(2))?;
    let method_definition_class = class_argument(StackOffset::new(1))?;
    let precondition_class = class_argument(StackOffset::new(0))?;

    let load_plan = build_load_plan(package_path)?;

    let mut plan_object =
        Smalltalk::instantiate::<TonelLoadPlanRef>(plan_class)?;

    let mut instruction_array =
        Array::new(load_plan.instructions().len())?;
    build_instructions(
        &proxy,
        load_plan.instructions(),
        &mut instruction_array,
        instruction_class,
        method_definition_class,
    )?;

    let mut preconditions_array =
        Array::new(load_plan.preconditions().len())?;
    build_preconditions(
        &proxy,
        load_plan.preconditions(),
        &mut preconditions_array,
        precondition_class,
    )?;

    let package_name = load_plan
        .package_name()
        .map(|value| string_object(&proxy, value))
        .unwrap_or_else(Smalltalk::nil_object);
    plan_object.set_package_name(package_name);
    plan_object.set_instructions(instruction_array);
    plan_object.set_preconditions(preconditions_array);

    let plan_any: AnyObjectRef = plan_object.into();
    let plan_pointer = ObjectPointer::from(plan_any.as_ptr());
    Smalltalk::method_return_value(plan_pointer);

    Ok(())
}

fn build_instructions(
    proxy: &vm_bindings::InterpreterProxy,
    instructions: &[LoadInstruction],
    instruction_array: &mut ArrayRef,
    instruction_class: ObjectRef,
    method_definition_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, instruction) in instructions.iter().enumerate() {
        let mut instruction_object =
            Smalltalk::instantiate::<TonelLoadInstructionRef>(
                instruction_class,
            )?;

        match instruction {
            LoadInstruction::Trait(document) => {
                let kind = string_object(proxy, "trait");
                let name = string_object(proxy, document.name());
                let path = string_object(
                    proxy,
                    document
                        .source_path()
                        .to_string_lossy()
                        .as_ref(),
                );

                instruction_object.set_kind(kind);
                instruction_object.set_name(name);
                instruction_object.set_path(path);
                instruction_object.set_detail(Smalltalk::nil_object());
            }
            LoadInstruction::Class(document) => {
                let kind = string_object(proxy, "class");
                let name = string_object(proxy, document.name());
                let path = string_object(
                    proxy,
                    document
                        .source_path()
                        .to_string_lossy()
                        .as_ref(),
                );

                instruction_object.set_kind(kind);
                instruction_object.set_name(name);
                instruction_object.set_path(path);
                instruction_object.set_detail(Smalltalk::nil_object());
            }
            LoadInstruction::Extension(document) => {
                let kind = string_object(proxy, "extension");
                let name = string_object(proxy, document.target_name());
                let path = string_object(
                    proxy,
                    document
                        .source_path()
                        .to_string_lossy()
                        .as_ref(),
                );

                instruction_object.set_kind(kind);
                instruction_object.set_name(name);
                instruction_object.set_path(path);
                instruction_object.set_detail(Smalltalk::nil_object());
            }
            LoadInstruction::Method(document) => {
                let kind = string_object(proxy, "method");
                let name = string_object(proxy, document.identifier());
                let path = string_object(
                    proxy,
                    document
                        .source_path()
                        .to_string_lossy()
                        .as_ref(),
                );

                let method_detail = build_method_detail(
                    proxy,
                    document,
                    method_definition_class,
                )?;
                let method_detail_any: AnyObjectRef = method_detail.into();

                instruction_object.set_kind(kind);
                instruction_object.set_name(name);
                instruction_object.set_path(path);
                instruction_object.set_detail(method_detail_any);
            }
        }

        let instruction_any: AnyObjectRef = instruction_object.into();
        instruction_array.insert(index, instruction_any);
    }

    Ok(())
}

fn build_method_detail(
    proxy: &vm_bindings::InterpreterProxy,
    document: &tonel_loader::MethodDocument,
    method_definition_class: ObjectRef,
) -> Result<TonelMethodDefinitionRef, TonelPrimitiveError> {
    let mut method_object =
        Smalltalk::instantiate::<TonelMethodDefinitionRef>(
            method_definition_class,
        )?;

    let definition = document.definition();
    method_object.set_selector(string_object(proxy, definition.selector.as_str()));
    method_object.set_owner_name(string_object(proxy, document.owner_name()));
    method_object.set_owner_kind(string_object(proxy, match document.owner_kind() {
            MethodOwnerKind::Trait => "trait",
            MethodOwnerKind::Class => "class",
            MethodOwnerKind::Extension => "extension",
        }));
    method_object.set_class_name(string_object(proxy, definition.class_name.as_str()));
    method_object.set_method_type(string_object(proxy, match definition.method_type {
            MethodType::Instance => "instance",
            MethodType::Class => "class",
        }));
    method_object.set_category(optional_string_object(proxy, definition.category.as_deref()));
    method_object.set_source(string_object(proxy, definition.source.as_str()));
    method_object.set_header(string_object(proxy, definition.header.as_str()));
    method_object.set_body(string_object(proxy, definition.body.as_str()));
    method_object.set_source_path(string_object(
        proxy,
        document
            .source_path()
            .to_string_lossy()
            .as_ref(),
    ));

    Ok(method_object)
}

fn build_preconditions(
    proxy: &vm_bindings::InterpreterProxy,
    preconditions: &[LoadPrecondition],
    preconditions_array: &mut ArrayRef,
    precondition_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, precondition) in preconditions.iter().enumerate() {
        let mut precondition_object =
            Smalltalk::instantiate::<TonelLoadPreconditionRef>(
                precondition_class,
            )?;

        precondition_object
            .set_required_name(string_object(proxy, precondition.required_name.as_str()));
        precondition_object.set_required_kind(string_object(
            proxy,
            dependency_kind_name(precondition.required_kind),
        ));
        precondition_object
            .set_reason(string_object(proxy, dependency_reason_name(precondition.reason)));

        match &precondition.dependent {
            tonel_loader::DependentEntity::Trait { name } => {
                precondition_object.set_dependent_kind(string_object(proxy, "trait"));
                precondition_object.set_dependent_name(string_object(proxy, name.as_str()));
                precondition_object
                    .set_dependent_source_path(Smalltalk::nil_object());
            }
            tonel_loader::DependentEntity::Class { name } => {
                precondition_object.set_dependent_kind(string_object(proxy, "class"));
                precondition_object.set_dependent_name(string_object(proxy, name.as_str()));
                precondition_object
                    .set_dependent_source_path(Smalltalk::nil_object());
            }
            tonel_loader::DependentEntity::Extension {
                target_name,
                source_path,
            } => {
                precondition_object.set_dependent_kind(string_object(proxy, "extension"));
                precondition_object
                    .set_dependent_name(string_object(proxy, target_name.as_str()));
                precondition_object.set_dependent_source_path(string_object(
                    proxy,
                    source_path.to_string_lossy().as_ref(),
                ));
            }
        }

        let precondition_any: AnyObjectRef = precondition_object.into();
        preconditions_array.insert(index, precondition_any);
    }

    Ok(())
}

fn string_object(proxy: &vm_bindings::InterpreterProxy, value: &str) -> AnyObjectRef {
    let oop = proxy.new_string(value);
    AnyObjectRef::from(RawObjectPointer::from(oop.as_i64()))
}

fn optional_string_object(
    proxy: &vm_bindings::InterpreterProxy,
    value: Option<&str>,
) -> AnyObjectRef {
    match value {
        Some(text) => string_object(proxy, text),
        None => Smalltalk::nil_object(),
    }
}

fn dependency_kind_name(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Class => "class",
        DependencyKind::Trait => "trait",
        DependencyKind::ClassOrTrait => "classOrTrait",
    }
}

fn dependency_reason_name(reason: DependencyReason) -> &'static str {
    match reason {
        DependencyReason::Superclass => "superclass",
        DependencyReason::TraitComposition => "traitComposition",
        DependencyReason::ClassTraitComposition => "classTraitComposition",
        DependencyReason::ExtensionTarget => "extensionTarget",
    }
}

fn class_argument(offset: StackOffset) -> Result<ObjectRef, TonelPrimitiveError> {
    let class_pointer = Smalltalk::stack_object_value_unchecked(offset);
    let class_any =
        AnyObjectRef::from(RawObjectPointer::from(class_pointer.as_i64()));
    Ok(class_any.as_object()?)
}
