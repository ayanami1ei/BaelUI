use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{self, Fields, Ident, ItemStruct};

/// Attribute macro form: `#[widget]` applied to a `struct` will ensure the
/// following fields exist (inject if missing):
/// - `pub window_id: winit::window::WindowId`
/// - `pub pattern: Vec<Vertex>`
/// - `pub texture: Vec<u8>`
///
/// It will also generate a `Widget` impl for the type. The generated `new`
/// initializes `pattern` and `texture` with empty `Vec`s.
//#[proc_macro_attribute]
pub fn impl_widget(_args: TokenStream, input: TokenStream) -> TokenStream {
    // parse the input as an item (expect a struct)
    let item = syn::parse::<ItemStruct>(input.clone());
    if item.is_err() {
        return quote! { compile_error!("#[widget] can only be applied to structs"); }.into();
    }
    let mut s = item.unwrap();

    // find field idents for pattern/texture
    let mut pattern_field: Option<Ident> = None;
    let mut texture_field: Option<Ident> = None;
    let mut has_window_id = false;

    if let Fields::Named(ref mut fields_named) = s.fields {
        for f in fields_named.named.iter() {
            if let Some(ident) = &f.ident {
                let name = ident.to_string();
                if name == "pattern" {
                    pattern_field = Some(ident.clone());
                }
                if name == "texture" {
                    texture_field = Some(ident.clone());
                }
                if name == "window_id" {
                    has_window_id = true;
                }
            }
            for attr in &f.attrs {
                if attr.path().is_ident("pattern") {
                    if let Some(ident) = &f.ident {
                        pattern_field = Some(ident.clone());
                    }
                }
                if attr.path().is_ident("texture") {
                    if let Some(ident) = &f.ident {
                        texture_field = Some(ident.clone());
                    }
                }
            }
        }

        // inject window_id field if missing
        if !has_window_id {
            let new_field: syn::Field =
                syn::parse_quote! { pub window_id: winit::window::WindowId };
            fields_named.named.push(new_field);
        }

        // inject pattern field if missing
        if pattern_field.is_none() {
            let new_field: syn::Field = syn::parse_quote! { pub pattern: Vec<Vertex> };
            fields_named.named.push(new_field);
            pattern_field = Some(Ident::new("pattern", Span::call_site()));
        }

        // inject texture field if missing
        if texture_field.is_none() {
            let new_field: syn::Field = syn::parse_quote! { pub texture: Vec<u8> };
            fields_named.named.push(new_field);
            texture_field = Some(Ident::new("texture", Span::call_site()));
        }
    } else {
        return quote! { compile_error!("#[widget] only supports structs with named fields"); }
            .into();
    }

    let name = &s.ident;
    let pattern_ident = pattern_field.unwrap();
    let texture_ident = texture_field.unwrap();

    // emit the (possibly modified) struct plus a Widget impl
    let expanded = quote! {
        #s

        impl Widget for #name {
            fn new(window_id: winit::window::WindowId, pattern:&Vec<Vertex>, texture:&Vec<u8>) -> Self {
                Self {
                    #pattern_ident: pattern.clone(),
                    #texture_ident: texture.clone(),
                    window_id,
                }
            }

            fn set_pattern(&mut self, pattern: Vec<Vertex>) {
                self.#pattern_ident = pattern;
            }

            fn set_texture(&mut self, texture: Vec<u8>) {
                self.#texture_ident = texture;
            }

            fn get_pattern(&self) -> &Vec<Vertex> {
                &self.#pattern_ident
            }

            fn get_texture(&self) -> &Vec<u8> {
                &self.#texture_ident
            }

            fn get_window(&self) -> winit::window::WindowId {
                self.window_id
            }
        }
    };
    expanded.into()
}
