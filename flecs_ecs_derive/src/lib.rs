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
/// ## Requirements:
///
/// - Types deriving `CachedComponentData` should also implement `Clone` and `Default`.
///   For enums, you'll need to provide an explicit implementation of `Default`. Structs can often use `#[derive(Default)]` for a derived implementation.
///
/// ### Note:
///
/// Ensure that enums annotated with `Component` have at least one variant; otherwise, a compile-time error will be triggered.
///
/// ## Example:
///
#[cfg_attr(doctest, doc = " ````no_test")]
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

// This function generates a series of trait implementations for structs.
// The implementations depend on the presence or absence of fields in the struct.
fn impl_cached_component_data_struct(
    data_struct: &syn::DataStruct, // Parsed data structure from the input token stream
    name: &syn::Ident,             // Name of the structure
) -> proc_macro2::TokenStream {
    // Determine if the struct has fields
    let has_fields = match &data_struct.fields {
        Fields::Named(fields) => !fields.named.is_empty(),
        Fields::Unnamed(fields) => !fields.unnamed.is_empty(),
        Fields::Unit => false,
    };

    // Common trait implementation for ComponentType and CachedComponentData
    let common_traits = quote! {
        impl ComponentType<Struct> for #name {}

        impl CachedComponentData for #name {

            fn register_explicit(world: *mut WorldT)
            {
                try_register_struct_component::<Self>(world);
            }

            fn get_data(world: *mut WorldT) -> &'static ComponentData
            {
                try_register_struct_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_data_unchecked() }
            }

            fn get_id(world: *mut WorldT) -> IdT {
                try_register_struct_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_id_unchecked() }
            }

            fn get_size(world: *mut WorldT) -> usize {
                try_register_struct_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_size_unchecked() }
            }

            fn get_alignment(world: *mut WorldT) -> usize {
                try_register_struct_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_alignment_unchecked() }
            }

            fn get_allow_tag(world: *mut WorldT) -> bool {
                try_register_struct_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_allow_tag_unchecked() }
            }

            fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
                static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
                &ONCE_LOCK
            }

            // Function for C compatibility, returns null-terminated string.
            fn get_symbol_name_c() -> &'static str {
                use std::any::type_name;
                static SYMBOL_NAME_C: OnceLock<String> = OnceLock::new();
                SYMBOL_NAME_C.get_or_init(|| {
                    let mut name = type_name::<Self>().replace("::", ".");
                    name.push('\0');  // Add null terminator to make it C compatible.
                    name
                })
            }

            // Function to return a &str slice without the null termination for Rust.
            fn get_symbol_name() -> &'static str {
                let name = Self::get_symbol_name_c();
                &name[..name.len() - 1]
            }
        }
    };

    // Specific trait implementation based on the presence of fields
    let specific_trait = if has_fields {
        quote! { impl NotEmptyComponent for #name {} }
    } else {
        quote! { impl EmptyComponent for #name {} }
    };

    // Combine common and specific trait implementations
    quote! {
        #specific_trait
        #common_traits
    }
}

fn generate_variant_constructor(
    variant: &syn::Variant,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => quote! { #name::#variant_ident },
        syn::Fields::Unnamed(fields) => {
            let defaults: Vec<_> = fields
                .unnamed
                .iter()
                .map(|_| quote! { Default::default() })
                .collect();
            quote! { #name::#variant_ident(#(#defaults),*) }
        }
        syn::Fields::Named(fields) => {
            let field_names: Vec<_> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let defaults: Vec<_> = field_names
                .iter()
                .map(|_| quote! { Default::default() })
                .collect();
            quote! { #name::#variant_ident { #(#field_names: #defaults),* } }
        }
    }
}

fn generate_variant_match_arm(
    variant: &syn::Variant,
    name: &syn::Ident,
    use_index: bool,
    index: usize,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let inner = if use_index {
        quote! { #index }
    } else {
        quote! {
            unsafe {
                let slice = concat!(stringify!(#variant_ident), "\0").as_bytes();
                std::ffi::CStr::from_bytes_with_nul_unchecked(slice)
            }
        }
    };

    match &variant.fields {
        syn::Fields::Unnamed(fields) => {
            let field_names: Vec<_> = fields.unnamed.iter().map(|_| quote!(_)).collect();
            quote! {
                #name::#variant_ident(#(#field_names),*) => {
                    #inner
                }
            }
        }
        syn::Fields::Named(fields) => {
            let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            quote! {
                #name::#variant_ident { #(#field_names),* } => {
                    #inner
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                #name::#variant_ident => {
                    #inner
                }
            }
        }
    }
}

fn impl_cached_component_data_enum(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // Ensure it's an enum and get the variants
    let variants = match &ast.data {
        syn::Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("#[derive(VariantName)] is only defined for enums"),
    };

    let variant_constructors: Vec<_> = variants
        .iter()
        .map(|variant| generate_variant_constructor(variant, name))
        .collect();

    let variant_name_arms: Vec<_> = variants
        .iter()
        .map(|variant| generate_variant_match_arm(variant, name, false, 0))
        .collect();

    let variant_index_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| generate_variant_match_arm(variant, name, true, index))
        .collect();

    let has_variants = !variants.is_empty();
    let size_variants = variants.len() as u32;
    let not_empty_trait_or_error = if has_variants {
        quote! { impl NotEmptyComponent for #name {} }
    } else {
        quote! { compile_error!("Enum components should have at least one variant!"); }
    };

    let cached_enum_data = quote! {
        impl CachedEnumData for #name {
            const SIZE_ENUM_FIELDS: u32 = #size_variants;
            type VariantIterator = std::vec::IntoIter<#name>;

            fn get_cstr_name(&self) -> &std::ffi::CStr {
                match self {
                    #(#variant_name_arms),*
                }
            }

            fn get_enum_index(&self) -> usize {
                match self {
                    #(#variant_index_arms),*
                }
            }

            fn __get_enum_data_ptr_mut() -> *mut u64 {
                static mut ENUM_FIELD_ENTITY_ID: [u64; #size_variants as usize] = [0; #size_variants as usize];
                unsafe { ENUM_FIELD_ENTITY_ID.as_mut_ptr() }
            }

            fn iter() -> Self::VariantIterator {
                vec![#(#variant_constructors),*].into_iter()
            }
        }
    };

    quote! {
        impl ComponentType<Enum> for #name {}

        impl CachedComponentData for #name {

            fn register_explicit(world: *mut WorldT)
            {
                try_register_enum_component::<Self>(world);
            }

            fn get_data(world: *mut WorldT) -> &'static ComponentData
            {
                try_register_enum_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_data_unchecked() }
            }

            fn get_id(world: *mut WorldT) -> IdT {
                try_register_enum_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_id_unchecked() }
            }

            fn get_size(world: *mut WorldT) -> usize {
                try_register_enum_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_size_unchecked() }
            }

            fn get_alignment(world: *mut WorldT) -> usize {
                try_register_enum_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_alignment_unchecked() }
            }

            fn get_allow_tag(world: *mut WorldT) -> bool {
                try_register_enum_component::<Self>(world);
                //this is safe because we checked if the component is registered / registered it
                unsafe { Self::get_allow_tag_unchecked() }
            }

            fn __get_once_lock_data() -> &'static OnceLock<ComponentData> {
                static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
                &ONCE_LOCK
            }

            // Function for C compatibility, returns null-terminated string.
            fn get_symbol_name_c() -> &'static str {
                use std::any::type_name;
                static SYMBOL_NAME_C: OnceLock<String> = OnceLock::new();
                SYMBOL_NAME_C.get_or_init(|| {
                    let mut name = type_name::<Self>().replace("::", ".");
                    name.push('\0');  // Add null terminator to make it C compatible.
                    name
                })
            }

            // Function to return a &str slice without the null termination for Rust.
            fn get_symbol_name() -> &'static str {
                let name = Self::get_symbol_name_c();
                &name[..name.len() - 1]
            }
        }

        #not_empty_trait_or_error

        #cached_enum_data
    }
}
