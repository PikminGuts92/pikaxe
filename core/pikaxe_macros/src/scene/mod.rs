mod anim;
mod draw;
mod group;
mod mesh;
mod milo_object;
mod poll;
mod trans;

use crate::*;
use lazy_static::*;
pub use milo_object::*;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Path, parse_macro_input};

type GetObjectTokensFn = fn() -> ObjectTokens;

lazy_static! {
    static ref OBJECT_TOKENS: HashMap<&'static str, GetObjectTokensFn> = {
        let mut m: HashMap<&'static str, GetObjectTokensFn> = HashMap::new();
        m.insert("Anim", anim::get_anim_tokens);
        m.insert("Draw", draw::get_draw_tokens);
        m.insert("Group", group::get_group_tokens);
        m.insert("RndMesh", mesh::get_mesh_tokens);
        m.insert("Poll", poll::get_poll_tokens);
        m.insert("Trans", trans::get_trans_tokens);
        m
    };
}

pub fn get_object_tokens(obj_type: &str) -> Option<ObjectTokens> {
    match OBJECT_TOKENS.get(obj_type) {
        Some(impl_trait_fn) => Some(impl_trait_fn()),
        _ => None,
    }
}

pub struct ObjectTokens {
    pub struct_fields: Box<[TokenStream]>,
    pub trait_impl: proc_macro2::TokenStream,
}

impl ObjectTokens {
    pub fn from_tokens(struct_fields: Box<[TokenStream]>, trait_impl: proc_macro2::TokenStream) -> Self {
        ObjectTokens {
            struct_fields,
            trait_impl,
        }
    }

    pub fn apply(self, input: TokenStream, trait_path: &Path) -> TokenStream {
        // TODO: Fix for regular milo attribute
        let mut input = parse_macro_input!(input as DeriveInput);

        insert_as_struct_fields(&mut input, self.struct_fields);
        with_trait_implementation(&input, trait_path, self.trait_impl)
    }
}