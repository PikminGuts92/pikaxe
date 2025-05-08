use crate::scene::get_object_tokens;
use crate::scene::get_milo_object_tokens;
use proc_macro::TokenStream;
use syn::{DeriveInput, Meta, parse::Parser, parse_macro_input, punctuated::Punctuated, Token};
use quote::quote;

mod common;
mod scene;

use common::*;

// TODO: Implment custom macro for getting type
/*#[proc_macro_attribute]
pub fn milo_object(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    for arg in &args {
        match arg {
            NestedMeta::Lit(lit) => {

            },
            NestedMeta::Meta(meta) => {

            },
            _ => {

            }
        }
    }

    input
}*/

#[proc_macro_attribute]
pub fn version(_args: TokenStream, input: TokenStream) -> TokenStream {
    let struct_fields = quote! {
        pub version: Option<u32>
    };

    let mut input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(s) = &mut input.data {
        if let syn::Fields::Named(fields) = &mut s.fields {
            fields.named.push(syn::Field::parse_named.parse2(struct_fields).unwrap())
        }
    }

    //let DeriveInput { ident, data, .. }

    let ident = input.ident.to_owned();

    let output = quote! {
        #input

        impl #ident {
            fn get_version(&self) -> Option<u32> {
                self.version
            }

            fn set_version(&mut self, version: Option<u32>) {
                self.version = version
            }
        }
    };

    output.into()

    /*(quote! {
        #input
    }).into()*/
}

#[proc_macro_attribute]
pub fn milo(args: TokenStream, input: TokenStream) -> TokenStream {
    //let args = parse_macro_input!(args as AttributeArgs);

    //syn::Attribute::parse_outer();
    //let args = parse_macro_input!(args with syn::Attribute::parse_outer);

    //Punctuated<Meta, Token![,]>

    // TODO: Move to struct (Replaces get_meta_paths)
    // https://docs.rs/syn/latest/syn/meta/fn.parser.html#example
    // https://docs.rs/syn/latest/syn/meta/struct.ParseNestedMeta.html
    let mut paths = Vec::new();
    let meta_path_parser = syn::meta::parser(|m| {
        paths.push(m.path);
        Ok(())
    });

    //let args = syn::Attribute::parse_outer.parse(args);
    parse_macro_input!(args with meta_path_parser);
    /*let Ok(args) = args else {
        println!("Failed to parse args");
        return input;
    };*/


    //let paths = syn::Attribute::parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);

    //let paths = get_meta_paths(&args);

    let mut input = parse_macro_input!(input as DeriveInput);
    let mut transformed_input = proc_macro2::TokenStream::new();

    // Inherit base milo object trait
    let path: syn::Path = syn::parse_str("pikaxe_traits::scene::MiloObject").unwrap();

    let base_tokens = get_milo_object_tokens();
    insert_as_struct_fields(&mut input, base_tokens.struct_fields);
    transformed_input = extend_token_stream_with_trait_implementation(transformed_input, &input.ident, &path, base_tokens.trait_impl);

    if let Some(path) = paths.first() {
        let trait_name = path.segments.last().unwrap().ident.to_string();

        match get_object_tokens(&trait_name) {
            Some(tokens) => {
                insert_as_struct_fields(&mut input, tokens.struct_fields);
                transformed_input = extend_token_stream_with_trait_implementation(transformed_input, &input.ident, path, tokens.trait_impl);
            },
            _ => panic!("Unsupported trait!"),
        };
    }

    (quote! {
        #input
        #transformed_input
    }).into()
}

#[proc_macro_attribute]
pub fn milo_super(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut paths = Vec::new();
    let meta_path_parser = syn::meta::parser(|m| {
        paths.push(m.path);
        Ok(())
    });

    parse_macro_input!(args with meta_path_parser);

    let mut input = parse_macro_input!(input as DeriveInput);
    let mut transformed_input = proc_macro2::TokenStream::new();

    for path in &paths {
        let trait_name = path.segments.last().unwrap().ident.to_string();

        transformed_input = match get_object_tokens(&trait_name) {
            Some(tokens) => {
                insert_as_struct_fields(&mut input, tokens.struct_fields);
                extend_token_stream_with_trait_implementation(transformed_input, &input.ident, path, tokens.trait_impl)
            },
            _ => panic!("Unsupported \"{}\" trait!", &trait_name),
        };
    }

    (quote! {
        #input
        #transformed_input
    }).into()
}