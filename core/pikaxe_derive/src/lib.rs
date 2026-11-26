use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    custom_keyword, Fields, Ident, ItemStruct, LitStr, Path, Token, Type,
};

/*
milo_component(name="Group", super=Object,Draw,Trans)
- Creates trait object

milo(name="Group", components=Object,Draw,Trans)
- Implements given traits
*/

struct ExtendsArgs {
    traits: Punctuated<Path, Token![+]>,
}

impl Parse for ExtendsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::custom_keyword!(extends);

        // If the macro input is completely empty, return empty traits list
        if input.is_empty() {
            return Ok(ExtendsArgs { traits: Punctuated::new() });
        }

        // Check if the user used the "extends =" syntax
        let _extends: extends = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;

        // Parse the remaining comma-separated list of traits
        let traits = Punctuated::parse_terminated(input)?;
        Ok(ExtendsArgs { traits })
    }
}

#[proc_macro_attribute]
pub fn autotrait(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ExtendsArgs);
    let input_struct = parse_macro_input!(item as ItemStruct);

    let struct_name = &input_struct.ident;
    let struct_name_str = struct_name.to_string();

    let trait_name = if struct_name_str.ends_with("Component") {
        format_ident!("{}", &struct_name_str[..struct_name_str.len() - 9])
    } else {
        format_ident!("{}Object", struct_name_str)
    };

    let trait_bounds = if !args.traits.is_empty() {
        let trait_list = &args.traits;
        quote! { : #trait_list }
    } else {
        quote! { }
    };

    let fields = match &input_struct.fields {
        Fields::Named(fields_named) => &fields_named.named,
        _ => panic!("#[autotrait] only supports structs with named fields."),
    };

    let component_get_fn = format_ident!("get_{}", snake_case(&struct_name_str));
    let component_get_mut_fn = format_ident!("get_{}_mut", snake_case(&struct_name_str));

    // Map over each field
    let methods = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let get_name = format_ident!("get_{}", field_name);
        let get_mut_name = format_ident!("get_{}_mut", field_name);
        let set_name = format_ident!("set_{}", field_name);

        // Check if the type is a primitive that implements Copy
        if is_copy_primitive(field_type) {
            // Primitive Copy-type logic: Return by value, skip `get_mut` method
            quote! {
                fn #get_name(&self) -> #field_type {
                    self.#component_get_fn().#field_name
                }

                fn #set_name(&mut self, value: #field_type) {
                    self.#component_get_mut_fn().#field_name = value;
                }
            }
        } else {
            // Reference type logic (e.g. String, custom structs): Keep original getters/setters
            quote! {
                fn #get_name(&self) -> &#field_type {
                    &self.#component_get_fn().#field_name
                }

                fn #get_mut_name(&mut self) -> &mut #field_type {
                    &mut self.#component_get_mut_fn().#field_name
                }

                fn #set_name(&mut self, value: #field_type) {
                    self.#component_get_mut_fn().#field_name = value;
                }
            }
        }
    });

    let expanded = quote! {
        #input_struct

        pub trait #trait_name #trait_bounds {
            fn #component_get_fn(&self) -> &#struct_name;
            fn #component_get_mut_fn(&mut self) -> &mut #struct_name;

            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn milo(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut class_name: Option<LitStr> = None;
    let args_parser = syn::meta::parser(|meta| {
        if !meta.path.is_ident("class_name") {
            return Err(meta.error("unsupported milo parameter, expected `class_name`"));
        }

        if class_name.is_some() {
            return Err(meta.error("duplicate `class_name` parameter"));
        }

        class_name = Some(meta.value()?.parse()?);
        Ok(())
    });

    parse_macro_input!(attr with args_parser);
    let input = parse_macro_input!(item as ItemStruct);

    let impls = expand_milo(&input, class_name)
        .unwrap_or_else(syn::Error::into_compile_error);

    // Struct is emitted even on error to avoid follow-up "type not found" errors
    quote! {
        #input
        #impls
    }
    .into()
}

fn expand_milo(input: &ItemStruct, class_name: Option<LitStr>) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let Fields::Named(fields_named) = &input.fields else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "milo only supports structs with named fields",
        ));
    };

    let class_name = class_name.unwrap_or_else(|| {
        let name = struct_name.to_string();
        let name = name.strip_suffix("Instance").unwrap_or(&name);
        LitStr::new(name, struct_name.span())
    });

    let mut has_object = false;
    let mut impls = Vec::new();

    for field in &fields_named.named {
        let Some(component) = component_ident(&field.ty) else {
            continue;
        };

        let component_name = component.to_string();
        let Some(trait_name) = component_name.strip_suffix("Component").filter(|n| !n.is_empty()) else {
            continue;
        };

        let trait_ident = format_ident!("{}", trait_name, span = component.span());
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let get_fn = format_ident!("get_{}", snake_case(&component_name));
        let get_mut_fn = format_ident!("get_{}_mut", snake_case(&component_name));

        let class_name_fn = (trait_name == "Object").then(|| {
            has_object = true;

            quote! {
                fn get_class_name(&self) -> &'static str {
                    #class_name
                }
            }
        });

        impls.push(quote! {
            impl #impl_generics #trait_ident for #struct_name #type_generics #where_clause {
                #class_name_fn

                fn #get_fn(&self) -> &#field_type {
                    &self.#field_name
                }

                fn #get_mut_fn(&mut self) -> &mut #field_type {
                    &mut self.#field_name
                }
            }
        });
    }

    if !has_object {
        return Err(syn::Error::new_spanned(
            struct_name,
            "milo requires a field of type `ObjectComponent`",
        ));
    }

    Ok(quote! { #(#impls)* })
}

/// Last path segment of a field type, e.g. `TransComponent` for `trans::TransComponent`
fn component_ident(ty: &Type) -> Option<&Ident> {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().map(|s| &s.ident),
        _ => None,
    }
}

fn is_copy_primitive(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(ident) = type_path.path.get_ident() {
            let name = ident.to_string();
            return matches!(
                name.as_str(),
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                "f32" | "f64" | "bool" | "char"
            );
        }
    }
    false
}

fn snake_case(s: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        } else {
            snake.push(ch);
        }
    }
    snake
}