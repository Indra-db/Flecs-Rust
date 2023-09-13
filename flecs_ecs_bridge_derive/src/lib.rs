extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_cached_component_data(&ast)
}

fn impl_cached_component_data(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl CachedComponentData for #name {
            fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
                static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
                &ONCE_LOCK
            }
            fn get_symbol_name() -> &'static str {
                use std::any::type_name;
                static SYMBOL_NAME: OnceLock<String> = OnceLock::new();
                SYMBOL_NAME.get_or_init(|| type_name::<Self>().replace("::", "."))
            }
        }
    };
    gen.into()
}
