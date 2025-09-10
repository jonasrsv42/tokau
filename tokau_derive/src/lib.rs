use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitInt, Type, parse_macro_input};

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

            fn value(&self) -> u32 {
                self.0
            }
        }

        impl #impl_generics TryFrom<u32> for #name #ty_generics #where_clause {
            type Error = ::tokau::TokauError;

            fn try_from(offset: u32) -> Result<Self, Self::Error> {
                if offset < #count {
                    Ok(#name(offset))
                } else {
                    Err(::tokau::TokauError::OutOfRange {
                        value: offset,
                        max: #count
                    })
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Space, attributes(dynamic))]
pub fn derive_space(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Must be an enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(name, "Space can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    // Collect token types and check for dynamic variant
    let mut token_types = Vec::new();
    let mut dynamic_field = None;

    for variant in variants {
        // Check if this is the dynamic variant
        let is_dynamic = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("dynamic"));

        if is_dynamic {
            if dynamic_field.is_some() {
                return syn::Error::new_spanned(
                    &variant.ident,
                    "Only one variant can be marked as #[dynamic]",
                )
                .to_compile_error()
                .into();
            }
            dynamic_field = Some(variant.ident.clone());
        } else {
            // Extract the token type from the variant
            match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    if let Some(field) = fields.unnamed.first() {
                        if let Type::Path(type_path) = &field.ty {
                            token_types.push(type_path.path.clone());
                        }
                    }
                }
                _ => {
                    return syn::Error::new_spanned(
                        &variant.ident,
                        "Space enum variants must have exactly one unnamed field",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    // Generate Position implementations
    let mut position_impls = Vec::new();
    let mut offset_expr = quote! { 0 };

    for token_type in &token_types {
        position_impls.push(quote! {
            impl Position<#token_type> for #name {
                const OFFSET: u32 = #offset_expr;
            }
        });

        // Update offset for next type
        offset_expr = quote! { #offset_expr + <#token_type as ::tokau::Token>::COUNT };
    }

    // Calculate RESERVED
    let reserved_expr = if token_types.is_empty() {
        quote! { 0 }
    } else {
        let counts: Vec<_> = token_types
            .iter()
            .map(|t| {
                quote! { <#t as ::tokau::Token>::COUNT }
            })
            .collect();
        quote! { #(#counts)+* }
    };

    // Generate decode method implementation - use try_as<T>() for simplicity and correctness
    // TODO: Optimize this to generate efficient jump-table with match statement and literal range bounds
    // Current approach uses multiple try_as<T>() calls which do redundant offset calculations.
    // Ideal approach would be: match id { 0..=4 => ..., 5..=8 => ..., ... }
    // Challenge: Rust requires literal constants in pattern ranges, not expressions like `OFFSET + COUNT`
    // Possible solutions:
    // - Use const evaluation tricks or const blocks to compute bounds at compile time
    // - Generate numeric literals by evaluating token counts during macro expansion
    // - Hybrid approach with both current fallback and optimized match version
    let mut decode_arms = Vec::new();

    // Add arms for each token type using is<T>() calls
    for (variant, token_type) in variants.iter().zip(&token_types) {
        let is_dynamic = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("dynamic"));
        if !is_dynamic {
            let variant_name = &variant.ident;
            decode_arms.push(quote! {
                if let Some(token) = <#name as ::tokau::TokenSpace>::try_as::<#token_type>(id) {
                    return Ok(#name::#variant_name(token));
                }
            });
        }
    }

    // Add dynamic variant if present
    if let Some(dynamic_variant) = &dynamic_field {
        decode_arms.push(quote! {
            if let Some(offset) = <#name as ::tokau::TokenSpace>::remainder(id) {
                return Ok(#name::#dynamic_variant(offset));
            }
        });
    }

    // Generate value() method implementation
    let mut value_arms = Vec::new();
    for (variant, _token_type) in variants.iter().zip(&token_types) {
        let variant_name = &variant.ident;
        value_arms.push(quote! {
            #name::#variant_name(token) => <#name as ::tokau::TokenSpace>::position_of(token)
        });
    }

    // Add dynamic variant value arm if present
    if let Some(dynamic_variant) = &dynamic_field {
        value_arms.push(quote! {
            #name::#dynamic_variant(offset) => Self::RESERVED + offset
        });
    }

    let expanded = quote! {
        #(#position_impls)*

        impl ::tokau::TokenSpace for #name {
            const RESERVED: u32 = #reserved_expr;

            fn value(self) -> u32 {
                match self {
                    #(#value_arms,)*
                }
            }
        }

        impl TryFrom<u32> for #name {
            type Error = ::tokau::TokauError;

            fn try_from(id: u32) -> Result<Self, Self::Error> {
                #(#decode_arms)*
                Err(::tokau::TokauError::OutOfRange {
                    value: id,
                    max: Self::RESERVED
                })
            }
        }
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

            fn value(&self) -> u32 {
                *self as u32
            }
        }

        impl TryFrom<u32> for #name {
            type Error = ::tokau::TokauError;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    #(#try_from_arms,)*
                    _ => Err(::tokau::TokauError::OutOfRange {
                        value,
                        max: Self::COUNT
                    }),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
