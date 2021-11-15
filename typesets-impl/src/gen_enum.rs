use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Variant;

pub fn variant_to_arm_partial(variant: &Variant) -> TokenStream {
    let name = variant.ident.clone();
    match variant.fields.clone() {
        syn::Fields::Named(fields) => {
            let field_idents: Vec<Ident> = fields
                .named
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .collect();
            quote! { #name { #(#field_idents),* } }
        }
        syn::Fields::Unnamed(fields) => {
            let field_idents: Vec<Ident> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| Ident::new(format!("v{}", idx).as_str(), Span::call_site()))
                .collect();
            quote! { #name( #(#field_idents),* ) }
        }
        syn::Fields::Unit => quote! { #name },
    }
}

pub fn gen_subtype_enum(subtype: Ident, supertype: Ident, variants: Vec<Variant>) -> TokenStream {
    let conversions = gen_enum_conversion(&subtype, &supertype, &variants);
    quote! {
        enum #subtype {
            #(#variants),*
        }
        #conversions
    }
}

pub fn gen_enum_conversion(
    subtype: &Ident,
    supertype: &Ident,
    variants: &Vec<Variant>,
) -> TokenStream {
    let arm_parts: Vec<TokenStream> = variants.iter().map(|v| variant_to_arm_partial(v)).collect();
    quote! {
        impl TryFrom<#supertype> for #subtype {
            type Error = crate::typesets::supertype::SupertypeError;
            fn try_from(supertype: #supertype) -> Result<Self, Self::Error> {
                match supertype {
                    #(#supertype::#arm_parts => Ok(#subtype::#arm_parts)),*,
                    other => Err(Self::Error::EnumNoOverlap {
                        supertype: stringify!(#supertype),
                        subtype: stringify!(#subtype),
                        variant: format!("{:?}", other)
                    })
                }
            }
        }

        impl From<#subtype> for #supertype {
            fn from(child: #subtype) -> Self {
                match child {
                    #(#supertype::#arm_parts => #subtype::#arm_parts),*
                }
            }
        }
    }
}
