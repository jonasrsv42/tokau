use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitInt, parse_macro_input};

// Attribute macro for cleaner syntax: #[range(1000)]
#[proc_macro_attribute]
pub fn range(args: TokenStream, input: TokenStream) -> TokenStream {
    let count = parse_macro_input!(args as LitInt);
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    // Verify it's a tuple struct with single field
    let is_valid = match &input.data {
        Data::Struct(data_struct) => {
            matches!(&data_struct.fields, Fields::Unnamed(fields) if fields.unnamed.len() == 1)
        }
        _ => false,
    };

    if !is_valid {
        return syn::Error::new_spanned(
            name,
            "range can only be applied to tuple structs with a single field: struct MyTokens(u32);",
        )
        .to_compile_error()
        .into();
    }

    // Get the fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => unreachable!(), // We already validated it's a struct
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #(#attrs)*
        #vis struct #name #generics #fields;

        impl #impl_generics ::tokau::Token for #name #ty_generics #where_clause {
            const COUNT: u32 = #count;
        }

        impl #impl_generics ::tokau::RangeToken for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Name)]
pub fn derive_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(name, "Name can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    let count = variants.len() as u32;

    let try_from_arms = variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        let index = i as u32;
        quote! {
            #index => Ok(#name::#variant_name)
        }
    });

    // Add #[repr(u32)] attribute to the enum
    let expanded = quote! {
        impl ::tokau::Token for #name {
            const COUNT: u32 = #count;
        }

        impl ::tokau::NameToken for #name {
            fn value(&self) -> u32 {
                *self as u32
            }
        }

        impl TryFrom<u32> for #name {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    #(#try_from_arms,)*
                    _ => Err(()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
