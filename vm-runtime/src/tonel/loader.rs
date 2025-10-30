use crate::objects::{Array, ArrayRef, ByteStringRef};
use crate::vm;
use std::path::PathBuf;
use thiserror::Error;
use tonel::MethodType;
use tonel_loader::{
    build_load_plan, BehaviorLoad, DependencyKind, DependencyReason, ExtensionLoad,
    LoadPrecondition, MethodLoad, MethodOwnerKind,
};
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelLoadPlan {
    this: Object,
    package_name: AnyObjectRef,
    behaviors: ArrayRef,
    methods: ArrayRef,
    extensions: ArrayRef,
    preconditions: ArrayRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelBehaviorLoad {
    this: Object,
    order: Immediate,
    kind: ByteStringRef,
    name: ByteStringRef,
    path: ByteStringRef,
    detail: AnyObjectRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelExtensionLoad {
    this: Object,
    order: Immediate,
    target_name: ByteStringRef,
    path: ByteStringRef,
    detail: AnyObjectRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelMethodDefinition {
    this: Object,
    selector: ByteStringRef,
    owner_name: ByteStringRef,
    owner_kind: ByteStringRef,
    class_name: ByteStringRef,
    method_type: ByteStringRef,
    category: AnyObjectRef,
    source: ByteStringRef,
    header: ByteStringRef,
    body: ByteStringRef,
    source_path: ByteStringRef,
    owner_order: Immediate,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelClassDetail {
    this: Object,
    superclass_name: AnyObjectRef,
    trait_composition: AnyObjectRef,
    class_trait_composition: AnyObjectRef,
    instance_variables: ArrayRef,
    class_variables: ArrayRef,
    class_instance_variables: ArrayRef,
    pool_dictionaries: ArrayRef,
    category: AnyObjectRef,
    package: AnyObjectRef,
    tag: AnyObjectRef,
    type_name: AnyObjectRef,
    comment: ByteStringRef,
    source_path: ByteStringRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelTraitDetail {
    this: Object,
    trait_composition: AnyObjectRef,
    class_trait_composition: AnyObjectRef,
    instance_variables: ArrayRef,
    class_instance_variables: ArrayRef,
    category: AnyObjectRef,
    package: AnyObjectRef,
    tag: AnyObjectRef,
    comment: ByteStringRef,
    source_path: ByteStringRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelExtensionDetail {
    this: Object,
    selector_names: ArrayRef,
    categories: ArrayRef,
    method_types: ArrayRef,
    source_path: ByteStringRef,
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct TonelLoadPrecondition {
    this: Object,
    required_name: ByteStringRef,
    required_kind: ByteStringRef,
    reason: ByteStringRef,
    dependent_kind: ByteStringRef,
    dependent_name: ByteStringRef,
    dependent_source_path: AnyObjectRef,
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
    const EXPECTED_ARGUMENTS: usize = 9;
    let argument_count = Smalltalk::method_argument_count();
    if argument_count != EXPECTED_ARGUMENTS {
        return Err(TonelPrimitiveError::WrongNumberOfArguments(argument_count));
    }

    let proxy = vm().proxy();

    let path_argument = Smalltalk::stack_object_value_unchecked(StackOffset::new(8));
    let path_object = AnyObjectRef::from(RawObjectPointer::from(path_argument.as_i64()));
    let path_byte_string = ByteStringRef::try_from(path_object)
        .map_err(|_| TonelPrimitiveError::ExpectedByteString)?;
    let package_path = PathBuf::from(path_byte_string.as_str());

    let plan_class = class_argument(StackOffset::new(7))?;
    let behavior_class = class_argument(StackOffset::new(6))?;
    let method_definition_class = class_argument(StackOffset::new(5))?;
    let extension_class = class_argument(StackOffset::new(4))?;
    let extension_detail_class = class_argument(StackOffset::new(3))?;
    let precondition_class = class_argument(StackOffset::new(2))?;
    let class_detail_class = class_argument(StackOffset::new(1))?;
    let trait_detail_class = class_argument(StackOffset::new(0))?;

    let load_plan = build_load_plan(package_path)?;

    let mut plan_object = Smalltalk::instantiate::<TonelLoadPlanRef>(plan_class)?;

    let mut behavior_array = Array::new(load_plan.behaviors().len())?;
    build_behaviors(
        &proxy,
        load_plan.behaviors(),
        &mut behavior_array,
        behavior_class,
        class_detail_class,
        trait_detail_class,
    )?;

    let mut method_array = Array::new(load_plan.methods().len())?;
    build_methods(
        &proxy,
        load_plan.methods(),
        &mut method_array,
        method_definition_class,
    )?;

    let mut extension_array = Array::new(load_plan.extensions().len())?;
    build_extensions(
        &proxy,
        load_plan.extensions(),
        &mut extension_array,
        extension_class,
        extension_detail_class,
    )?;

    let mut preconditions_array = Array::new(load_plan.preconditions().len())?;
    build_preconditions(
        &proxy,
        load_plan.preconditions(),
        &mut preconditions_array,
        precondition_class,
    )?;

    if let Some(value) = load_plan.package_name() {
        plan_object.set_package_name(byte_string(&proxy, value)?);
    }
    plan_object.set_behaviors(behavior_array);
    plan_object.set_methods(method_array);
    plan_object.set_extensions(extension_array);
    plan_object.set_preconditions(preconditions_array);

    let plan_any: AnyObjectRef = plan_object.into();
    let plan_pointer = ObjectPointer::from(plan_any.as_ptr());
    Smalltalk::method_return_value(plan_pointer);

    Ok(())
}

fn build_class_detail(
    proxy: &vm_bindings::InterpreterProxy,
    document: &tonel_loader::ClassDocument,
    class_detail_class: ObjectRef,
) -> Result<TonelClassDetailRef, TonelPrimitiveError> {
    let mut detail = Smalltalk::instantiate::<TonelClassDetailRef>(class_detail_class)?;

    let definition = document.definition();

    if let Some(super_name) = document.superclass_name() {
        detail.set_superclass_name(byte_string(proxy, super_name)?);
    }

    if let Some(text) = definition.trait_composition.as_deref() {
        detail.set_trait_composition(byte_string(proxy, text)?);
    }

    if let Some(text) = definition.class_trait_composition.as_deref() {
        detail.set_class_trait_composition(byte_string(proxy, text)?);
    }

    detail.set_instance_variables(byte_string_array_from_strings(
        proxy,
        &definition.inst_vars,
    )?);
    detail.set_class_variables(byte_string_array_from_strings(
        proxy,
        &definition.class_vars,
    )?);
    detail.set_class_instance_variables(byte_string_array_from_strings(
        proxy,
        &definition.class_inst_vars,
    )?);
    detail.set_pool_dictionaries(byte_string_array_from_strings(proxy, &definition.pools)?);

    if let Some(category) = definition.category.as_deref() {
        detail.set_category(byte_string(proxy, category)?);
    }
    if let Some(package) = definition.package.as_deref() {
        detail.set_package(byte_string(proxy, package)?);
    }

    if let Some(tag) = definition.tag.as_deref() {
        detail.set_tag(byte_string(proxy, tag)?);
    }

    if let Some(type_name) = definition.type_.as_deref() {
        detail.set_type_name(byte_string(proxy, type_name)?);
    }

    detail.set_comment(byte_string(proxy, definition.comment.as_str())?);

    let source_path_string = document.source_path().to_string_lossy();
    detail.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

    Ok(detail)
}

fn build_trait_detail(
    proxy: &vm_bindings::InterpreterProxy,
    document: &tonel_loader::TraitDocument,
    trait_detail_class: ObjectRef,
) -> Result<TonelTraitDetailRef, TonelPrimitiveError> {
    let mut detail = Smalltalk::instantiate::<TonelTraitDetailRef>(trait_detail_class)?;

    let definition = document.definition();

    if let Some(text) = definition.trait_composition.as_deref() {
        detail.set_trait_composition(byte_string(proxy, text)?);
    }

    if let Some(text) = definition.class_trait_composition.as_deref() {
        detail.set_class_trait_composition(byte_string(proxy, text)?);
    }

    detail.set_instance_variables(byte_string_array_from_strings(
        proxy,
        &definition.inst_vars,
    )?);
    detail.set_class_instance_variables(byte_string_array_from_strings(
        proxy,
        &definition.class_inst_vars,
    )?);

    if let Some(category) = definition.category.as_deref() {
        detail.set_category(byte_string(proxy, category)?);
    }

    if let Some(package) = definition.package.as_deref() {
        detail.set_package(byte_string(proxy, package)?);
    }
    if let Some(tag) = definition.tag.as_deref() {
        detail.set_tag(byte_string(proxy, tag)?);
    }

    detail.set_comment(byte_string(proxy, definition.comment.as_str())?);

    let source_path_string = document.source_path().to_string_lossy();
    detail.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

    Ok(detail)
}

fn build_extension_detail(
    proxy: &vm_bindings::InterpreterProxy,
    document: &tonel_loader::ExtensionDocument,
    extension_detail_class: ObjectRef,
) -> Result<TonelExtensionDetailRef, TonelPrimitiveError> {
    let mut detail = Smalltalk::instantiate::<TonelExtensionDetailRef>(extension_detail_class)?;

    let methods = document.methods();
    let selector_names: Vec<String> = methods
        .iter()
        .map(|method| method.selector.clone())
        .collect();
    detail.set_selector_names(byte_string_array_from_strings(proxy, &selector_names)?);

    let mut categories = Array::new(methods.len())?;
    let mut method_type_names = Vec::with_capacity(methods.len());

    for (index, method) in methods.iter().enumerate() {
        if let Some(category) = method.category.as_deref() {
            let category_value = byte_string(proxy, category)?;
            categories.insert(index, category_value);
        } else {
            categories.insert(index, Smalltalk::nil_object());
        }

        let method_type = match method.method_type {
            MethodType::Instance => "instance",
            MethodType::Class => "class",
        };
        method_type_names.push(method_type.to_string());
    }

    detail.set_categories(categories);
    let method_types = byte_string_array_from_strings(proxy, &method_type_names)?;
    detail.set_method_types(method_types);

    let source_path_string = document.source_path().to_string_lossy();
    detail.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

    Ok(detail)
}

fn build_preconditions(
    proxy: &vm_bindings::InterpreterProxy,
    preconditions: &[LoadPrecondition],
    preconditions_array: &mut ArrayRef,
    precondition_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, precondition) in preconditions.iter().enumerate() {
        let mut precondition_object =
            Smalltalk::instantiate::<TonelLoadPreconditionRef>(precondition_class)?;

        precondition_object
            .set_required_name(byte_string(proxy, precondition.required_name.as_str())?);
        precondition_object.set_required_kind(byte_string(
            proxy,
            dependency_kind_name(precondition.required_kind),
        )?);
        precondition_object.set_reason(byte_string(
            proxy,
            dependency_reason_name(precondition.reason),
        )?);

        match &precondition.dependent {
            tonel_loader::DependentEntity::Trait { name } => {
                precondition_object.set_dependent_kind(byte_string(proxy, "trait")?);
                precondition_object.set_dependent_name(byte_string(proxy, name.as_str())?);
            }
            tonel_loader::DependentEntity::Class { name } => {
                precondition_object.set_dependent_kind(byte_string(proxy, "class")?);
                precondition_object.set_dependent_name(byte_string(proxy, name.as_str())?);
            }
            tonel_loader::DependentEntity::Extension {
                target_name,
                source_path,
            } => {
                precondition_object.set_dependent_kind(byte_string(proxy, "extension")?);
                precondition_object.set_dependent_name(byte_string(proxy, target_name.as_str())?);
                let source_path_string = source_path.to_string_lossy();
                precondition_object
                    .set_dependent_source_path(byte_string(proxy, source_path_string.as_ref())?);
            }
        }

        let precondition_any: AnyObjectRef = precondition_object.into();
        preconditions_array.insert(index, precondition_any);
    }

    Ok(())
}

fn byte_string(
    proxy: &vm_bindings::InterpreterProxy,
    value: &str,
) -> Result<ByteStringRef, TonelPrimitiveError> {
    let oop = proxy.new_string(value);
    let any = AnyObjectRef::from(RawObjectPointer::from(oop.as_i64()));
    Ok(ByteStringRef::try_from(any)?)
}

fn byte_string_array_from_strings(
    proxy: &vm_bindings::InterpreterProxy,
    items: &[String],
) -> Result<ArrayRef, TonelPrimitiveError> {
    let mut array = Array::new(items.len())?;
    for (index, item) in items.iter().enumerate() {
        let value = byte_string(proxy, item.as_str())?;
        array.insert(index, value);
    }
    Ok(array)
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
    let class_any = AnyObjectRef::from(RawObjectPointer::from(class_pointer.as_i64()));
    Ok(class_any.as_object()?)
}
fn build_behaviors(
    proxy: &vm_bindings::InterpreterProxy,
    behaviors: &[BehaviorLoad],
    behavior_array: &mut ArrayRef,
    behavior_class: ObjectRef,
    class_detail_class: ObjectRef,
    trait_detail_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, behavior) in behaviors.iter().enumerate() {
        let mut behavior_object = Smalltalk::instantiate::<TonelBehaviorLoadRef>(behavior_class)?;

        behavior_object.set_order(behavior.order());
        behavior_object.set_name(byte_string(proxy, behavior.name())?);

        let path_string = behavior.source_path().to_string_lossy();
        behavior_object.set_path(byte_string(proxy, path_string.as_ref())?);

        if let Some(class_document) = behavior.as_class() {
            behavior_object.set_kind(byte_string(proxy, "class")?);
            let class_detail = build_class_detail(proxy, class_document, class_detail_class)?;
            behavior_object.set_detail(class_detail);
        } else if let Some(trait_document) = behavior.as_trait() {
            behavior_object.set_kind(byte_string(proxy, "trait")?);
            let trait_detail = build_trait_detail(proxy, trait_document, trait_detail_class)?;
            behavior_object.set_detail(trait_detail);
        } else {
            unreachable!("Behavior must be either class or trait");
        }

        let behavior_any: AnyObjectRef = behavior_object.into();
        behavior_array.insert(index, behavior_any);
    }

    Ok(())
}

fn build_methods(
    proxy: &vm_bindings::InterpreterProxy,
    methods: &[MethodLoad],
    method_array: &mut ArrayRef,
    method_definition_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, method) in methods.iter().enumerate() {
        let mut method_object =
            Smalltalk::instantiate::<TonelMethodDefinitionRef>(method_definition_class)?;

        method_object.set_owner_order(method.owner_order());
        method_object.set_selector(byte_string(proxy, method.definition().selector.as_str())?);
        method_object.set_owner_name(byte_string(proxy, method.owner_name())?);
        method_object.set_owner_kind(byte_string(
            proxy,
            match method.owner_kind() {
                MethodOwnerKind::Trait => "trait",
                MethodOwnerKind::Class => "class",
                MethodOwnerKind::Extension => "extension",
            },
        )?);
        method_object.set_class_name(byte_string(proxy, method.definition().class_name.as_str())?);
        method_object.set_method_type(byte_string(
            proxy,
            match method.definition().method_type {
                MethodType::Instance => "instance",
                MethodType::Class => "class",
            },
        )?);
        if let Some(category) = method.definition().category.as_deref() {
            method_object.set_category(byte_string(proxy, category)?);
        }
        method_object.set_source(byte_string(proxy, method.definition().source.as_str())?);
        method_object.set_header(byte_string(proxy, method.definition().header.as_str())?);
        method_object.set_body(byte_string(proxy, method.definition().body.as_str())?);
        let source_path_string = method.source_path().to_string_lossy();
        method_object.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

        let method_any: AnyObjectRef = method_object.into();
        method_array.insert(index, method_any);
    }

    Ok(())
}

fn build_extensions(
    proxy: &vm_bindings::InterpreterProxy,
    extensions: &[ExtensionLoad],
    extension_array: &mut ArrayRef,
    extension_class: ObjectRef,
    extension_detail_class: ObjectRef,
) -> Result<(), TonelPrimitiveError> {
    for (index, extension) in extensions.iter().enumerate() {
        let mut extension_object =
            Smalltalk::instantiate::<TonelExtensionLoadRef>(extension_class)?;

        extension_object.set_order(extension.order());
        extension_object.set_target_name(byte_string(proxy, extension.target_name())?);
        let path_string = extension.source_path().to_string_lossy();
        extension_object.set_path(byte_string(proxy, path_string.as_ref())?);
        let detail = build_extension_detail(proxy, extension.document(), extension_detail_class)?;
        extension_object.set_detail(detail);

        let extension_any: AnyObjectRef = extension_object.into();
        extension_array.insert(index, extension_any);
    }

    Ok(())
}
