use crate::scene::ObjectTokens;
use proc_macro::TokenStream;
use quote::quote;

pub fn get_draw_tokens() -> ObjectTokens {
    let struct_fields = [
        quote! { pub draw_objects: Vec<String> }.into(),
        quote! { pub showing: bool }.into(),
        quote! { pub sphere: pikaxe_traits::scene::Sphere }.into(),
        quote! { pub draw_order: f32 }.into(),
        quote! { pub override_include_in_depth_only_pass: pikaxe_traits::scene::OverrideIncludeInDepthOnlyPass }.into(),
    ];

    let trait_impl = quote! {
        fn get_showing(&self) -> bool {
            self.showing
        }

        fn set_showing(&mut self, showing: bool) {
            self.showing = showing;
        }

        fn get_draw_objects(&self) -> &Vec<String> {
            &self.draw_objects
        }
        fn get_draw_objects_mut(&mut self) -> &mut Vec<String> {
            &mut self.draw_objects
        }
        fn set_draw_objects(&mut self, draw_objects: Vec<String>) {
            self.draw_objects = draw_objects;
        }

        fn get_sphere(&self) -> &pikaxe_traits::scene::Sphere {
            &self.sphere
        }

        fn get_sphere_mut(&mut self) -> &mut pikaxe_traits::scene::Sphere {
            &mut self.sphere
        }

        fn set_sphere(&mut self, sphere: pikaxe_traits::scene::Sphere) {
            self.sphere = sphere;
        }

        fn get_draw_order(&self) -> f32 {
            self.draw_order
        }

        fn set_draw_order(&mut self, draw_order: f32) {
            self.draw_order = draw_order;
        }

        fn get_override_include_in_depth_only_pass(&self) -> &pikaxe_traits::scene::OverrideIncludeInDepthOnlyPass {
            &self.override_include_in_depth_only_pass
        }

        fn get_override_include_in_depth_only_pass_mut(&mut self) -> &mut pikaxe_traits::scene::OverrideIncludeInDepthOnlyPass {
            &mut self.override_include_in_depth_only_pass
        }

        fn set_override_include_in_depth_only_pass(&mut self, override_include: pikaxe_traits::scene::OverrideIncludeInDepthOnlyPass) {
            self.override_include_in_depth_only_pass = override_include;
        }
    };

    ObjectTokens::from_tokens(
        Box::new(struct_fields),
        trait_impl
    )
}