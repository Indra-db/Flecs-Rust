extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    Data, DeriveInput, Fields, Ident, LitInt, Result,
};

#[proc_macro_derive(Raptor)]
pub fn repr_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let has_repr_c = check_repr_c(&input); // Ensure this function correctly checks for #[repr(C)]
    let name = &input.ident;

    let generated_code = match &input.data {
        Data::Enum(_) if !has_repr_c => {
            // Case for enums without #[repr(C)]
            quote! { impl flecs_ecs::core::EmptyComponent for #name {} }
        }
        Data::Enum(_) => {
            // Case for enums with #[repr(C)]
            quote! { impl flecs_ecs::core::NotEmptyComponent for #name {} }
        }
        _ => {
            // Error case for non-enum types
            return quote! {
                compile_error!("The type is neither a struct nor an enum!");
            }
            .into();
        }
    };

    generated_code.into()
}

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

    let is_struct = matches!(input.data, Data::Struct(_));
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

    // Combine the generated code with the original struct definition
    let output = quote! {
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

        fn __register_lifecycle_hooks(mut type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
            use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
            const NEEDS_DROP: bool = <#name as flecs_ecs::core::component_registration::registration_traits::ComponentInfo>::NEEDS_DROP;
            const IMPLS_CLONE: bool = #name::IMPLS_CLONE;
            const IMPLS_DEFAULT: bool = #name::IMPLS_DEFAULT;

            flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(&mut type_hooks);

            if IMPLS_CLONE {
                flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type,>(&mut type_hooks);
            } else {
                flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(&mut type_hooks);
            }

            if IMPLS_DEFAULT {
                flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type,>(&mut type_hooks);
            } else {
                flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(&mut type_hooks);
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

            fn __register_lifecycle_hooks(mut type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const NEEDS_DROP: bool = <#name as flecs_ecs::core::component_registration::registration_traits::ComponentInfo>::NEEDS_DROP;
                const IMPLS_CLONE: bool = #name::IMPLS_CLONE;
                const IMPLS_DEFAULT: bool = #name::IMPLS_DEFAULT;
                flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(&mut type_hooks);

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type,>(&mut type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(&mut type_hooks);
                }

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type,>(&mut type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(&mut type_hooks);
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
                fn __register_lifecycle_hooks(mut type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const NEEDS_DROP: bool = <#name<#ty> as flecs_ecs::core::component_registration::registration_traits::ComponentInfo>::NEEDS_DROP;
                    const IMPLS_CLONE: bool = #name::<#ty>::IMPLS_CLONE;
                    const IMPLS_DEFAULT: bool = #name::<#ty>::IMPLS_DEFAULT;
                    flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name<#ty>>(&mut type_hooks);

                    if IMPLS_CLONE {
                        flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type,>(&mut type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name<#ty>>(&mut type_hooks);
                    }

                    if IMPLS_DEFAULT {
                        flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type,>(&mut type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name<#ty>>(&mut type_hooks);
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
                fn __register_lifecycle_hooks(mut type_hooks: &mut flecs_ecs::core::TypeHooksT)  {
                    use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                    const NEEDS_DROP: bool = <#name<#ty> as flecs_ecs::core::component_registration::registration_traits::ComponentInfo>::NEEDS_DROP;
                    const IMPLS_CLONE: bool = #name::<#ty>::IMPLS_CLONE;
                    const IMPLS_DEFAULT: bool = #name::<#ty>::IMPLS_DEFAULT;
                    flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name<#ty>>(&mut type_hooks);

                    if IMPLS_CLONE {
                        flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type,>(&mut type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name<#ty>>(&mut type_hooks);
                    }

                    if IMPLS_DEFAULT {
                        flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name<#ty>>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type,>(&mut type_hooks);
                    } else {
                        flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name<#ty>>(&mut type_hooks);
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
