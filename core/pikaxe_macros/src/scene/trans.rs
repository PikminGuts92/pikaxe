use crate::scene::ObjectTokens;
use proc_macro::TokenStream;
use quote::quote;

pub fn get_trans_tokens() -> ObjectTokens {
    let struct_fields = [
        quote! { pub local_xfm: pikaxe_traits::scene::Matrix }.into(),
        quote! { pub world_xfm: pikaxe_traits::scene::Matrix }.into(),
        quote! { pub trans_objects: Vec<String> }.into(),
        quote! { pub constraint: pikaxe_traits::scene::TransConstraint }.into(),
        quote! { pub target: String }.into(),
        quote! { pub preserve_scale: bool }.into(),
        quote! { pub parent: String }.into(),
    ];

    let trait_impl = quote! {
        fn get_local_xfm(&self) -> &pikaxe_traits::scene::Matrix {
            &self.local_xfm
        }
        fn get_local_xfm_mut(&mut self) -> &mut pikaxe_traits::scene::Matrix {
            &mut self.local_xfm
        }
        fn set_local_xfm(&mut self, xfm: pikaxe_traits::scene::Matrix) {
            self.local_xfm = xfm;
        }

        fn get_world_xfm(&self) -> &pikaxe_traits::scene::Matrix {
            &self.world_xfm
        }
        fn get_world_xfm_mut(&mut self) -> &mut pikaxe_traits::scene::Matrix {
            &mut self.world_xfm
        }
        fn set_world_xfm(&mut self, xfm: pikaxe_traits::scene::Matrix) {
            self.world_xfm = xfm;
        }

        fn get_trans_objects(&self) -> &Vec<String> {
            &self.trans_objects
        }
        fn get_trans_objects_mut(&mut self) -> &mut Vec<String> {
            &mut self.trans_objects
        }
        fn set_trans_objects(&mut self, trans_objects: Vec<String>) {
            self.trans_objects = trans_objects;
        }

        fn get_constraint(&self) -> &pikaxe_traits::scene::TransConstraint {
            &self.constraint
        }
        fn get_constraint_mut(&mut self) -> &mut pikaxe_traits::scene::TransConstraint {
            &mut self.constraint
        }
        fn set_constraint(&mut self, constraint: pikaxe_traits::scene::TransConstraint) {
            self.constraint = constraint;
        }

        fn get_target(&self) -> &String {
            &self.target
        }
        fn get_target_mut(&mut self) -> &mut String {
            &mut self.target
        }
        fn set_target(&mut self, target: String) {
            self.target = target;
        }

        fn get_preserve_scale(&self) -> bool {
            self.preserve_scale
        }
        fn set_preserve_scale(&mut self, preserve_scale: bool) {
            self.preserve_scale = preserve_scale;
        }

        fn get_parent(&self) -> &String {
            &self.parent
        }
        fn get_parent_mut(&mut self) -> &mut String {
            &mut self.parent
        }
        fn set_parent(&mut self, parent: String) {
            self.parent = parent;
        }
    };

    ObjectTokens::from_tokens(
        Box::new(struct_fields),
        trait_impl
    )
}