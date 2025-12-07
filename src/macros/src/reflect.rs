use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use crate::resolve_path_name;

pub fn derive_reflect_implementation(item: TokenStream) -> TokenStream {
    let path = resolve_path_name();
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (reflect_impl_block, get_type_info_impl_block) = match &input.data {
        Data::Struct(data_struct) => {
            let is_tuple = matches!(data_struct.fields, Fields::Unnamed(_));
            let fields: Vec<_> = data_struct
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    f.ident
                        .as_ref()
                        .map(|ident| quote! { #ident })
                        .unwrap_or_else(|| {
                            let i = syn::Index::from(i);
                            quote! { #i }
                        })
                })
                .collect();

            let field_names: Vec<_> = fields.iter().map(|f| f.to_string()).collect();
            let field_types: Vec<_> = fields
                .iter()
                .map(|f| quote! { self.#f.type_info() })
                .collect();
            let field_indices: Vec<_> = (0..fields.len()).collect();

            let reflect = quote! {
                fn field_by_index(&self, index: usize) -> Option<&dyn #path::reflect::Reflect> {
                    match index {
                        #(#field_indices => Some(&self.#fields),)*
                        _ => None,
                    }
                }

                fn set_field_by_index(&mut self, index: usize, value: Box<dyn std::any::Any>) -> Result<(), Box<dyn std::any::Any>> {
                    match index {
                        #(#field_indices => value.downcast::<_>().map(|value| self.#fields = *value),)*
                        _ => Err(value),
                    }
                }
            };

            let get_type_info = quote! {
                fn type_info(&self) -> #path::reflect::type_info::TypeInfo {
                    use #path::reflect::type_info::{TypeInfo, StructInfo, TypePathInfo};

                    TypeInfo::Struct(StructInfo::new(
                        TypePathInfo::new(stringify!(#name), std::any::type_name::<Self>()),
                        [#(#field_names),*],
                        [#(#field_types),*],
                        #is_tuple
                    ))
                }

                fn type_name(&self) -> &'static str {
                    stringify!(#name)
                }
            };

            (reflect, get_type_info)
        }
        Data::Enum(data_enum) => {
            let variant_names: Vec<_> = data_enum
                .variants
                .iter()
                .map(|v| v.ident.to_string())
                .collect();

            let variant_matches = data_enum.variants.iter().map(|v| {
                let variant_name = &v.ident;
                match &v.fields {
                    Fields::Named(named) => {
                        let indices = 0..named.named.len();
                        let field_names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
                        quote! {
                            Self::#variant_name { #( #field_names),* } => match index {
                                #( #indices => Some(#field_names), )*
                                _ => None,
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let indices = 0..unnamed.unnamed.len();
                        let field_idents: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| Ident::new(&format!("field_{}", i), Span::call_site()))
                            .collect();
                        quote! {
                            Self::#variant_name( #( #field_idents ),* ) => match index {
                                #( #indices => Some(#field_idents), )*
                                _ => None,
                            }
                        }
                    }
                    Fields::Unit => quote! { Self::#variant_name => None },
                }
            });

            let set_variant_matches = data_enum.variants.iter().map(|v| {
                let variant_name = &v.ident;
                match &v.fields {
                    Fields::Named(named) => {
                        let indices = 0..named.named.len();
                        let field_names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
                        let field_types = named.named.iter().map(|f| &f.ty);
                        quote! {
                            Self::#variant_name { #( #field_names),* } => match index {
                                #( #indices => value.downcast::<#field_types>().map(|value| *#field_names = *value), )*
                                _ => Err(value),
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let indices = 0..unnamed.unnamed.len();
                        let field_idents: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| Ident::new(&format!("field_{}", i), Span::call_site()))
                            .collect();
                        let field_types = unnamed.unnamed.iter().map(|f| &f.ty);
                        quote! {
                            Self::#variant_name( #( #field_idents ),* ) => match index {
                                #( #indices => value.downcast::<#field_types>().map(|value| *#field_idents = *value), )*
                                _ => Err(value),
                            }
                        }
                    }
                    Fields::Unit => quote! { Self::#variant_name => Err(value) },
                }
            });

            let reflect = quote! {
                fn field_by_index(&self, index: usize) -> Option<&dyn #path::reflect::Reflect> {
                    match self {
                        #(#variant_matches,)*
                        _ => None,
                    }
                }

                fn set_field_by_index(&mut self, index: usize, value: Box<dyn std::any::Any>) -> Result<(), Box<dyn std::any::Any>> {
                    match self {
                        #(#set_variant_matches,)*
                        _ => Err(value),
                    }
                }
            };

            let get_type_info = quote! {
                fn type_info(&self) -> #path::reflect::type_info::TypeInfo {
                    use #path::reflect::type_info::{TypeInfo, EnumInfo, TypePathInfo};

                    TypeInfo::Enum(EnumInfo::new(
                        TypePathInfo::new(stringify!(#name), std::any::type_name::<Self>()),
                        [#(#variant_names),*],
                    ))
                }

                fn type_name(&self) -> &'static str {
                    stringify!(#name)
                }
            };

            (reflect, get_type_info)
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "Reflect cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics #path::reflect::Reflect for #name #ty_generics #where_clause {
            #reflect_impl_block
        }

        impl #impl_generics #path::reflect::type_info::GetTypeInfo for #name #ty_generics #where_clause {
            #get_type_info_impl_block
        }
    };

    TokenStream::from(expanded)
}
