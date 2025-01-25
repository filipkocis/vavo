use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use proc_macro::{TokenStream};
use proc_macro2::{Span};
use proc_macro_crate::{crate_name, FoundCrate};

fn resolve_path_name() -> proc_macro2::TokenStream {
    match crate_name("vavo") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(crate_name)) => {
            let ident = Ident::new(&crate_name, Span::call_site());
            quote!(#ident)
        }
        Err(_) => {
            return syn::Error::new(
                Span::call_site(),
                "Could not find the `vavo` crate. Ensure it is a dependency in your Cargo.toml.",
            )
            .to_compile_error()
            .into();
        }
    }
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(item: proc_macro::TokenStream) -> TokenStream {
    let path = resolve_path_name();
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let expanded = quote! {
        impl #impl_generics #path::ecs::resources::Resource for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Asset)]
pub fn derive_asset(item: proc_macro::TokenStream) -> TokenStream {
    let path = resolve_path_name();
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let expanded = quote! {
        impl #impl_generics #path::assets::Asset for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RenderAsset)]
pub fn derive_render_asset(item: proc_macro::TokenStream) -> TokenStream {
    let path = resolve_path_name();
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let expanded = quote! {
        impl #impl_generics #path::render_assets::RenderAsset for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(States)]
pub fn derive_states(item: proc_macro::TokenStream) -> TokenStream {
    let path = resolve_path_name();
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let expanded = quote! {
        impl #impl_generics #path::ecs::state::States for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}
