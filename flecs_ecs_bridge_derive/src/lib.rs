extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// `Component` macro for defining ECS components.
///
/// When a type is decorated with `#[derive(Component)]`, several trait implementations are automatically added based on its structure:
///
/// - Depending on whether the type is a struct or an enum, the relevant `ComponentType<Struct>` or `ComponentType<Enum>` trait is implemented.
/// - Based on the presence of fields or variants, the type will implement either `EmptyComponent` or `NotEmptyComponent`.
/// - The `CachedComponentData` trait is implemented, providing storage mechanisms for the component.
///
/// # Requirements:
///
/// - Types deriving `CachedComponentData` should also implement `Clone` and `Default`.
///   For enums, you'll need to provide an explicit implementation of `Default`. Structs can often use `#[derive(Default)]` for a derived implementation.
///
/// # Note:
///
/// Ensure that enums annotated with `Component` have at least one variant; otherwise, a compile-time error will be triggered.
///
/// # Example:
///
/// ```ignore
/// #[derive(Clone, Default, Component)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Clone, Component)]
/// enum State {
///     Idle,
///     Running,
///     Jumping,
/// }
///
/// impl Default for State {
///     fn default() -> Self {
///         State::Idle
///     }
/// }
/// ```
#[proc_macro_derive(Component)]
pub fn component_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    // Parse the input tokens into a syntax tree
    let input = syn::parse_macro_input!(input as DeriveInput);

    // Build the output
    let expanded: TokenStream = match &input.data {
        Data::Struct(data_struct) => impl_cached_component_data_struct(data_struct, &input.ident),
        Data::Enum(_) => impl_cached_component_data_enum(&input),
        _ => quote! {
            compile_error!("The type is neither a struct nor an enum!");
        },
    };

    // Convert the generated code into a TokenStream and return it
    ProcMacroTokenStream::from(expanded)
}

fn impl_cached_component_data_struct(
    data_struct: &syn::DataStruct,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let has_fields = match &data_struct.fields {
        Fields::Named(fields) => !fields.named.is_empty(),
        Fields::Unnamed(fields) => !fields.unnamed.is_empty(),
        Fields::Unit => false,
    };

    if has_fields {
        quote! {
            impl NotEmptyComponent for #name {}

            impl ComponentType<Struct> for #name {}

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
        }
    } else {
        quote! {
            impl EmptyComponent for #name {}

            impl ComponentType<Struct> for #name {}

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
        }
    }
}

fn impl_cached_component_data_enum(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // Check if the enum has any variants
    let has_variants = match &ast.data {
        Data::Enum(data_enum) => !data_enum.variants.is_empty(),
        _ => panic!("Expected enum data!"), // This shouldn't happen as we check before calling this function
    };

    // If it has variants, produce the NotEmptyTrait implementation. Otherwise, produce a compile error.
    let not_empty_trait_or_error = if has_variants {
        quote! {
            impl NotEmptyComponent for #name {}
        }
    } else {
        quote! {
            compile_error!("Enum components should have at least one variant!");
        }
    };

    quote! {
        impl ComponentType<Enum> for #name {}

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

        #not_empty_trait_or_error
    }
}
