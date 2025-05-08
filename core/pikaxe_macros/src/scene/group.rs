use crate::scene::ObjectTokens;
use proc_macro::TokenStream;
use quote::quote;

pub fn get_group_tokens() -> ObjectTokens {
    let struct_fields = [
        quote! { pub objects: Vec<String> }.into(),
        quote! { pub environ: String }.into(),
        quote! { pub draw_only: String }.into(),
        quote! { pub lod: String }.into(),
        quote! { pub lod_screen_size: f32 }.into(),
        quote! { pub sort_in_world: bool }.into(),
    ];

    let trait_impl = quote! {
        fn get_objects(&self) -> &Vec<String> {
            &self.objects
        }
        fn get_objects_mut(&mut self) -> &mut Vec<String> {
            &mut self.objects
        }
        fn set_objects(&mut self, objects: Vec<String>) {
            self.objects = objects;
        }

        fn get_environ(&self) -> &String {
            &self.environ
        }
        fn get_environ_mut(&mut self) -> &mut String {
            &mut self.environ
        }
        fn set_environ(&mut self, environ: String) {
            self.environ = environ;
        }

        fn get_draw_only(&self) -> &String {
            &self.draw_only
        }
        fn get_draw_only_mut(&mut self) -> &mut String {
            &mut self.draw_only
        }
        fn set_draw_only(&mut self, draw_only: String) {
            self.draw_only = draw_only;
        }

        fn get_lod(&self) -> &String {
            &self.lod
        }
        fn get_lod_mut(&mut self) -> &mut String {
            &mut self.lod
        }
        fn set_lod(&mut self, lod: String) {
            self.lod = lod;
        }

        fn get_lod_screen_size(&self) -> f32 {
            self.lod_screen_size
        }
        fn set_lod_screen_size(&mut self, lod_screen_size: f32) {
            self.lod_screen_size = lod_screen_size;
        }

        fn get_sort_in_world(&self) -> bool {
            self.sort_in_world
        }
        fn set_sort_in_world(&mut self, sort_in_world: bool) {
            self.sort_in_world = sort_in_world;
        }
    };

    ObjectTokens::from_tokens(
        Box::new(struct_fields),
        trait_impl
    )
}