extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

#[proc_macro_derive(Component)]
pub fn component_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    // Parse the input tokens into a syntax tree
    let input = syn::parse_macro_input!(input as DeriveInput);

    // Build the output
    let expanded: TokenStream = match &input.data {
        Data::Struct(_) => impl_cached_component_data_struct(&input),
        Data::Enum(_) => impl_cached_component_data_enum(&input),
        _ => quote! {
            compile_error!("The type is neither a struct nor an enum!");
        },
    };

    // Convert the generated code into a TokenStream and return it
    ProcMacroTokenStream::from(expanded)
}

fn impl_cached_component_data_struct(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl CachedComponentData<StructComponent> for #name {
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

fn impl_cached_component_data_enum(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl CachedComponentData<EnumComponent> for #name {
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
