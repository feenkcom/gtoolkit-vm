use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, LitInt, LitStr, Result, Type, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_derive(PharoObject, attributes(pharo_object, pharo_field))]
pub fn derive_pharo_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_pharo_object(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_pharo_object(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "PharoObject derive does not support generic parameters",
        ));
    }

    let struct_ident = &input.ident;
    let ref_ident = format_ident!("{struct_ident}Ref");
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new(
                    data_struct.fields.span(),
                    "PharoObject derive only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "PharoObject derive only supports structs",
            ));
        }
    };

    if fields.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "PharoObject derive requires at least one field",
        ));
    }

    let expected_slots = parse_expected_slots(&input.attrs)?
        .unwrap_or_else(|| default_expected_slots(fields.iter()));

    let expected_slots_lit = LitInt::new(&expected_slots.to_string(), Span::call_site());
    let setter_impl = generate_setters(struct_ident, fields)?;

    let tokens = quote! {
        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        #vis struct #ref_ident(::vm_object_model::ObjectRef);

        impl #ref_ident {
            pub unsafe fn from_any_object_unchecked(value: ::vm_object_model::AnyObjectRef) -> Self {
                Self(::vm_object_model::ObjectRef::from_raw_pointer_unchecked(value.into_inner()))
            }
        }

        impl ::core::ops::Deref for #struct_ident {
            type Target = ::vm_object_model::Object;

            fn deref(&self) -> &Self::Target {
                &self.this
            }
        }

        impl ::core::ops::DerefMut for #struct_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.this
            }
        }

        impl ::core::ops::Deref for #ref_ident {
            type Target = #struct_ident;

            fn deref(&self) -> &Self::Target {
                unsafe { self.0.cast() }
            }
        }

        impl ::core::ops::DerefMut for #ref_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.0.cast_mut() }
            }
        }

        impl ::core::convert::TryFrom<::vm_object_model::AnyObjectRef> for #ref_ident {
            type Error = ::vm_object_model::Error;

            fn try_from(value: ::vm_object_model::AnyObjectRef) -> ::std::result::Result<Self, Self::Error> {
                const EXPECTED_AMOUNT_OF_SLOTS: usize = #expected_slots_lit;

                let object = value.as_object()?;
                let actual_amount_of_slots = object.amount_of_slots();

                if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
                    Err(vm_object_model::Error::WrongAmountOfSlots {
                        object: object.header().clone(),
                        type_name: ::core::any::type_name::<Self>().to_string(),
                        expected: EXPECTED_AMOUNT_OF_SLOTS,
                        actual: actual_amount_of_slots,
                    })?;
                }

                Ok(Self(object))
            }
        }

        impl ::core::convert::From<#ref_ident> for ::vm_object_model::AnyObjectRef {
            fn from(value: #ref_ident) -> Self {
                value.0.into()
            }
        }

        #setter_impl
    };

    Ok(tokens)
}

fn parse_expected_slots(attrs: &[Attribute]) -> Result<Option<usize>> {
    let mut expected_slots = None;
    for attr in attrs {
        if !attr.path().is_ident("pharo_object") {
            continue;
        }

        let args = attr.parse_args::<PharoObjectArgs>()?;
        if let Some(value) = args.expected_slots {
            expected_slots = Some(value);
        }
    }
    Ok(expected_slots)
}

fn default_expected_slots<'a, I>(fields: I) -> usize
where
    I: Iterator<Item = &'a syn::Field>,
{
    fields
        .filter(|field| {
            field
                .ident
                .as_ref()
                .map(|ident| ident != "this")
                .unwrap_or(true)
        })
        .count()
}

struct PharoObjectArgs {
    expected_slots: Option<usize>,
}

impl Parse for PharoObjectArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                expected_slots: None,
            });
        }

        let ident: Ident = input.parse()?;
        if ident != "expected_slots" {
            return Err(syn::Error::new(
                ident.span(),
                "unsupported attribute key; expected `expected_slots`",
            ));
        }

        input.parse::<syn::Token![=]>()?;
        let value: LitInt = input.parse()?;
        let expected_slots = value.base10_parse::<usize>()?;

        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if !input.is_empty() {
                return Err(syn::Error::new(
                    input.span(),
                    "unexpected tokens after `expected_slots` specification",
                ));
            }
        }

        Ok(Self {
            expected_slots: Some(expected_slots),
        })
    }
}

#[derive(Default)]
struct SetterOptions {
    skip: bool,
    visibility: Option<Visibility>,
    name: Option<Ident>,
}

fn generate_setters(
    struct_ident: &Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<proc_macro2::TokenStream> {
    let mut methods = Vec::new();

    for field in fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };

        if field_ident == "this" {
            continue;
        }

        let mut options = parse_setter_options(field)?;
        if options.skip {
            continue;
        }

        let setter_ident = options
            .name
            .take()
            .unwrap_or_else(|| format_ident!("set_{}", field_ident));

        let setter_visibility: Visibility = options
            .visibility
            .take()
            .unwrap_or_else(|| syn::parse_quote!(pub));

        let field_ty = &field.ty;

        let body = if is_immediate_type(field_ty) {
            quote! {
                #setter_visibility fn #setter_ident(&mut self, value: impl Into<::vm_object_model::Immediate>) {
                    self.#field_ident = value.into();
                }
            }
        } else {
            quote! {
                #setter_visibility fn #setter_ident(&mut self, value: impl Into<#field_ty>) {
                    let value = value.into();
                    ::vm_object_model::assign_field!(self, self.#field_ident, value);
                }
            }
        };

        methods.push(body);
    }

    if methods.is_empty() {
        Ok(quote! {})
    } else {
        Ok(quote! {
            impl #struct_ident {
                #(#methods)*
            }
        })
    }
}

fn parse_setter_options(field: &syn::Field) -> Result<SetterOptions> {
    let mut options = SetterOptions::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("pharo_field") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip_setter") {
                options.skip = true;
                return Ok(());
            }

            if meta.path.is_ident("setter") {
                if meta.input.peek(syn::token::Paren) {
                    meta.parse_nested_meta(|setter_meta| {
                        if setter_meta.path.is_ident("skip") {
                            options.skip = true;
                            Ok(())
                        } else if setter_meta.path.is_ident("private") {
                            options.visibility = Some(Visibility::Inherited);
                            Ok(())
                        } else if setter_meta.path.is_ident("name") {
                            let lit: LitStr = setter_meta.value()?.parse()?;
                            set_setter_name(&mut options, lit)
                        } else if setter_meta.path.is_ident("visibility") {
                            let lit: LitStr = setter_meta.value()?.parse()?;
                            let visibility = parse_visibility(&lit)?;
                            options.visibility = Some(visibility);
                            Ok(())
                        } else {
                            Err(setter_meta.error(
                                "unsupported setter option; expected `skip`, `private`, `name`, or `visibility`",
                            ))
                        }
                    })?;
                    return Ok(());
                }

                let lit: LitStr = meta.value()?.parse()?;
                set_setter_name(&mut options, lit)?;
                return Ok(());
            }

            Err(meta.error(
                "unsupported field attribute; expected `skip_setter` or `setter(...)`",
            ))
        })?;
    }

    Ok(options)
}

fn set_setter_name(options: &mut SetterOptions, lit: LitStr) -> Result<()> {
    if options.name.is_some() {
        return Err(syn::Error::new(lit.span(), "setter name already specified"));
    }

    let ident = Ident::new(&lit.value(), lit.span());
    options.name = Some(ident);
    Ok(())
}

fn parse_visibility(lit: &LitStr) -> Result<Visibility> {
    if lit.value() == "private" {
        return Ok(Visibility::Inherited);
    }

    syn::parse_str::<Visibility>(&lit.value()).map_err(|_| {
        syn::Error::new(
            lit.span(),
            "invalid visibility; expected values like `pub`, `pub(crate)` or `private`",
        )
    })
}

fn is_immediate_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "Immediate")
            .unwrap_or(false),
        _ => false,
    }
}
