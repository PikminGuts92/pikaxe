use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Meta, Path, parse::Parser, punctuated::Punctuated, Token};

/*
TODO: Finish migrating from v1

pub fn get_meta_list(args: &Vec<NestedMeta>) -> Vec<String> {
    let mut metas = Vec::new();

    for arg in args {
        if let NestedMeta::Meta(meta) = arg {
            if let Meta::Path(path) = meta {
                let segment = path.segments.last().unwrap();
                metas.push(segment.ident.to_string());
            } else {
                panic!()
            }
        }
    }

    metas
} */

pub fn get_meta_paths(args: &Vec<Attribute>) -> Vec<Path> {
    let mut paths = Vec::new();

    for arg in args {
        //arg.parse_nested_meta(logic)
        let nested_meta = arg.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);

        let Ok(nested_meta) = nested_meta else {
            continue;
        };

        //let nested = arg.parse_nested_meta(|_| Ok(())).unwrap(); // Accept all meta attributes

        for meta in nested_meta {
            paths.push(meta.path().clone());
        }
    }

    paths
}

pub fn insert_as_struct_fields<T>(input: &mut DeriveInput, struct_fields: T) where T : AsRef<[TokenStream]> {
    if let syn::Data::Struct(s) = &mut input.data {
        if let syn::Fields::Named(fields) = &mut s.fields {

            // Insert fields into struct
            for field in struct_fields.as_ref() {
                fields.named.push(syn::Field::parse_named.parse(field.to_owned().into()).unwrap())
            }
        }
    }
}

pub fn with_trait_implementation(input: &DeriveInput, trait_path: &Path, trait_impl: proc_macro2::TokenStream) -> TokenStream {
    let ident = input.ident.to_owned();

    (quote! {
        #input

        impl #trait_path for #ident {
            #trait_impl
        }
    }).into()
}

pub fn extend_token_stream_with_trait_implementation(
    input: proc_macro2::TokenStream,
    ident: &proc_macro2::Ident,
    trait_path: &Path,
    trait_impl: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        #input

        impl #trait_path for #ident {
            #trait_impl
        }
    }
}