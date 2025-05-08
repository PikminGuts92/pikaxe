use crate::scene::ObjectTokens;
use proc_macro::TokenStream;
use quote::quote;

pub fn get_poll_tokens() -> ObjectTokens {
    let struct_fields = [
        quote! { pub target_1: String }.into(),
        quote! { pub target_2: String }.into(),
    ];

    let trait_impl = quote! {
        fn get_target_1(&self) -> &String {
            &self.target_1
        }
        fn get_target_1_mut(&mut self) -> &mut String {
            &mut self.target_1
        }
        fn set_target_1(&mut self, target: String) {
            self.target_1 = target;
        }

        fn get_target_2(&self) -> &String {
            &self.target_2
        }
        fn get_target_2_mut(&mut self) -> &mut String {
            &mut self.target_2
        }
        fn set_target_2(&mut self, target: String) {
            self.target_2 = target;
        }
    };

    ObjectTokens::from_tokens(
        Box::new(struct_fields),
        trait_impl
    )
}