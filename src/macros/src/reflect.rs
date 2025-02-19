use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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
            let fields: Vec<_> = data_struct.fields.iter().enumerate().map(|(i, f)| 
                f.ident.as_ref()
                    .map(|ident| quote! { #ident })
                    .unwrap_or_else(|| {
                        let i = syn::Index::from(i);
                        quote! { #i }
                    })
            ).collect();

            let field_names: Vec<_> = fields.iter().map(|f| f.to_string()).collect();
            let field_types: Vec<_> = fields.iter().map(|f| quote! { self.#f.type_info() }).collect();
            let field_indices = 0..fields.len();
            let field_indices_2 = 0..fields.len();

            let reflect = quote! {
                fn field_by_index(&self, index: usize) -> Option<&dyn #path::reflect::Reflect> {
                    match index {
                        #(#field_indices => Some(&self.#fields),)*
                        _ => None,
                    }
                }
                
                fn set_field_by_index(&mut self, index: usize, value: Box<dyn std::any::Any>) -> Result<(), Box<dyn std::any::Any>> {
                    match index {
                        #(#field_indices_2 => {
                            match value.downcast::<_>() {
                                Ok(value) => {
                                    self.#fields = *value;
                                    Ok(())
                                },
                                Err(value) => Err(value),
                            }
                        },)*
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
            let variant_names: Vec<_> = data_enum.variants.iter().map(|v| v.ident.to_string()).collect();

            let reflect = quote! {
                fn field_by_index(&self, _index: usize) -> Option<&dyn #path::reflect::Reflect> {
                    None // Reflection for enum variants requires a different approach
                }
                
                fn set_field_by_index(&mut self, _index: usize, _value: Box<dyn std::any::Any>) -> Result<(), Box<dyn std::any::Any>> {
                    Err(_value) // Not applicable for enums
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
            return syn::Error::new_spanned(name, "Reflect cannot be derived for unions").to_compile_error().into();
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
