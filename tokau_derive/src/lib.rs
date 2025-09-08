use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

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
