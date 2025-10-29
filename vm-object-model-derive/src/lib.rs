use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, LitInt, Result,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_derive(PharoObject, attributes(pharo_object))]
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
