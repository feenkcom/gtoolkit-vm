use crate::objects::{Array, ArrayRef, ByteStringRef};
use crate::vm;
use std::path::PathBuf;
use ston::Value;
use thiserror::Error;
use tonel::MethodType;
use tonel_loader::{
    build_load_plan, BehaviorLoad, ClassDocument, DependencyKind,
    DependencyReason, DependentEntity, ExtensionDocument, ExtensionLoad, LoadPrecondition,
    MethodDocument, MethodLoad, MethodOwnerKind, TraitDocument,
};
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[derive(Debug, Error)]
pub enum TonelPrimitiveError {
    #[error("Wrong number of arguments: {0}")]
    WrongNumberOfArguments(usize),
    #[error("Expected a ByteString for the tonel package path")]
    ExpectedByteString,
    #[error("Expected {expected} classes in the helper array, received {actual}")]
    InvalidClassesArrayLength { expected: usize, actual: usize },
    #[error("Missing class entry at index {0} in the helper array")]
    MissingClass(usize),
    #[error("Tonel loader error")]
    Loader(#[from] tonel_loader::LoaderError),
    #[error("Object model error")]
    ObjectModel(#[from] vm_object_model::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveTonelBuildLoadPlan() -> Result<(), TonelPrimitiveError> {
    const EXPECTED_ARGUMENTS: usize = 2;
    let argument_count = Smalltalk::method_argument_count();
    if argument_count != EXPECTED_ARGUMENTS {
        return Err(TonelPrimitiveError::WrongNumberOfArguments(argument_count));
    }

    let proxy = vm().proxy();

    let path_object = Smalltalk::get_method_argument(0);
    let path_byte_string = ByteStringRef::try_from(path_object)
        .map_err(|_| TonelPrimitiveError::ExpectedByteString)?;
    let package_path = PathBuf::from(path_byte_string.as_str());

    let classes_argument = Smalltalk::get_method_argument(1);
    let classes_array = ArrayRef::try_from(classes_argument)?;
    let classes = PharoClasses::from_array(classes_array)?;

    let load_plan = build_load_plan(package_path)?;
    let mut plan_object = plan::build_pharo_load_plan(&proxy, &load_plan, &classes)?;

    if let Some(name) = load_plan.package_name() {
        plan_object.set_package_name(byte_string(&proxy, name)?);
    }

    let plan_any: AnyObjectRef = plan_object.into();
    let plan_pointer = ObjectPointer::from(plan_any.as_ptr());
    Smalltalk::method_return_value(plan_pointer);

    Ok(())
}

struct PharoClasses {
    load_plan: ObjectRef,
    behavior_load: ObjectRef,
    class_document: ObjectRef,
    class_definition: ObjectRef,
    trait_document: ObjectRef,
    trait_definition: ObjectRef,
    extension_load: ObjectRef,
    extension_document: ObjectRef,
    method_load: ObjectRef,
    method_document: ObjectRef,
    method_definition: ObjectRef,
    load_precondition: ObjectRef,
    dependent_entity: ObjectRef,
}

impl PharoClasses {
    const EXPECTED_COUNT: usize = 13;

    fn from_array(array: ArrayRef) -> Result<Self, TonelPrimitiveError> {
        if array.len() != Self::EXPECTED_COUNT {
            return Err(TonelPrimitiveError::InvalidClassesArrayLength {
                expected: Self::EXPECTED_COUNT,
                actual: array.len(),
            });
        }

        Ok(Self {
            load_plan: class_from_array(&array, 0)?,
            behavior_load: class_from_array(&array, 1)?,
            class_document: class_from_array(&array, 2)?,
            class_definition: class_from_array(&array, 3)?,
            trait_document: class_from_array(&array, 4)?,
            trait_definition: class_from_array(&array, 5)?,
            extension_load: class_from_array(&array, 6)?,
            extension_document: class_from_array(&array, 7)?,
            method_load: class_from_array(&array, 8)?,
            method_document: class_from_array(&array, 9)?,
            method_definition: class_from_array(&array, 10)?,
            load_precondition: class_from_array(&array, 11)?,
            dependent_entity: class_from_array(&array, 12)?,
        })
    }
}

mod plan {
    use super::*;

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelLoadPlan {
        this: Object,
        package_name: AnyObjectRef,
        behaviors: ArrayRef,
        methods: ArrayRef,
        extensions: ArrayRef,
        preconditions: ArrayRef,
    }

    pub(super) fn build_pharo_load_plan(
        proxy: &vm_bindings::InterpreterProxy,
        load_plan: &tonel_loader::LoadPlan,
        classes: &PharoClasses,
    ) -> Result<TonelLoadPlanRef, TonelPrimitiveError> {
        let mut plan_object = Smalltalk::instantiate::<TonelLoadPlanRef>(classes.load_plan)?;

        let behaviors = instruction::build_behavior_loads(proxy, load_plan.behaviors(), classes)?;
        plan_object.set_behaviors(behaviors);

        let methods = instruction::build_method_loads(proxy, load_plan.methods(), classes)?;
        plan_object.set_methods(methods);

        let extensions =
            instruction::build_extension_loads(proxy, load_plan.extensions(), classes)?;
        plan_object.set_extensions(extensions);

        let preconditions =
            instruction::build_preconditions(proxy, load_plan.preconditions(), classes)?;
        plan_object.set_preconditions(preconditions);

        Ok(plan_object)
    }
}

mod instruction {
    use super::*;

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelBehaviorLoad {
        this: Object,
        order: Immediate,
        document: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelMethodLoad {
        this: Object,
        owner_order: Immediate,
        document: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelExtensionLoad {
        this: Object,
        order: Immediate,
        document: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelLoadPrecondition {
        this: Object,
        required_name: ByteStringRef,
        required_kind: ByteStringRef,
        reason: ByteStringRef,
        dependent: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelDependentEntity {
        this: Object,
        kind: ByteStringRef,
        name: ByteStringRef,
        source_path: AnyObjectRef,
    }

    pub(super) fn build_behavior_loads(
        proxy: &vm_bindings::InterpreterProxy,
        behaviors: &[BehaviorLoad],
        classes: &PharoClasses,
    ) -> Result<ArrayRef, TonelPrimitiveError> {
        let mut array = Array::new(behaviors.len())?;
        for (index, behavior) in behaviors.iter().enumerate() {
            let mut behavior_object =
                Smalltalk::instantiate::<TonelBehaviorLoadRef>(classes.behavior_load)?;
            behavior_object.set_order(behavior.order());

            let document = documents::build_behavior_document(proxy, behavior, classes)?;
            behavior_object.set_document(document);

            let behavior_any: AnyObjectRef = behavior_object.into();
            array.insert(index, behavior_any);
        }
        Ok(array)
    }

    pub(super) fn build_method_loads(
        proxy: &vm_bindings::InterpreterProxy,
        methods: &[MethodLoad],
        classes: &PharoClasses,
    ) -> Result<ArrayRef, TonelPrimitiveError> {
        let mut array = Array::new(methods.len())?;
        for (index, method) in methods.iter().enumerate() {
            let mut method_object =
                Smalltalk::instantiate::<TonelMethodLoadRef>(classes.method_load)?;
            method_object.set_owner_order(method.owner_order());

            let document = documents::build_method_document(proxy, method.document(), classes)?;
            method_object.set_document(document);

            let method_any: AnyObjectRef = method_object.into();
            array.insert(index, method_any);
        }
        Ok(array)
    }

    pub(super) fn build_extension_loads(
        proxy: &vm_bindings::InterpreterProxy,
        extensions: &[ExtensionLoad],
        classes: &PharoClasses,
    ) -> Result<ArrayRef, TonelPrimitiveError> {
        let mut array = Array::new(extensions.len())?;
        for (index, extension) in extensions.iter().enumerate() {
            let mut extension_object =
                Smalltalk::instantiate::<TonelExtensionLoadRef>(classes.extension_load)?;
            extension_object.set_order(extension.order());

            let document =
                documents::build_extension_document(proxy, extension.document(), classes)?;
            extension_object.set_document(document);

            let extension_any: AnyObjectRef = extension_object.into();
            array.insert(index, extension_any);
        }
        Ok(array)
    }

    pub(super) fn build_preconditions(
        proxy: &vm_bindings::InterpreterProxy,
        preconditions: &[LoadPrecondition],
        classes: &PharoClasses,
    ) -> Result<ArrayRef, TonelPrimitiveError> {
        let mut array = Array::new(preconditions.len())?;
        for (index, precondition) in preconditions.iter().enumerate() {
            let mut precondition_object =
                Smalltalk::instantiate::<TonelLoadPreconditionRef>(classes.load_precondition)?;

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

            let dependent = build_dependent_entity(proxy, &precondition.dependent, classes)?;
            precondition_object.set_dependent(dependent);

            let precondition_any: AnyObjectRef = precondition_object.into();
            array.insert(index, precondition_any);
        }
        Ok(array)
    }

    fn build_dependent_entity(
        proxy: &vm_bindings::InterpreterProxy,
        dependent: &DependentEntity,
        classes: &PharoClasses,
    ) -> Result<AnyObjectRef, TonelPrimitiveError> {
        let mut entity =
            Smalltalk::instantiate::<TonelDependentEntityRef>(classes.dependent_entity)?;

        match dependent {
            DependentEntity::Trait { name } => {
                entity.set_kind(byte_string(proxy, "trait")?);
                entity.set_name(byte_string(proxy, name.as_str())?);
            }
            DependentEntity::Class { name } => {
                entity.set_kind(byte_string(proxy, "class")?);
                entity.set_name(byte_string(proxy, name.as_str())?);
            }
            DependentEntity::Extension {
                target_name,
                source_path,
            } => {
                entity.set_kind(byte_string(proxy, "extension")?);
                entity.set_name(byte_string(proxy, target_name.as_str())?);
                let source_path_string = source_path.to_string_lossy();
                entity.set_source_path(byte_string(proxy, source_path_string.as_ref())?);
            }
        }

        Ok(entity.into())
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
}

mod documents {
    use super::*;

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelClassDocument {
        this: Object,
        definition: AnyObjectRef,
        methods: ArrayRef,
        source_path: ByteStringRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelTraitDocument {
        this: Object,
        definition: AnyObjectRef,
        methods: ArrayRef,
        source_path: ByteStringRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelExtensionDocument {
        this: Object,
        target_name: ByteStringRef,
        methods: ArrayRef,
        source_path: ByteStringRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelMethodDocument {
        this: Object,
        definition: AnyObjectRef,
        identifier: ByteStringRef,
        owner_name: ByteStringRef,
        owner_kind: ByteStringRef,
        source_path: ByteStringRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelClassDefinition {
        this: Object,
        name: ByteStringRef,
        superclass: AnyObjectRef,
        comment: ByteStringRef,
        trait_composition: AnyObjectRef,
        class_trait_composition: AnyObjectRef,
        instance_variables: ArrayRef,
        class_variables: ArrayRef,
        class_instance_variables: ArrayRef,
        pool_dictionaries: ArrayRef,
        package: AnyObjectRef,
        tag: AnyObjectRef,
        category: AnyObjectRef,
        type_name: AnyObjectRef,
        raw_metadata: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelTraitDefinition {
        this: Object,
        name: ByteStringRef,
        comment: ByteStringRef,
        trait_composition: AnyObjectRef,
        class_trait_composition: AnyObjectRef,
        instance_variables: ArrayRef,
        class_instance_variables: ArrayRef,
        package: AnyObjectRef,
        tag: AnyObjectRef,
        category: AnyObjectRef,
        raw_metadata: AnyObjectRef,
    }

    #[derive(Debug, PharoObject)]
    #[repr(C)]
    pub(super) struct TonelMethodDefinition {
        this: Object,
        class_name: ByteStringRef,
        method_type: ByteStringRef,
        selector: ByteStringRef,
        header: ByteStringRef,
        body: ByteStringRef,
        source: ByteStringRef,
        category: AnyObjectRef,
        raw_metadata: AnyObjectRef,
    }

    pub(super) fn build_behavior_document(
        proxy: &vm_bindings::InterpreterProxy,
        behavior: &BehaviorLoad,
        classes: &PharoClasses,
    ) -> Result<AnyObjectRef, TonelPrimitiveError> {
        if let Some(class_document) = behavior.as_class() {
            build_class_document(proxy, class_document, classes).map(AnyObjectRef::from)
        } else if let Some(trait_document) = behavior.as_trait() {
            build_trait_document(proxy, trait_document, classes).map(AnyObjectRef::from)
        } else {
            unreachable!("BehaviorLoad must reference either a class or trait document");
        }
    }

    pub(super) fn build_method_document(
        proxy: &vm_bindings::InterpreterProxy,
        document: &MethodDocument,
        classes: &PharoClasses,
    ) -> Result<TonelMethodDocumentRef, TonelPrimitiveError> {
        let mut method_document =
            Smalltalk::instantiate::<TonelMethodDocumentRef>(classes.method_document)?;

        let definition = build_method_definition(proxy, document.definition(), classes)?;
        method_document.set_definition(definition);

        method_document.set_identifier(byte_string(proxy, document.identifier())?);
        method_document.set_owner_name(byte_string(proxy, document.owner_name())?);
        method_document.set_owner_kind(byte_string(
            proxy,
            method_owner_kind_name(document.owner_kind()),
        )?);
        let source_path_string = document.source_path().to_string_lossy();
        method_document.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

        Ok(method_document)
    }

    pub(super) fn build_extension_document(
        proxy: &vm_bindings::InterpreterProxy,
        document: &ExtensionDocument,
        classes: &PharoClasses,
    ) -> Result<TonelExtensionDocumentRef, TonelPrimitiveError> {
        let mut extension_document =
            Smalltalk::instantiate::<TonelExtensionDocumentRef>(classes.extension_document)?;

        extension_document.set_target_name(byte_string(proxy, document.target_name())?);

        let method_definitions = build_method_definitions(proxy, document.methods(), classes)?;
        extension_document.set_methods(method_definitions);

        let source_path_string = document.source_path().to_string_lossy();
        extension_document.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

        Ok(extension_document)
    }

    fn build_class_document(
        proxy: &vm_bindings::InterpreterProxy,
        document: &ClassDocument,
        classes: &PharoClasses,
    ) -> Result<TonelClassDocumentRef, TonelPrimitiveError> {
        let mut class_document =
            Smalltalk::instantiate::<TonelClassDocumentRef>(classes.class_document)?;

        let definition = build_class_definition(proxy, document.definition(), classes)?;
        class_document.set_definition(definition);

        let method_definitions = build_method_definitions(proxy, document.methods(), classes)?;
        class_document.set_methods(method_definitions);

        let source_path_string = document.source_path().to_string_lossy();
        class_document.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

        Ok(class_document)
    }

    fn build_trait_document(
        proxy: &vm_bindings::InterpreterProxy,
        document: &TraitDocument,
        classes: &PharoClasses,
    ) -> Result<TonelTraitDocumentRef, TonelPrimitiveError> {
        let mut trait_document =
            Smalltalk::instantiate::<TonelTraitDocumentRef>(classes.trait_document)?;

        let definition = build_trait_definition(proxy, document.definition(), classes)?;
        trait_document.set_definition(definition);

        let method_definitions = build_method_definitions(proxy, document.methods(), classes)?;
        trait_document.set_methods(method_definitions);

        let source_path_string = document.source_path().to_string_lossy();
        trait_document.set_source_path(byte_string(proxy, source_path_string.as_ref())?);

        Ok(trait_document)
    }

    fn build_class_definition(
        proxy: &vm_bindings::InterpreterProxy,
        definition: &tonel::ClassDefinition,
        classes: &PharoClasses,
    ) -> Result<TonelClassDefinitionRef, TonelPrimitiveError> {
        let mut class_definition =
            Smalltalk::instantiate::<TonelClassDefinitionRef>(classes.class_definition)?;

        class_definition.set_name(byte_string(proxy, definition.name.as_str())?);

        if let Some(superclass) = definition.superclass.as_deref() {
            if !superclass.eq_ignore_ascii_case("nil") {
                class_definition.set_superclass(byte_string(proxy, superclass)?);
            }
        }

        class_definition.set_comment(byte_string(proxy, definition.comment.as_str())?);

        if let Some(text) = definition.trait_composition.as_deref() {
            class_definition.set_trait_composition(byte_string(proxy, text)?);
        }
        if let Some(text) = definition.class_trait_composition.as_deref() {
            class_definition.set_class_trait_composition(byte_string(proxy, text)?);
        }

        class_definition.set_instance_variables(byte_string_array_from_strings(
            proxy,
            &definition.inst_vars,
        )?);
        class_definition.set_class_variables(byte_string_array_from_strings(
            proxy,
            &definition.class_vars,
        )?);
        class_definition.set_class_instance_variables(byte_string_array_from_strings(
            proxy,
            &definition.class_inst_vars,
        )?);
        class_definition
            .set_pool_dictionaries(byte_string_array_from_strings(proxy, &definition.pools)?);

        if let Some(package) = definition.package.as_deref() {
            class_definition.set_package(byte_string(proxy, package)?);
        }
        if let Some(tag) = definition.tag.as_deref() {
            class_definition.set_tag(byte_string(proxy, tag)?);
        }
        if let Some(category) = definition.category.as_deref() {
            class_definition.set_category(byte_string(proxy, category)?);
        }
        if let Some(type_name) = definition.type_.as_deref() {
            class_definition.set_type_name(byte_string(proxy, type_name)?);
        }

        let metadata_string = ston_value_to_string(&definition.raw_metadata);
        class_definition.set_raw_metadata(byte_string(proxy, metadata_string.as_str())?);

        Ok(class_definition)
    }

    fn build_trait_definition(
        proxy: &vm_bindings::InterpreterProxy,
        definition: &tonel::TraitDefinition,
        classes: &PharoClasses,
    ) -> Result<TonelTraitDefinitionRef, TonelPrimitiveError> {
        let mut trait_definition =
            Smalltalk::instantiate::<TonelTraitDefinitionRef>(classes.trait_definition)?;

        trait_definition.set_name(byte_string(proxy, definition.name.as_str())?);
        trait_definition.set_comment(byte_string(proxy, definition.comment.as_str())?);

        if let Some(text) = definition.trait_composition.as_deref() {
            trait_definition.set_trait_composition(byte_string(proxy, text)?);
        }
        if let Some(text) = definition.class_trait_composition.as_deref() {
            trait_definition.set_class_trait_composition(byte_string(proxy, text)?);
        }

        trait_definition.set_instance_variables(byte_string_array_from_strings(
            proxy,
            &definition.inst_vars,
        )?);
        trait_definition.set_class_instance_variables(byte_string_array_from_strings(
            proxy,
            &definition.class_inst_vars,
        )?);

        if let Some(package) = definition.package.as_deref() {
            trait_definition.set_package(byte_string(proxy, package)?);
        }
        if let Some(tag) = definition.tag.as_deref() {
            trait_definition.set_tag(byte_string(proxy, tag)?);
        }
        if let Some(category) = definition.category.as_deref() {
            trait_definition.set_category(byte_string(proxy, category)?);
        }

        let metadata_string = ston_value_to_string(&definition.raw_metadata);
        trait_definition.set_raw_metadata(byte_string(proxy, metadata_string.as_str())?);

        Ok(trait_definition)
    }

    fn build_method_definitions(
        proxy: &vm_bindings::InterpreterProxy,
        methods: &[tonel::MethodDefinition],
        classes: &PharoClasses,
    ) -> Result<ArrayRef, TonelPrimitiveError> {
        let mut array = Array::new(methods.len())?;
        for (index, method) in methods.iter().enumerate() {
            let definition = build_method_definition(proxy, method, classes)?;
            let definition_any: AnyObjectRef = definition.into();
            array.insert(index, definition_any);
        }
        Ok(array)
    }

    fn build_method_definition(
        proxy: &vm_bindings::InterpreterProxy,
        definition: &tonel::MethodDefinition,
        classes: &PharoClasses,
    ) -> Result<TonelMethodDefinitionRef, TonelPrimitiveError> {
        let mut method_definition =
            Smalltalk::instantiate::<TonelMethodDefinitionRef>(classes.method_definition)?;

        method_definition.set_class_name(byte_string(proxy, definition.class_name.as_str())?);
        method_definition.set_method_type(byte_string(
            proxy,
            match definition.method_type {
                MethodType::Instance => "instance",
                MethodType::Class => "class",
            },
        )?);
        method_definition.set_selector(byte_string(proxy, definition.selector.as_str())?);
        method_definition.set_header(byte_string(proxy, definition.header.as_str())?);
        method_definition.set_body(byte_string(proxy, definition.body.as_str())?);
        method_definition.set_source(byte_string(proxy, definition.source.as_str())?);

        if let Some(category) = definition.category.as_deref() {
            method_definition.set_category(byte_string(proxy, category)?);
        }

        if let Some(metadata) = definition.metadata.as_ref() {
            let metadata_object = build_method_metadata(proxy, metadata)?;
            method_definition.set_raw_metadata(metadata_object);
        }

        Ok(method_definition)
    }

    fn build_method_metadata(
        proxy: &vm_bindings::InterpreterProxy,
        metadata: &tonel::MethodMetadata,
    ) -> Result<ByteStringRef, TonelPrimitiveError> {
        let metadata_string = ston_value_to_string(&metadata.raw);
        byte_string(proxy, metadata_string.as_str())
    }

    fn method_owner_kind_name(kind: MethodOwnerKind) -> &'static str {
        match kind {
            MethodOwnerKind::Trait => "trait",
            MethodOwnerKind::Class => "class",
            MethodOwnerKind::Extension => "extension",
        }
    }
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

fn class_from_array(array: &ArrayRef, index: usize) -> Result<ObjectRef, TonelPrimitiveError> {
    let value = array
        .get(index)
        .ok_or(TonelPrimitiveError::MissingClass(index))?;
    Ok(value.as_object()?)
}

fn ston_value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "nil".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Integer(number) => number.to_string(),
        Value::Float(number) => {
            let mut string = number.to_string();
            if !string.contains('.') && !string.contains('e') && !string.contains('E') {
                string.push_str(".0");
            }
            string
        }
        Value::String(text) => format!("'{}'", text.replace('\'', "''")),
        Value::Symbol(text) => {
            if text
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                format!("#{}", text)
            } else {
                format!("#'{}'", text.replace('\'', "''"))
            }
        }
        Value::Array(items) => {
            let inner = items
                .iter()
                .map(ston_value_to_string)
                .collect::<Vec<_>>()
                .join(" ");
            format!("[{}]", inner)
        }
        Value::Map(map) => {
            let inner = map
                .iter()
                .map(|(key, value)| {
                    format!(
                        "'{}': {}",
                        key.replace('\'', "''"),
                        ston_value_to_string(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(". ");
            format!("{{{}}}", inner)
        }
    }
}
