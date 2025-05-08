use crate::scene::ObjectTokens;
use proc_macro::TokenStream;
use quote::quote;

pub fn get_milo_object_tokens() -> ObjectTokens {
    let struct_fields = [
        quote! { pub name: String }.into(),
        quote! { pub type2: String }.into(),
        quote! { pub note: String }.into(),
    ];

    let trait_impl = quote! {
        fn get_name(&self) -> &String {
            &self.name
        }
        fn get_name_mut(&mut self) -> &mut String {
            &mut self.name
        }
        fn set_name(&mut self, name: String) {
            self.name = name;
        }

        fn get_type(&self) -> &String {
            &self.type2
        }
        fn get_type_mut(&mut self) -> &mut String {
            &mut self.type2
        }
        fn set_type(&mut self, note: String) {
            self.note = note;
        }

        fn get_note(&self) -> &String {
            &self.note
        }
        fn get_note_mut(&mut self) -> &mut String {
            &mut self.note
        }
        fn set_note(&mut self, note: String) {
            self.note = note;
        }
    };

    ObjectTokens::from_tokens(
        Box::new(struct_fields),
        trait_impl
    )
}
