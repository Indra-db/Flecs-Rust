extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::{Bracket, Comma},
    Data, DeriveInput, Expr, Fields, Ident, LitInt, LitStr, Result, Token, Type,
};

/// `Component` macro for defining ECS components with optional register attribute when the type is generic over a single T.
///
/// When a type is decorated with `#[derive(Component)]`, several trait implementations are automatically added based on its structure:
///
/// - Depending on whether the type is a struct or an enum, the relevant `ComponentType<Struct>` or `ComponentType<Enum>` trait is implemented.
/// - Based on the presence of fields or variants, the type will implement either `EmptyComponent` or `NotEmptyComponent`.
/// - The `ComponentId` trait is implemented, providing storage mechanisms for the component.
///
/// The `register` attribute can be used to handle `ComponentId` implementation trait over a specific T in a generic component with the world.
/// This attribute is only supported when the type is generic over a single T.
///
/// ## Requirements:
///
/// - Types deriving `ComponentId` should also implement `Clone` and `Default` when the Type needs a `Drop`.
///   The `Default` implementation can usually be derived via `#[derive(Default)]`. For enums, you'll need to flag the default variant within the enumeration.
///
/// # Note:
///
/// Ensure that enums annotated with `Component` have at least one variant; otherwise, a compile-time error will be triggered.
///
/// ## Example:
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```ignore
/// #[derive(Component)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Component)]
/// #[register(Position)] //this will generate the ComponentId trait for the type Generic<Position>
/// struct Generic<T>
///     where T: Default + Clone
/// {
///     value: T,
/// }
///
/// #[derive(Component)]
/// #[repr(C)]
/// enum State {
///     #[default]
///     Idle,
///     Running,
///     Jumping,
/// }
/// ```
#[proc_macro_derive(Component, attributes(register))]
pub fn component_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let is_not_generic = input.generics.params.is_empty();
    let has_more_than_one_generic = input.generics.params.len() > 1;

    let mut is_struct = matches!(input.data, Data::Struct(_));
    let has_repr_c = check_repr_c(&input);
    let is_tag;
    let mut generated_impls = vec![];

    match input.data.clone() {
        Data::Struct(data_struct) => {
            let has_fields = match data_struct.fields {
                Fields::Named(ref fields) => !fields.named.is_empty(),
                Fields::Unnamed(ref fields) => !fields.unnamed.is_empty(),
                Fields::Unit => false,
            };
            is_tag = generate_tag_trait(has_fields);
            generated_impls.push(impl_cached_component_data_struct(
                &mut input, has_fields, &is_tag,
            ));
        }
        Data::Enum(_) => {
            is_tag = generate_tag_trait(!has_repr_c);
            if !has_repr_c {
                is_struct = true;
                generated_impls.push(impl_cached_component_data_struct(&mut input, true, &is_tag));
            } else {
                generated_impls.push(impl_cached_component_data_enum(&mut input));
            }
        }
        _ => return quote! { compile_error!("The type is neither a struct nor an enum!"); }.into(),
    };

    let name = &input.ident;

    let attrs = input
        .attrs
        .clone()
        .into_iter()
        .find(|attr| attr.path().is_ident("register"));

    let type_attrs = attrs.map(|attr| {
        attr.parse_args::<TypeAttributes>()
            .unwrap_or_else(|_| TypeAttributes(Vec::new()))
    });

    let mut has_attributes = false;
    let component_id_trait = if let Some(type_attrs) = type_attrs {
        type_attrs
            .0
            .into_iter()
            .map(|ty| {
                has_attributes = true;
                generate_component_id_impl(name, &ty, is_struct, &is_tag)
            })
            .collect()
    } else {
        quote! {}
    };

    let is_attribute_supported = if has_attributes && is_not_generic || has_more_than_one_generic {
        quote! { compile_error!("the register attribute can only be used when the type is generic over a single T.
        For more complex cases please implement `ComponentId` trait manually over the specialized T"); }
    } else {
        quote! {}
    };

    input.generics.make_where_clause();

    let (impl_generics, type_generics, where_clause) = &input.generics.split_for_impl();

    // Combine the generated code with the original struct definition
    let output = quote! {

        unsafe impl #impl_generics Send for #name #type_generics #where_clause{}
        unsafe impl #impl_generics Sync for #name #type_generics #where_clause{}

        #is_attribute_supported

        #component_id_trait

        #( #generated_impls )*
    };

    output.into()
}

fn generate_tag_trait(has_fields: bool) -> proc_macro2::TokenStream {
    if has_fields {
        quote! {
            const IS_TAG: bool = false;
            type TagType = flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
        }
    } else {
        quote! {
            const IS_TAG: bool = true;
            type TagType = flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
        }
    }
}

// This function generates a series of trait implementations for structs.
// The implementations depend on the presence or absence of fields in the struct.
fn impl_cached_component_data_struct(
    ast: &mut syn::DeriveInput, // Name of the structure
    has_fields: bool,
    is_tag: &TokenStream,
) -> proc_macro2::TokenStream {
    let is_generic = !ast.generics.params.is_empty();

    ast.generics.make_where_clause();

    let name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let component_info_impl = quote! {
        fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
            static ONCE_LOCK: std::sync::OnceLock<flecs_ecs::core::IdComponent> = std::sync::OnceLock::new();
            &ONCE_LOCK
        }

        fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
            flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(type_hooks);
        }

        fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
            use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
            const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;

            if IMPLS_DEFAULT {
                flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
            }
        }

        fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
            use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
            const IMPLS_CLONE: bool = #name::IMPLS_CLONE;

            if IMPLS_CLONE {
                flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
            } else {
                flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                    type_hooks,
                );
            }
        }
    };

    // Common trait implementation for ComponentType and ComponentId
    let common_traits = if !is_generic {
        quote! {
            impl #impl_generics  flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause{}

            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name #type_generics #where_clause {
                const IS_ENUM: bool = false;

                #is_tag
                const IMPLS_CLONE: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
                };
                const IMPLS_DEFAULT: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    } else {
        quote! {
            impl #impl_generics  flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause{}
        }
    };

    let component_id = if !is_generic {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentId for #name #type_generics #where_clause{
                type UnderlyingType = #name;
                type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;

                #component_info_impl
            }
        }
    } else {
        quote! {}
    };

    // Specific trait implementation based on the presence of fields
    let is_empty_component_trait = if has_fields {
        quote! { impl #impl_generics flecs_ecs::core::NotEmptyComponent for #name #type_generics #where_clause{} }
    } else {
        quote! { impl #impl_generics flecs_ecs::core::EmptyComponent for #name #type_generics #where_clause {} }
    };

    // Combine common and specific trait implementations
    quote! {
        #is_empty_component_trait
        #common_traits
        #component_id
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

fn impl_cached_component_data_enum(ast: &mut syn::DeriveInput) -> proc_macro2::TokenStream {
    let is_generic = !ast.generics.params.is_empty();

    ast.generics.make_where_clause();

    let name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

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
        quote! { impl #impl_generics flecs_ecs::core::NotEmptyComponent for #name #type_generics #where_clause {} }
    } else {
        quote! { compile_error!("Enum components should have at least one variant!"); }
    };

    let cached_enum_data_impl = quote! {
        const SIZE_ENUM_FIELDS: u32 = #size_variants;
        type VariantIterator = std::vec::IntoIter<#name #impl_generics>;

        fn name_cstr(&self) -> &std::ffi::CStr {
            match self {
                #(#variant_name_arms),*
            }
        }

        fn enum_index(&self) -> usize {
            const _: () = assert!(std::mem::size_of::<#name>()  == 4, "Enum size is not 4 bytes. For Flecs enum behaviour, the enum size must be 4 bytes");
            match self {
                #(#variant_index_arms),*
            }
        }

        fn __enum_data_mut() -> *mut u64 {
            static mut ENUM_FIELD_ENTITY_ID: [u64; #size_variants as usize] = [0; #size_variants as usize];
            unsafe { ENUM_FIELD_ENTITY_ID.as_mut_ptr() }
        }

        fn iter() -> Self::VariantIterator {
            vec![#(#variant_constructors),*].into_iter()
        }
    };

    let cached_enum_data = quote! {
        impl #impl_generics flecs_ecs::core::CachedEnumData for #name #type_generics #where_clause{
            #cached_enum_data_impl
        }

    };

    let component_info_impl = quote! {
            fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
                static ONCE_LOCK: std::sync::OnceLock<flecs_ecs::core::IdComponent> = std::sync::OnceLock::new();
                &ONCE_LOCK
            }


            fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(type_hooks);
            }

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_CLONE: bool = #name::IMPLS_CLONE;

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }
    };

    let component_id = if !is_generic {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentId for #name #type_generics #where_clause{
                type UnderlyingType = #name;
                type UnderlyingEnumType = #name;

                #component_info_impl
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics flecs_ecs::core::ComponentType<flecs_ecs::core::Enum> for #name #type_generics #where_clause {}


        impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name #type_generics #where_clause{
            const IS_ENUM: bool = true;
            const IS_TAG: bool = false;
            type TagType =
            flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
            const IMPLS_CLONE: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
            };
            const IMPLS_DEFAULT: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
            };
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
        }

        #component_id

        #not_empty_trait_or_error

        #cached_enum_data
    }
}

fn check_repr_c(input: &syn::DeriveInput) -> bool {
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_args_with(|input: ParseStream| {
                let mut found_repr_c = false;
                while !input.is_empty() {
                    let path = input.call(syn::Path::parse_mod_style)?;

                    if path.is_ident("C") {
                        found_repr_c = true;
                        break;
                    }
                }
                Ok(found_repr_c)
            });

            if let Ok(found_repr_c) = result {
                if found_repr_c {
                    return true; // Return true immediately if `#[repr(C)]` is found
                }
            }
        }
    }

    false // Return false if no `#[repr(C)]` is found
}

struct TypeAttributes(Vec<Ident>);

impl Parse for TypeAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut types = Vec::new();
        while !input.is_empty() {
            types.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(TypeAttributes(types))
    }
}

fn generate_component_id_impl(
    name: &Ident,
    ty: &Ident,
    is_struct: bool,
    is_tag: &TokenStream,
) -> TokenStream {
    if is_struct {
        quote! {
            impl flecs_ecs::core::component_registration::registration_traits::ComponentId for #name<#ty> {
                type UnderlyingType = #name<#ty>;
                type UnderlyingEnumType = flecs_ecs::core::component_registration::registration_types::NoneEnum;
                fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
                    static ONCE_LOCK: std::sync::OnceLock<flecs_ecs::core::IdComponent> = std::sync::OnceLock::new();
                    &ONCE_LOCK
                }
                fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                    flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name<#ty>>(type_hooks);
                }

                fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const IMPLS_DEFAULT: bool =  #name::<#ty>::IMPLS_DEFAULT;

                    if IMPLS_DEFAULT {
                        flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                    }
                }

                fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const IMPLS_CLONE: bool = #name::<#ty>::IMPLS_CLONE;

                    if IMPLS_CLONE {
                        flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name<#ty>>(
                            type_hooks,
                        );
                    }
                }
            }

            impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name<#ty> {
                const IS_ENUM: bool = false;

                #is_tag
                const IMPLS_CLONE: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsClone::<#name<#ty>>::IMPLS
                };
                const IMPLS_DEFAULT: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsDefault::<#name<#ty>>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    } else {
        quote! {
            impl flecs_ecs::core::component_registration::registration_traits::ComponentId for #name<#ty> {
                type UnderlyingType = #name<#ty>;
                type UnderlyingEnumType = #name<#ty>;
                fn __get_once_lock_data() -> &'static std::sync::OnceLock<flecs_ecs::core::IdComponent> {
                    static ONCE_LOCK: std::sync::OnceLock<flecs_ecs::core::IdComponent> = std::sync::OnceLock::new();
                    &ONCE_LOCK
                }
                fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                    flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name<#ty>>(type_hooks);
                }

                fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const IMPLS_DEFAULT: bool =  #name::<#ty>::IMPLS_DEFAULT;

                    if IMPLS_DEFAULT {
                        flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                    }
                }

                fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const IMPLS_CLONE: bool = #name::<#ty>::IMPLS_CLONE;

                    if IMPLS_CLONE {
                        flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name<#ty>>(
                            type_hooks,
                        );
                    }
                }
            }

            impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name<#ty> {
                const IS_ENUM: bool = true;
                const IS_TAG: bool = false;
                type TagType =
                flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
                const IMPLS_CLONE: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsClone::<#name<#ty>>::IMPLS
                };
                const IMPLS_DEFAULT: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsDefault::<#name<#ty>>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    }
}

struct Tuples {
    macro_ident: Ident,
    start: usize,
    end: usize,
    idents: Vec<Ident>,
}

impl Parse for Tuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        let mut idents = vec![];
        while input.parse::<Comma>().is_ok() {
            let ident = input.parse::<Ident>()?;
            idents.push(ident);
        }

        Ok(Tuples {
            macro_ident,
            start,
            end,
            idents,
        })
    }
}

#[proc_macro]
pub fn tuples(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Tuples);
    let len = 1 + input.end - input.start;
    let mut tuples = Vec::with_capacity(len);
    for i in 0..=len {
        tuples.push(format_ident!("P{}", i));
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let tuples = &tuples[..i];
        let idents = &input.idents;

        quote! {
            #macro_ident!(#(#idents,)* #(#tuples),*);
        }
    });

    ProcMacroTokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
enum Reference {
    Mut,
    Ref,
    #[default]
    None,
}

impl Parse for Reference {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            if input.peek(Token![mut]) {
                input.parse::<Token![mut]>()?;
                Ok(Reference::Mut)
            } else {
                Ok(Reference::Ref)
            }
        } else {
            Ok(Reference::None)
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
enum Access {
    In,
    Out,
    InOut,
    Filter,
    None,
    #[default]
    Omitted,
}

impl Parse for Access {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Bracket) {
            let inner;
            bracketed!(inner in input);
            if inner.peek(Token![in]) {
                inner.parse::<Token![in]>()?;
                Ok(Access::In)
            } else if inner.peek(kw::out) {
                inner.parse::<kw::out>()?;
                Ok(Access::Out)
            } else if inner.peek(kw::inout) {
                inner.parse::<kw::inout>()?;
                Ok(Access::InOut)
            } else if inner.peek(kw::filter) {
                inner.parse::<kw::filter>()?;
                Ok(Access::Filter)
            } else if inner.peek(kw::none) {
                inner.parse::<kw::none>()?;
                Ok(Access::None)
            } else {
                Ok(Access::Omitted)
            }
        } else {
            Ok(Access::Omitted)
        }
    }
}

#[derive(Debug)]
enum TermIdent {
    Local(Ident),
    Variable(LitStr),
    Type(Type),
    Literal(LitStr),
    SelfType,
    SelfVar,
    Singleton,
    Wildcard,
    Any,
}

impl Parse for TermIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(TermIdent::Wildcard)
        } else if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            Ok(TermIdent::Any)
        } else if input.peek(Token![$]) {
            // Variable
            input.parse::<Token![$]>()?;
            if input.peek(Ident) {
                Ok(TermIdent::Local(input.parse::<Ident>()?))
            } else if input.peek(LitStr) {
                Ok(TermIdent::Variable(input.parse::<LitStr>()?))
            } else if input.peek(Token![self]) {
                input.parse::<Token![self]>()?;
                Ok(TermIdent::SelfVar)
            } else {
                Ok(TermIdent::Singleton)
            }
        } else if input.peek(LitStr) {
            Ok(TermIdent::Literal(input.parse::<LitStr>()?))
        } else if input.peek(Token![Self]) {
            input.parse::<Token![Self]>()?;
            Ok(TermIdent::SelfType)
        } else {
            Ok(TermIdent::Type(input.parse::<Type>()?))
        }
    }
}

fn peek_id(input: &ParseStream) -> bool {
    input.peek(Ident)
        || input.peek(Token![*])
        || input.peek(Token![_])
        || input.peek(Token![$])
        || input.peek(LitStr)
        || input.peek(Token![Self])
}

#[derive(Default, Debug, PartialEq, Eq)]
enum TermOper {
    Not,
    Optional,
    AndFrom,
    NotFrom,
    OrFrom,
    Or,
    #[default]
    And,
}

mod kw {
    // Operators
    syn::custom_keyword!(and);
    syn::custom_keyword!(not);
    syn::custom_keyword!(or);

    // Traversal
    syn::custom_keyword!(cascade);
    syn::custom_keyword!(desc);
    syn::custom_keyword!(up);

    // Access
    syn::custom_keyword!(out);
    syn::custom_keyword!(inout);
    syn::custom_keyword!(filter);
    syn::custom_keyword!(none);
}

impl Parse for TermOper {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::and) {
            input.parse::<kw::and>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::AndFrom)
        } else if input.peek(kw::not) {
            input.parse::<kw::not>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::NotFrom)
        } else if input.peek(kw::or) {
            input.parse::<kw::or>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::OrFrom)
        } else if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            Ok(TermOper::Not)
        } else if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            Ok(TermOper::Optional)
        } else {
            Ok(TermOper::And)
        }
    }
}

#[derive(Default, Debug)]
struct TermId {
    ident: Option<TermIdent>,
    trav_self: bool,
    trav_up: bool,
    up_ident: Option<TermIdent>,
    trav_desc: bool,
    trav_cascade: bool,
    cascade_ident: Option<TermIdent>,
}

fn peek_trav(input: ParseStream) -> bool {
    input.peek(kw::cascade)
        || input.peek(kw::desc)
        || input.peek(kw::up)
        || input.peek(Token![self])
}

impl Parse for TermId {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = if !peek_trav(input) {
            let ident = input.parse::<TermIdent>()?;
            if input.peek(Token![|]) && !input.peek2(Token![|]) {
                input.parse::<Token![|]>()?;
            }
            Some(ident)
        } else {
            None
        };
        let mut out = Self {
            ident,
            ..Self::default()
        };
        while peek_trav(input) {
            if input.peek(kw::cascade) {
                input.parse::<kw::cascade>()?;
                out.trav_cascade = true;

                if input.peek(Ident) || input.peek(Token![$]) {
                    out.cascade_ident = Some(input.parse::<TermIdent>()?);
                }
            }
            if input.peek(kw::desc) {
                input.parse::<kw::desc>()?;
                out.trav_desc = true;
            }
            if input.peek(kw::up) {
                input.parse::<kw::up>()?;
                out.trav_up = true;

                if input.peek(Ident) || input.peek(Token![$]) {
                    out.up_ident = Some(input.parse::<TermIdent>()?);
                }
            }
            if input.peek(Token![self]) {
                input.parse::<Token![self]>()?;
                out.trav_self = true;
            }
            if input.peek(Token![|]) && !input.peek2(Token![|]) {
                input.parse::<Token![|]>()?;
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
enum TermType {
    Pair(TermId, TermId),
    Component(TermId),
}

#[derive(Debug)]
struct Term {
    access: Access,
    reference: Reference,
    oper: TermOper,
    source: TermId,
    ty: TermType,
}

impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        let access = input.parse::<Access>()?;
        let oper = input.parse::<TermOper>()?;
        let reference = input.parse::<Reference>()?;
        if peek_id(&input) {
            let initial = input.parse::<TermId>()?;
            if !input.peek(Token![,]) && !input.is_empty() {
                // Component or pair with explicit source
                let inner;
                parenthesized!(inner in input);
                let source = inner.parse::<TermId>()?;
                if inner.peek(Token![,]) {
                    // Pair
                    inner.parse::<Token![,]>()?;
                    let second = inner.parse::<TermId>()?;
                    Ok(Term {
                        access,
                        reference,
                        oper,
                        source,
                        ty: TermType::Pair(initial, second),
                    })
                } else {
                    // Component
                    Ok(Term {
                        access,
                        reference,
                        oper,
                        source,
                        ty: TermType::Component(initial),
                    })
                }
            } else {
                // Base case single component identifier
                Ok(Term {
                    access,
                    reference,
                    oper,
                    source: TermId::default(),
                    ty: TermType::Component(initial),
                })
            }
        } else {
            // Pair without explicit source
            let inner;
            parenthesized!(inner in input);
            let first = inner.parse::<TermId>()?;
            inner.parse::<Token![,]>()?;
            let second = inner.parse::<TermId>()?;
            Ok(Term {
                access,
                reference,
                oper,
                source: TermId::default(),
                ty: TermType::Pair(first, second),
            })
        }
    }
}

#[derive(Debug)]
struct Dsl {
    terms: Vec<Term>,
    doc: Option<TokenStream>,
}

impl Parse for Dsl {
    fn parse(input: ParseStream) -> Result<Self> {
        let string = input.cursor().token_stream().to_string();
        let stripped = string
            .replace('\"', "")
            .replace("& mut", "")
            .replace('&', "")
            .replace(" ,", ",");
        let string = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
        let doc = syn::parse_str::<TokenStream>(&format!("#[doc = \"{string}\"]")).ok();

        let mut terms = Vec::new();
        terms.push(input.parse::<Term>()?);
        while input.peek(Token![,]) || input.peek(Token![|]) {
            if input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                input.parse::<Token![|]>()?;
                terms.last_mut().unwrap().oper = TermOper::Or;
            } else {
                input.parse::<Token![,]>()?;
            }
            terms.push(input.parse::<Term>()?);
        }

        Ok(Dsl { terms, doc })
    }
}

struct Builder {
    name: Option<LitStr>,
    world: Expr,
    dsl: Dsl,
}

impl Parse for Builder {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = if input.peek(LitStr) {
            let name = input.parse::<LitStr>()?;
            input.parse::<Token![,]>()?;
            Some(name)
        } else {
            None
        };
        let world = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let dsl = input.parse::<Dsl>()?;
        Ok(Builder { name, world, dsl })
    }
}

fn expand_trav(term: &TermId) -> Vec<TokenStream> {
    let mut ops = Vec::new();
    if term.trav_up {
        match &term.up_ident {
            Some(ident) => match ident {
                TermIdent::Local(ident) => ops.push(quote! { .up_id(#ident) }),
                TermIdent::Type(ty) => ops.push(quote! { .up_type::<#ty>() }),
                _ => panic!("Invalid up traversal."),
            },
            None => ops.push(quote! { .up() }),
        }
    }
    if term.trav_cascade {
        match &term.cascade_ident {
            Some(ident) => match ident {
                TermIdent::Local(ident) => ops.push(quote! { .cascade_id(#ident) }),
                TermIdent::Type(ty) => ops.push(quote! { .cascade_type::<#ty>() }),
                _ => panic!("Invalid cascade traversal."),
            },
            None => ops.push(quote! { .cascade() }),
        }
    }
    if term.trav_desc {
        ops.push(quote! { .desc() });
    }
    if term.trav_self {
        ops.push(quote! { .self_() });
    }
    ops
}

fn expand_type(ident: &TermIdent) -> Option<TokenStream> {
    match ident {
        TermIdent::Type(ty) => Some(quote! { #ty }),
        TermIdent::Wildcard => Some(quote! { flecs::Wildcard }),
        TermIdent::Any => Some(quote! { flecs::Any }),
        TermIdent::SelfType => Some(quote! { Self }),
        _ => None,
    }
}

fn expand_term_type(term: &Term) -> Option<TokenStream> {
    let ty = match &term.ty {
        TermType::Pair(first, second) => {
            let first = first.ident.as_ref()?;
            let second = second.ident.as_ref()?;
            let first = expand_type(first)?;
            let second = expand_type(second)?;
            quote! { (#first, #second) }
        }
        TermType::Component(id) => {
            let id = id.ident.as_ref()?;
            expand_type(id)?
        }
    };

    let access_type = match term.reference {
        Reference::Mut => quote! { &mut #ty },
        Reference::Ref => quote! { & #ty },
        Reference::None => return None,
    };

    match &term.oper {
        TermOper::Optional => Some(quote! { Option<#access_type> }),
        TermOper::And => Some(quote! { #access_type }),
        _ => None,
    }
}

fn expand_dsl(terms: &mut [Term]) -> (TokenStream, Vec<TokenStream>) {
    let mut iter_terms = Vec::new();
    for t in terms.iter() {
        match expand_term_type(t) {
            Some(ty) => iter_terms.push(ty),
            None => break,
        };
    }
    let iter_type = if iter_terms.len() == 1 {
        quote! {
            #( #iter_terms )*
        }
    } else {
        quote! {
            (#(
                #iter_terms,
            )*)
        }
    };
    let builder_calls = terms
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let index = i as i32;
            let mut ops = Vec::new();
            let mut needs_accessor = false;
            let iter_term = i < iter_terms.len();
            let mut term_accessor = if !iter_term {
                quote! { .term() }
            } else {
                quote! { .term_at(#index) }
            };

            match &t.ty {
                TermType::Pair(first, second) => {
                    let first_id = first.ident.as_ref().expect("Pair with no first.");
                    let second_id = second.ident.as_ref().expect("Pair with no second.");
                    let first_ty = expand_type(first_id);
                    let second_ty = expand_type(second_id);

                    match first_id {
                        TermIdent::Variable(var) => {
                            let var_name = format!("${}", var.value());
                            ops.push(quote! { .set_first_name(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_first_id(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_first_id(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_first_name(#lit) }),
                        TermIdent::Singleton => panic!("Unexpected singleton identifier."),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_first::<#first_ty>() });
                            }
                        }
                    };

                    match second_id {
                        TermIdent::Variable(var) => {
                            let var_name = format!("${}", var.value());
                            ops.push(quote! { .set_second_name(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_second_id(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_second_id(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_second_name(#lit) }),
                        TermIdent::Singleton => panic!("Unexpected singleton identifier."),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_second::<#second_ty>() });
                            }
                        }
                    };

                    // Configure traversal for first
                    let id_ops = expand_trav(first);
                    if !id_ops.is_empty() {
                        ops.push(quote! { .first() #( #id_ops )* });
                    }

                    // Configure traversal for second
                    let id_ops = expand_trav(second);
                    if !id_ops.is_empty() {
                        ops.push(quote! { .second() #( #id_ops )* });
                    }
                }
                TermType::Component(term) => {
                    let id = term.ident.as_ref().expect("Term with no component.");
                    let ty = expand_type(id);

                    match id {
                        TermIdent::Variable(var) => {
                            let var_name = var.value();
                            ops.push(quote! { .set_var(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_id(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_id(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .name(#lit) }),
                        TermIdent::Singleton => panic!("Unexpected singleton identifier."),
                        _ => {
                            if !iter_term {
                                term_accessor = quote! { .with::<#ty>() };
                                needs_accessor = true;
                            }
                        }
                    };

                    // Configure traversal
                    let id_ops = expand_trav(term);
                    if !id_ops.is_empty() {
                        ops.push(quote! { #( #id_ops )* });
                    }
                }
            }

            // Configure source
            if let Some(source) = &t.source.ident {
                let ty = expand_type(source);
                match source {
                    TermIdent::Variable(var) => {
                        let var_name = format!("${}", var.value());
                        ops.push(quote! { .set_src_name(#var_name) });
                    }
                    TermIdent::SelfVar => ops.push(quote! { .set_src_id(self) }),
                    TermIdent::Local(ident) => ops.push(quote! { .set_src_id(#ident) }),
                    TermIdent::Literal(lit) => ops.push(quote! { .set_src_name(#lit) }),
                    TermIdent::Singleton => ops.push(quote! { .singleton() }),
                    _ => ops.push(quote! { .set_src::<#ty>() }),
                };
            }

            // Configure operator
            match &t.oper {
                TermOper::Not => ops.push(quote! { .not() }),
                TermOper::Optional => ops.push(quote! { .optional() }),
                TermOper::Or => ops.push(quote! { .or() }),
                TermOper::AndFrom => ops.push(quote! { .and_from() }),
                TermOper::NotFrom => ops.push(quote! { .not_from() }),
                TermOper::OrFrom => ops.push(quote! { .or_from() }),
                TermOper::And => {}
            }

            // Configure traversal for source
            let id_ops = expand_trav(&t.source);
            if !id_ops.is_empty() {
                ops.push(quote! { .src() #( #id_ops )* });
            }

            // Configure access
            if !iter_term {
                ops.push(match &t.reference {
                    Reference::Mut => quote! { .set_inout() },
                    Reference::Ref => quote! { .set_in() },
                    Reference::None => quote! { .set_inout_none() },
                });
            }

            match &t.access {
                Access::In => ops.push(quote! { .set_in() }),
                Access::Out => ops.push(quote! { .set_out() }),
                Access::InOut => ops.push(quote! { .set_inout() }),
                Access::Filter => ops.push(quote! { .filter() }),
                Access::None => ops.push(quote! { .set_inout_none() }),
                Access::Omitted => {}
            }

            if !ops.is_empty() || needs_accessor {
                Some(quote! {
                    #term_accessor
                    #( #ops )*
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    (iter_type, builder_calls)
}

#[proc_macro]
pub fn query(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;
    let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            #doc
            #world.query_named::<#iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            #doc
            #world.query::<#iter_type>()
            #(
                #builder_calls
            )*
        },
    };
    ProcMacroTokenStream::from(output)
}

#[proc_macro]
pub fn system(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;

    let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            #doc

            #world.system_named::<#iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            #doc
            #world.system::<#iter_type>()
            #(
                #builder_calls
            )*
        },
    };
    ProcMacroTokenStream::from(output)
}

struct Observer {
    name: Option<LitStr>,
    world: Expr,
    event: Type,
    dsl: Dsl,
}

impl Parse for Observer {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = if input.peek(LitStr) {
            let name = input.parse::<LitStr>()?;
            input.parse::<Token![,]>()?;
            Some(name)
        } else {
            None
        };
        let world = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let event = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;
        let dsl = input.parse::<Dsl>()?;
        Ok(Observer {
            name,
            world,
            event,
            dsl,
        })
    }
}

#[proc_macro]
pub fn observer(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Observer);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let event_type = input.event;
    let world = input.world;

    let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            #doc
            #world.observer_named::<#event_type, #iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            #doc
            #world.observer::<#event_type, #iter_type>()
            #(
                #builder_calls
            )*
        },
    };

    ProcMacroTokenStream::from(output)
}
