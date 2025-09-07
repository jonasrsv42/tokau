use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Name)]
pub fn derive_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(
                name,
                "Name can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };
    
    let count = variants.len() as u32;
    
    let value_arms = variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        let index = i as u32;
        quote! {
            #name::#variant_name => #index
        }
    });
    
    let try_from_arms = variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        let index = i as u32;
        quote! {
            #index => Ok(#name::#variant_name)
        }
    });
    
    let expanded = quote! {
        impl ::tokau::Token for #name {
            const COUNT: u32 = #count;
        }
        
        impl ::tokau::NameToken for #name {
            fn value(&self) -> u32 {
                match self {
                    #(#value_arms,)*
                }
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