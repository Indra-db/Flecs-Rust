//#![cfg_attr(not(feature = "std"), no_std)] // Enable `no_std` if `std` feature is disabled

extern crate proc_macro;

//#[cfg(feature = "std")]
//extern crate std;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use alloc::{format, string::ToString, vec::Vec};

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DeriveInput, Expr, Fields, Ident, LitInt, LitStr, Path, Result, Token, Type, bracketed,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::{Bracket, Comma},
};

/// `Component` macro for defining Flecs ECS components.
///
/// When a type is decorated with `#[derive(Component)]`, several trait implementations are automatically added based on its structure:
///
/// - Depending on whether the type is a struct or Rust enum or `repr(C)` enum.
///   when it's a struct or Rust Enum it implements`ComponentType<Struct>` and in a C compatible enum the `ComponentType<Enum>` trait is also implemented.
/// - Based on the presence of fields or variants, the type will implement either `TagComponent` or `DataComponent`.
/// - The `ComponentId` trait is implemented, providing storage mechanisms for the component.
///
/// # Generic types
/// - Generic types are supported, but they don't have first-class support for the `ComponentId` trait where it automatically registers the
///   ctor and copy hooks (Default & Clone) which are used for either `EntityView::add` or `EntityView::duplicate` and some other operations.
///   In that case, the user has to manually register the hooks for each variant of T of the generic component
///   by using `T::register_ctor_hook` and `T::register_clone_hook`.
///
/// # Enums:
///
/// Ensure that enums annotated with `Component` have at least one variant; otherwise, a compile-time error will be triggered.
///
/// ## Example:
///
/// ```ignore
/// #[derive(Component)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Component)]
/// struct Generic<T>
/// {
///     value: T,
/// }
///
/// #[derive(Component, Default)]
/// #[repr(C)]
/// enum State {
///     #[default]
///     Idle,
///     Running,
///     Jumping,
/// }
/// ```
#[proc_macro_derive(Component, attributes(meta, skip, on_registration))]
pub fn component_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let has_repr_c = check_repr_c(&input);
    let has_on_registration = input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("on_registration"));

    let mut generated_impls: Vec<TokenStream> = Vec::new();

    match input.data.clone() {
        Data::Struct(data_struct) => {
            let has_fields = match data_struct.fields {
                Fields::Named(ref fields) => !fields.named.is_empty(),
                Fields::Unnamed(ref fields) => !fields.unnamed.is_empty(),
                Fields::Unit => false,
            };
            let is_tag = generate_tag_trait(has_fields);
            generated_impls.push(impl_cached_component_data_struct(
                &mut input,
                has_fields,
                &is_tag,
                has_on_registration,
            ));
        }
        Data::Enum(_) => {
            let is_tag = generate_tag_trait(!has_repr_c.0);
            if !has_repr_c.0 {
                generated_impls.push(impl_cached_component_data_struct(
                    &mut input,
                    true,
                    &is_tag,
                    has_on_registration,
                ));
            } else {
                generated_impls.push(impl_cached_component_data_enum(
                    &mut input,
                    has_on_registration,
                    has_repr_c.1,
                ));
            }
        }
        _ => return quote! { compile_error!("The type is neither a struct nor an enum!"); }.into(),
    };

    input.generics.make_where_clause();

    let meta_impl = impl_meta(&input, has_repr_c.0, input.ident.clone());

    // Combine the generated code with the original struct definition
    let output = quote! {
        #( #generated_impls )*
        #meta_impl
    };

    output.into()
}

fn impl_meta(input: &DeriveInput, has_repr_c: bool, struct_name: Ident) -> TokenStream {
    let has_meta_attribute = input.attrs.iter().any(|attr| attr.path().is_ident("meta"));

    if !has_meta_attribute {
        return quote! {};
    }

    let mut meta_fields_impl = Vec::new();

    match input.data.clone() {
        Data::Struct(data_struct) => {
            if let Fields::Named(fields_named) = &data_struct.fields {
                for field in &fields_named.named {
                    let is_ignored = field.attrs.iter().any(|attr| attr.path().is_ident("skip"));

                    if is_ignored {
                        continue;
                    }

                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    if let Some(field_name) = field_name {
                        meta_fields_impl.push(quote! {
                            .member(id!(world, #field_type), (stringify!(#field_name), flecs_ecs::addons::meta::Count(1), core::mem::offset_of!(#struct_name, #field_name)))
                        });
                    } else {
                        meta_fields_impl.push( quote! {
                            compile_error!("Meta expects named fields, unnamed fields are not supported");
                        }
                        );
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            if !has_repr_c {
                meta_fields_impl.push( quote! {
                    compile_error!("Meta currently does not support Rust Algebraic enums, please use #[repr(C)] if it's a C compatible enum");
                }
                );
            } else {
                for variant in &data_enum.variants {
                    let is_ignored = variant
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("skip"));

                    if is_ignored {
                        continue;
                    }

                    let variant_name = &variant.ident;

                    meta_fields_impl.push(quote! {
                        .constant(stringify!(#variant_name), #struct_name::#variant_name)
                    });
                }
            }
        }
        _ => return quote! { compile_error!("The type is neither a struct nor an enum!"); },
    };

    let meta_fn_impl = quote! {
        use flecs_ecs::addons::meta::*;
        use flecs_ecs::core::WorldProvider;
        let world = component.world();
        let id = #struct_name::get_id(world);
        component
        #( #meta_fields_impl )*;
    };

    meta_impl_return(meta_fn_impl, struct_name)
}

#[cfg(feature = "flecs_meta")]
fn meta_impl_return(meta_fn_impl: TokenStream, struct_name: Ident) -> TokenStream {
    quote! {
        impl flecs_ecs::addons::Meta<#struct_name> for #struct_name {
            fn meta(component: flecs_ecs::core::Component<'_, #struct_name>) {
                #meta_fn_impl
            }
        }
    }
}

#[cfg(not(feature = "flecs_meta"))]
fn meta_impl_return(_meta_fn_impl: TokenStream, struct_name: Ident) -> TokenStream {
    quote! {
        impl flecs_ecs::addons::Meta<#struct_name> for #struct_name {
            fn meta(component: flecs_ecs::core::Component<'_, #struct_name>) {
                println!("Meta is not enabled, please enable the `flecs_meta` feature to use the meta attribute");
                // Optionally, you can leave this empty or provide an alternative implementation
            }
        }
    }
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

#[derive(Debug, Default)]
struct GenericTypeInfo {
    contains_type_bound: bool,
    contains_generic_type: bool,
    is_bound_default: bool,
    is_bound_clone: bool,
    is_bound_partial_ord: bool,
    is_bound_partial_eq: bool,
}

impl GenericTypeInfo {
    pub fn set_contains_type_bound(&mut self) {
        self.contains_type_bound = true;
    }

    pub fn set_contains_generic_type(&mut self) {
        self.contains_generic_type = true;
    }

    pub fn set_is_bound_default(&mut self) {
        self.is_bound_default = true;
    }

    pub fn set_is_bound_clone(&mut self) {
        self.is_bound_clone = true;
    }

    pub fn set_contains_partial_ord_bound(&mut self) {
        self.is_bound_partial_ord = true;
    }

    pub fn set_contains_partial_eq_bound(&mut self) {
        self.is_bound_partial_eq = true;
    }
}

// This function generates a series of trait implementations for structs.
// The implementations depend on the presence or absence of fields in the struct.
fn impl_cached_component_data_struct(
    ast: &mut syn::DeriveInput, // Name of the structure
    has_fields: bool,
    is_tag: &TokenStream,
    has_on_registration: bool,
) -> proc_macro2::TokenStream {
    let is_generic = !ast.generics.params.is_empty();

    ast.generics.make_where_clause();

    let name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let iter = &ast.generics.params.iter();
    let iter_where = where_clause.iter();
    let mut contains_lifetime_bound = false;

    let mut type_info_vec: Vec<(syn::Ident, GenericTypeInfo)> = Vec::new();

    //populate map with all the type generics
    iter.clone().for_each(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            type_info_vec.push((type_param.ident.clone(), Default::default()));
        }
    });

    iter.clone().for_each(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            // Find the corresponding entry in the vector.
            if let Some((_, type_info)) = type_info_vec
                .iter_mut()
                .find(|(id, _)| *id == type_param.ident)
            {
                type_info.set_contains_generic_type();

                if !type_param.bounds.empty_or_trailing() {
                    type_info.set_contains_type_bound();
                    type_param.bounds.iter().for_each(|bound| {
                        if let syn::TypeParamBound::Trait(trait_bound) = bound {
                            if trait_bound.path.is_ident("Default") {
                                type_info.set_is_bound_default();
                            } else if trait_bound.path.is_ident("Clone") {
                                type_info.set_is_bound_clone();
                            } else if trait_bound.path.is_ident("PartialOrd") {
                                type_info.set_contains_partial_ord_bound();
                            } else if trait_bound.path.is_ident("PartialEq") {
                                type_info.set_contains_partial_eq_bound();
                            }
                        }
                    });
                }
            }
        } else if let syn::GenericParam::Lifetime(_) = param {
            contains_lifetime_bound = true;
        }
    });

    iter_where.for_each(|where_clause| {
        for predicate in where_clause.predicates.iter() {
            if let syn::WherePredicate::Type(predicate_type) = predicate {
                // Extract the Ident from the bounded type
                if let Type::Path(type_path) = &predicate_type.bounded_ty {
                    if let Some(segment) = type_path.path.segments.first() {
                        let type_ident = &segment.ident;
                        for bound in predicate_type.bounds.iter() {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                if trait_bound.path.is_ident("Default") {
                                    if let Some((_, gtype_info)) =
                                        type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                    {
                                        gtype_info.set_is_bound_default();
                                    }
                                } else if trait_bound.path.is_ident("Clone") {
                                    if let Some((_, gtype_info)) =
                                        type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                    {
                                        gtype_info.set_is_bound_clone();
                                    }
                                } else if trait_bound.path.is_ident("PartialOrd") {
                                    if let Some((_, gtype_info)) =
                                        type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                    {
                                        gtype_info.set_contains_partial_ord_bound();
                                    }
                                } else if trait_bound.path.is_ident("PartialEq") {
                                    if let Some((_, gtype_info)) =
                                        type_info_vec.iter_mut().find(|(id, _)| *id == *type_ident)
                                    {
                                        gtype_info.set_contains_partial_eq_bound();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let mut contains_any_type_bound = false;
    let mut contains_any_generic_type = false;
    let mut contains_all_default_bound = true;
    let mut contains_all_clone_bound = true;
    let mut contains_all_partial_ord_bound = true;
    let mut contains_all_partial_eq_bound = true;

    type_info_vec.iter().for_each(|(_, type_info)| {
        if type_info.contains_type_bound {
            contains_any_type_bound = true;
        }
        if type_info.contains_generic_type {
            contains_any_generic_type = true;
        }
        contains_all_default_bound &= type_info.is_bound_default;
        contains_all_clone_bound &= type_info.is_bound_clone;
        contains_all_partial_ord_bound &=
            type_info.contains_type_bound && type_info.is_bound_partial_ord;
        contains_all_partial_eq_bound &=
            type_info.contains_type_bound && type_info.is_bound_partial_eq;
    });

    let mut contains_where_bound = false;
    if let Some(where_clause) = &ast.generics.where_clause {
        contains_where_bound = !where_clause.predicates.is_empty();
    }

    let hook_impl = if !is_generic {
        quote! {

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
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

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }
        }
    } else if contains_lifetime_bound && !contains_any_generic_type {
        quote! {

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::<'_>::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name::<'_> as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_CLONE: bool = #name::<'_>::IMPLS_CLONE;

                if IMPLS_CLONE {
                    flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
                } else {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::<'_>::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::<'_>::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
                }
            }
        }
    // } else if contains_any_generic_type && contains_all_default_bound && contains_all_clone_bound {
    //     quote! {
    //     fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
    //         use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
    //         const IMPLS_DEFAULT: bool =  #name::#type_generics::IMPLS_DEFAULT;

    //         if IMPLS_DEFAULT {
    //             flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
    //         }
    //     }

    //     fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
    //         use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
    //         const IMPLS_CLONE: bool = #name::#type_generics::IMPLS_CLONE;

    //         if IMPLS_CLONE {
    //             flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
    //         } else {
    //             flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name #type_generics>(
    //                 type_hooks,
    //             );
    //         }
    //     }
    //     }
    // } else if contains_any_generic_type && contains_all_default_bound {
    //     quote! {
    //         fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
    //             use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
    //             const IMPLS_DEFAULT: bool =  #name::#type_generics::IMPLS_DEFAULT;

    //             if IMPLS_DEFAULT {
    //                 flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
    //             }
    //         }
    //     }
    // } else if contains_any_generic_type && contains_all_clone_bound {
    //     quote! {
    //         fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
    //             use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
    //             const IMPLS_CLONE: bool = #name::#type_generics::IMPLS_CLONE;

    //             if IMPLS_CLONE {
    //                 flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,#name #type_generics>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
    //             } else {
    //                 flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name #type_generics>(
    //                     type_hooks,
    //                 );
    //             }
    //         }
    //     }
    } else {
        quote! {
            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                    flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<#name #type_generics>(
                        type_hooks,
                    );
            }
        }
    };

    let component_info_impl = quote! {
        #[inline(always)]
        fn index() -> u32 {
            static INDEX: ::core::sync::atomic::AtomicU32 = ::core::sync::atomic::AtomicU32::new(u32::MAX);
            Self::get_or_init_index(&INDEX)
        }

        fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t)  {
            flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name #type_generics>(type_hooks);
        }

        #hook_impl
    };

    let is_generic_const = if !contains_any_generic_type {
        quote! {
            const IS_GENERIC: bool = false;
        }
    } else {
        quote! {
            const IS_GENERIC: bool = true;
        }
    };
    let clone_default = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_default_bound && contains_all_clone_bound)
    {
        quote! {
            const IMPLS_CLONE: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
            };
            const IMPLS_DEFAULT: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
            };
        }
    } else if contains_any_generic_type && contains_all_default_bound {
        quote! {
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsDefault::<#name #type_generics>::IMPLS
            };
        }
    } else if contains_any_generic_type && contains_all_clone_bound {
        quote! {
            const IMPLS_CLONE: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsClone::<#name #type_generics>::IMPLS
            };
            const IMPLS_DEFAULT: bool = false;
        }
    } else {
        quote! {
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
        }
    };

    let partial_ord_bound = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_partial_ord_bound)
    {
        quote! {
            const IMPLS_PARTIAL_ORD: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
            };
        }
    } else {
        quote! {
            const IMPLS_PARTIAL_ORD: bool = false;
        }
    };

    let partial_eq_bound = if !is_generic
        || (contains_lifetime_bound && !contains_any_generic_type)
        || (contains_any_generic_type && contains_all_partial_eq_bound)
    {
        quote! {
            const IMPLS_PARTIAL_EQ: bool = {
                use flecs_ecs::core::utility::traits::DoesNotImpl;
                flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
            };
        }
    } else {
        quote! {
            const IMPLS_PARTIAL_EQ: bool = false;
        }
    };

    // Common trait implementation for ComponentType and ComponentId
    let common_traits = {
        quote! {
            impl #impl_generics  flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause{}

            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name #type_generics #where_clause {
                #is_generic_const
                const IS_ENUM: bool = false;

                #is_tag
                #clone_default
                #partial_ord_bound
                #partial_eq_bound
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    };
    let where_clause_quote = if contains_where_bound {
        quote! { #where_clause Self: 'static }
    } else {
        quote! {
            where
            Self: 'static
        }
    };

    let component_id = quote! {
        impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentId for #name #type_generics
        #where_clause_quote
        {
            type UnderlyingType = #name #type_generics;
            type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
            type UnderlyingTypeOfEnum = flecs_ecs::core::component_registration::NoneEnum;
            #component_info_impl
        }
    };

    // Specific trait implementation based on the presence of fields
    let is_empty_component_trait = if has_fields {
        quote! { impl #impl_generics flecs_ecs::core::DataComponent for #name #type_generics #where_clause{} }
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::TagComponent for #name #type_generics #where_clause {}

            impl #impl_generics flecs_ecs::core::IntoEntity for #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = false;
                const IS_TYPED_MUT_REF: bool = false;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

            impl #impl_generics flecs_ecs::core::IntoEntity for &'static #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = true;
                const IS_TYPED_MUT_REF: bool = false;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

            impl #impl_generics flecs_ecs::core::IntoEntity for &'static mut #name #type_generics #where_clause {
                const IS_TYPED_PAIR: bool = false;
                const IS_TYPED: bool = true;
                const IF_ID_IS_DEFAULT: bool = true;
                const IS_TYPED_SECOND: bool = false;
                const IF_ID_IS_DEFAULT_SECOND: bool = false;
                const IS_ENUM: bool = false;
                const IS_TYPE_TAG: bool = true;
                const IS_TYPED_REF: bool = false;
                const IS_TYPED_MUT_REF: bool = true;
                fn into_entity<'a>(
                    self,
                    world: impl flecs_ecs::core::WorldProvider<'a>,
                ) -> flecs_ecs::core::Entity {
                    world.world().component_id::<Self>()
                }
            }

        }
    };

    let on_component_registration = if has_on_registration {
        quote! {}
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::OnComponentRegistration for #name #type_generics {
                fn on_component_registration(_world: flecs_ecs::core::WorldRef, _component_id: flecs_ecs::core::Entity) {}
            }
        }
    };

    // Combine common and specific trait implementations
    quote! {
        #is_empty_component_trait
        #common_traits
        #component_id
        #on_component_registration
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
                core::ffi::CStr::from_bytes_with_nul_unchecked(slice)
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

fn impl_cached_component_data_enum(
    ast: &mut syn::DeriveInput,
    has_on_registration: bool,
    underlying_enum_type: TokenStream,
) -> proc_macro2::TokenStream {
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
        quote! { impl #impl_generics flecs_ecs::core::DataComponent for #name #type_generics #where_clause {} }
    } else {
        quote! { compile_error!("Enum components should have at least one variant!"); }
    };

    let cached_enum_data_impl = quote! {
        const SIZE_ENUM_FIELDS: u32 = #size_variants;
        type VariantIterator = std::vec::IntoIter<#name #impl_generics>;

        fn name_cstr(&self) -> &core::ffi::CStr {
            match self {
                #(#variant_name_arms),*
            }
        }

        fn enum_index(&self) -> usize {
            //const _: () = assert!(core::mem::size_of::<#name>()  == 4, "Enum size is not 4 bytes. For Flecs enum behaviour, the enum size must be 4 bytes");
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
        impl #impl_generics flecs_ecs::core::EnumComponentInfo for #name #type_generics #where_clause{
            #cached_enum_data_impl
        }

    };

    let component_info_impl = quote! {
            #[inline(always)]
            fn index() -> u32 {
                static INDEX: ::core::sync::atomic::AtomicU32 = ::core::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }

            fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t)  {
                flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<#name>(type_hooks);
            }

            fn __register_default_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_DEFAULT: bool =  #name::IMPLS_DEFAULT;
                const IS_ENUM: bool =  <#name as ComponentInfo>::IS_ENUM;

                if IMPLS_DEFAULT {
                    flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
                } else if !IS_ENUM {
                    flecs_ecs::core::lifecycle_traits::register_ctor_panic_lifecycle_actions::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_clone_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
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

            fn __register_compare_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_ORD: bool = #name::IMPLS_PARTIAL_ORD;

                if IMPLS_PARTIAL_ORD {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_ORD,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialOrdType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_ord_panic_lifecycle_action::<#name>(
                        type_hooks,
                    );
                }
            }

            fn __register_equals_hooks(type_hooks: &mut flecs_ecs::sys::ecs_type_hooks_t) {
                use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
                const IMPLS_PARTIAL_EQ: bool = #name::IMPLS_PARTIAL_EQ;

                if IMPLS_PARTIAL_EQ {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_lifecycle_action::<<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_PARTIAL_EQ,#name>as flecs_ecs::core::component_registration::registration_traits::FlecsPartialEqType> ::Type, >(
                        type_hooks,
                    );
                } else {
                    flecs_ecs::core::lifecycle_traits::register_partial_eq_panic_lifecycle_action::<#name>(
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
                type UnderlyingTypeOfEnum = #underlying_enum_type;

                #component_info_impl
            }
        }
    } else {
        quote! {}
    };

    let component_info = if !is_generic {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name #type_generics #where_clause {
                const IS_GENERIC: bool = false;
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
                const IMPLS_PARTIAL_ORD: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_EQ: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::ComponentInfo for #name #type_generics #where_clause {
                const IS_GENERIC: bool = true;
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
                const IMPLS_PARTIAL_ORD: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialOrd::<#name #type_generics>::IMPLS
                };
                const IMPLS_PARTIAL_EQ: bool = {
                    use flecs_ecs::core::utility::traits::DoesNotImpl;
                    flecs_ecs::core::utility::types::ImplementsPartialEq::<#name #type_generics>::IMPLS
                };
                const IS_REF: bool = false;
                const IS_MUT: bool = false;
            }
        }
    };

    let on_component_registration = if has_on_registration {
        quote! {}
    } else {
        quote! {
            impl #impl_generics flecs_ecs::core::component_registration::registration_traits::OnComponentRegistration for #name #type_generics {
                fn on_component_registration(_world: flecs_ecs::core::WorldRef, _component_id: flecs_ecs::core::Entity) {}
            }
        }
    };

    let into_entity = quote! {
        impl #impl_generics flecs_ecs::core::IntoEntity for #name #type_generics #where_clause {
            const IS_TYPED_PAIR: bool = false;
            const IS_TYPED: bool = true;
            const IF_ID_IS_DEFAULT: bool = <Self as flecs_ecs::core::ComponentInfo>::IMPLS_DEFAULT;
            const IS_TYPED_SECOND: bool = true;
            const IF_ID_IS_DEFAULT_SECOND: bool = false;
            const IS_ENUM: bool = true;
            const IS_TYPE_TAG: bool = true;
            const IS_TYPED_REF: bool = false;
            const IS_TYPED_MUT_REF: bool = false;
            fn into_entity<'a>(self, world: impl flecs_ecs::core::WorldProvider<'a>) -> flecs_ecs::core::Entity {
                let world = world.world();
                *<Self as flecs_ecs::core::EnumComponentInfo>::id_variant(&self, world)
            }
        }
    };

    quote! {
        impl #impl_generics flecs_ecs::core::ComponentType<flecs_ecs::core::Enum> for #name #type_generics #where_clause {}
        impl #impl_generics flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for #name #type_generics #where_clause {}

        #component_info

        #component_id

        #not_empty_trait_or_error

        #cached_enum_data

        #on_component_registration

        #into_entity
    }
}

fn check_repr_c(input: &syn::DeriveInput) -> (bool, TokenStream) {
    let mut token_stream = TokenStream::new();

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_args_with(|input: ParseStream| {
                let mut found_repr_c = false;
                while !input.is_empty() {
                    let path = input.call(syn::Path::parse_mod_style)?;

                    if path.is_ident("C") {
                        found_repr_c = true;

                        // get the underlying ident type as tokenstream
                        token_stream = quote! {
                            i32
                        };
                        break;
                    } else if path.is_ident("i8")
                        || path.is_ident("u8")
                        || path.is_ident("i16")
                        || path.is_ident("u16")
                        || path.is_ident("i32")
                        || path.is_ident("u32")
                        || path.is_ident("i64")
                        || path.is_ident("u64")
                    {
                        found_repr_c = true;

                        // get the underlying ident type as tokenstream
                        let ident = path.get_ident().cloned().unwrap();
                        token_stream = quote! {
                            #ident
                        };
                        break;
                    }
                }
                Ok(found_repr_c)
            });

            if let Ok(found_repr_c) = result {
                if found_repr_c {
                    return (true, token_stream); // Return true immediately if `#[repr(C)]` is found
                }
            }
        }
    }

    (
        false,
        quote! { flecs_ecs::core::component_registration::NoneEnum },
    ) // Return false if no `#[repr(C)]` and variants is found
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
        let mut idents: Vec<Ident> = Vec::new();
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

enum TermIdent {
    Local(Ident),
    Variable(LitStr),
    Type(Type),
    EnumType(Path),
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
        } else if input.peek(kw::variant) {
            input.parse::<kw::variant>()?;
            Ok(TermIdent::EnumType(input.parse::<Path>()?))
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

    // For flecs enum type
    syn::custom_keyword!(variant);
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

struct TermId {
    ident: Option<TermIdent>,
    trav_self: bool,
    trav_up: bool,
    up_ident: Option<TermIdent>,
    trav_desc: bool,
    trav_cascade: bool,
    cascade_ident: Option<TermIdent>,
    span: Span,
}

impl TermId {
    fn new(ident: Option<TermIdent>, span: Span) -> Self {
        Self {
            ident,
            trav_self: false,
            trav_up: false,
            up_ident: None,
            trav_desc: false,
            trav_cascade: false,
            cascade_ident: None,
            span,
        }
    }
}

fn peek_trav(input: ParseStream) -> bool {
    input.peek(kw::cascade)
        || input.peek(kw::desc)
        || input.peek(kw::up)
        || input.peek(Token![self])
}

impl Parse for TermId {
    fn parse(input: ParseStream) -> Result<Self> {
        let span: Span = input.span();
        let ident = if !peek_trav(input) {
            let ident = input.parse::<TermIdent>()?;
            if input.peek(Token![|]) && !input.peek2(Token![|]) {
                input.parse::<Token![|]>()?;
            }
            Some(ident)
        } else {
            None
        };
        let mut out = Self::new(ident, span);

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

#[allow(clippy::large_enum_variant)]
enum TermType {
    Pair(TermId, TermId),
    Component(TermId),
}

struct Term {
    access: Access,
    reference: Reference,
    oper: TermOper,
    source: TermId,
    ty: TermType,
    span: Span,
}

impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();
        let access = input.parse::<Access>()?;
        let oper = input.parse::<TermOper>()?;
        let reference = input.parse::<Reference>()?;
        if peek_id(&input) {
            let initial = input.parse::<TermId>()?;
            if !input.peek(Token![,]) && !input.peek(Token![|]) && !input.is_empty() {
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
                        span,
                    })
                } else {
                    // Component
                    Ok(Term {
                        access,
                        reference,
                        oper,
                        source,
                        ty: TermType::Component(initial),
                        span,
                    })
                }
            } else {
                // Base case single component identifier
                Ok(Term {
                    access,
                    reference,
                    oper,
                    source: TermId::new(None, input.span()),
                    ty: TermType::Component(initial),
                    span,
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
                source: TermId::new(None, input.span()),
                ty: TermType::Pair(first, second),
                span,
            })
        }
    }
}

struct Dsl {
    terms: Vec<Term>,
    //TODO 2024 edition doesn't support it anymore. Need to find workaround
    _doc: Option<TokenStream>,
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
        let doc = doc.map(|doc| {
            quote! {
                #[allow(clippy::suspicious_doc_comments)]
                #doc
                const _: () = ();
            }
        });

        let mut terms = Vec::new();
        terms.push(input.parse::<Term>()?);
        while input.peek(Token![,]) || input.peek(Token![|]) {
            if input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                input.parse::<Token![|]>()?;
                terms.last_mut().unwrap().oper = TermOper::Or;
            } else {
                input.parse::<Token![,]>()?;

                // Handle optional trailing comma
                if input.is_empty() {
                    break;
                }
            }
            terms.push(input.parse::<Term>()?);
        }

        Ok(Dsl { terms, _doc: doc })
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
                TermIdent::Type(ty) => ops.push(quote! { .up_id(id::<#ty>()) }),
                _ => ops
                    .push(quote_spanned!(term.span => ; compile_error!("Invalid up traversal.") )),
            },
            None => ops.push(quote! { .up() }),
        }
    }
    if term.trav_cascade {
        match &term.cascade_ident {
            Some(ident) => match ident {
                TermIdent::Local(ident) => ops.push(quote! { .cascade_id(#ident) }),
                TermIdent::Type(ty) => ops.push(quote! { .cascade_id(id::<#ty>()) }),
                _ => ops.push(
                    quote_spanned!(term.span => ; compile_error!("Invalid cascade traversal.") ),
                ),
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
        TermIdent::EnumType(ty) => Some(quote! { #ty }),
        TermIdent::Wildcard => Some(quote! { flecs_ecs::core::flecs::Wildcard }),
        TermIdent::Any => Some(quote! { flecs_ecs::core::flecs::Any }),
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
            let index = i as u32;
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
                            ops.push(quote! { .set_first(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_first(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_first(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_first(#lit) }),
                        TermIdent::Singleton => ops.push(quote_spanned!{ first.span => ; compile_error!("Unexpected singleton identifier.") }),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_first(id::<#first_ty>()) });
                            }
                        }
                    };

                    match second_id {
                        TermIdent::Variable(var) => {
                            let var_name = format!("${}", var.value());
                            ops.push(quote! { .set_second(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_second(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_second(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_second(#lit) }),
                        TermIdent::Singleton => ops.push(quote_spanned!{ second.span => ; compile_error!("Unexpected singleton identifier.") }),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_second(id::<#second_ty>()) });
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
                        TermIdent::Singleton => ops.push(quote_spanned!{ term.span => ; compile_error!("Unexpected singleton identifier.") }),
                        TermIdent::EnumType(_) => {
                            if !iter_term {
                                term_accessor = quote! { .with_enum(#ty) };
                                needs_accessor = true;
                            }
                        },
                        _ => {
                            if !iter_term {
                                term_accessor = quote! { .with(id::<#ty>()) };
                                needs_accessor = true;
                            }
                        },
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
                        ops.push(quote! { .set_src(#var_name) });
                    }
                    TermIdent::SelfVar => ops.push(quote! { .set_src(self) }),
                    TermIdent::Local(ident) => ops.push(quote! { .set_src(#ident) }),
                    TermIdent::Literal(lit) => ops.push(quote! { .set_src(#lit) }),
                    TermIdent::Singleton => ops.push(quote! { .singleton() }),
                    _ => ops.push(quote! { .set_src(id::<#ty>()) }),
                };
            }

            // Configure operator

            if iter_term {
                if !matches!(t.oper, TermOper::And | TermOper::Optional) {
                    ops.push(quote_spanned!{
                        t.span => ; compile_error!("Only 'optional' and 'and' operators allowed for static terms.")
                    });
                }
            } else {
                match &t.oper {
                    TermOper::Not => ops.push(quote! { .not() }),
                    TermOper::Or => ops.push(quote! { .or() }),
                    TermOper::AndFrom => ops.push(quote! { .and_from() }),
                    TermOper::NotFrom => ops.push(quote! { .not_from() }),
                    TermOper::OrFrom => ops.push(quote! { .or_from() }),
                    TermOper::Optional => ops.push(quote! { .optional() }),
                    TermOper::And => {}
                }
            }

            // Configure traversal for source
            let id_ops = expand_trav(&t.source);
            if !id_ops.is_empty() {
                ops.push(quote! { .src() #( #id_ops )* });
            }

            // Configure access
            if iter_term {
                if !matches!(t.access, Access::Omitted | Access::Filter) {
                    ops.push(quote_spanned!{t.span => ; compile_error!("Only [filter] is allowed on static terms.")});
                }

                if t.access == Access::Filter {
                    ops.push(quote! { .filter() });
                }
            } else {
                match &t.reference {
                    Reference::None => {},
                    _ => ops.push(
                        quote_spanned!{
                            t.span => ; compile_error!("Static term located after a dynamic term, re-order such that `&` and `&mut are first.")
                        })
                }

                match &t.access {
                    Access::In => ops.push(quote! { .set_in() }),
                    Access::Out => ops.push(quote! { .set_out() }),
                    Access::InOut => ops.push(quote! { .set_inout() }),
                    Access::Filter => ops.push(quote! { .filter() }),
                    Access::None => ops.push(quote! { .set_inout_none() }),
                    Access::Omitted => {}
                }
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

/// Function-like macro for defining a query with `QueryBuilder`.
///
/// Usage: `query!("query_name", world, ... terms ...)`.
///
/// Returns `&mut QueryBuilder`.
///
/// Diverges from the [flecs query manual](https://github.com/SanderMertens/flecs/blob/master/docs/FlecsQueryLanguage.md) in the following respects:
///
/// 1. If the first argument is a string literal it will be used as a name.
/// 2. The next argument is a value implementing `WorldProvider`
/// 3. Terms prefixed with `&mut` or `&` will appear in the closure and must appear first:
/// ```ignore
/// // Like this:
/// query!(world, &mut MyComponent);
/// // Not like this:
/// query!(world, MyFilter, &mut MyComponent);
/// ```
/// 4. String literal terms will be matched by name:
/// ```ignore
/// query!(world, "MyComponent");
/// ```
/// 5. String literals prefixed by `$` are variables:
/// ```ignore
/// query!(world, &mut Location($"my_var"), (LocatedIn, $"my_var"));
/// ```
/// 6. Values that implement `Into<Entity>` prefixed by `$` will be used as ids:
/// ```ignore
/// query!(world, $my_entity);
/// ```
///
/// Other operators all function according to the manual.
///
/// Advanced operations are currently unsupported.
///
/// # Examples
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Component)]
/// struct Foo(u8);
///
/// #[derive(Component)]
/// struct Bar(u8);
///
/// #[derive(Component)]
/// struct Bazz;
///
/// let mut world = World::new();
///
/// // Basic
/// let builder = world.query::<(&Foo, &mut Bar)>().with(Bazz::id()).build();
/// let dsl = query!(&mut world, &Foo, &mut Bar, Bazz).build();
/// assert_eq!(builder.to_string(), dsl.to_string());
///
/// // Logical modifiers
/// let builder = world
///     .query::<()>()
///     .with(Foo::id())
///     .or()
///     .with(Bar::id())
///     .without(Bazz::id())
///     .build();
///
/// let dsl = query!(&mut world, Foo || Bar, !Bazz).build();
/// assert_eq!(builder.to_string(), dsl.to_string());
/// ```
#[proc_macro]
pub fn query(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;
    //TODO 2024 edition doesn't support it anymore. Need to find workaround
    //let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            //{
                //#doc
                (#world).query_named::<#iter_type>(#name)
                #(
                    #builder_calls
                )*
            //}
        },
        None => quote! {
            //{
                //#doc
                (#world).query::<#iter_type>()
                #(
                    #builder_calls
                )*
            //}
        },
    };
    ProcMacroTokenStream::from(output)
}

/// Function-like macro for defining a system with `SystemBuilder`.
///
/// Usage: `system!("system_name", world, ... terms ...)`.
///
/// Returns `&mut SystemBuilder`.
///
/// See [`query`] for examples & DSL divergences from the flecs spec.
///
/// [`query`]: macro@query
#[proc_macro]
pub fn system(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;

    //TODO 2024 edition doesn't support it anymore. Need to find workaround
    //let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            //{
                //#doc
                (#world).system_named::<#iter_type>(#name)
                #(
                    #builder_calls
                )*
            //}

        },
        None => quote! {
            //{
                //#doc
                (#world).system::<#iter_type>()
                #(
                    #builder_calls
                )*
            //}
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

/// Function-like macro for defining an observer with `ObserverBuilder`.
///
/// Usage: `observer!("observer_name", world, EventType, ... terms ...)`.
///
/// Returns `&mut ObserverBuilder`.
///
/// See [`query`] for examples & DSL divergences from the flecs spec.
///
/// [`query`]: macro@query
#[proc_macro]
pub fn observer(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Observer);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let event_type = input.event;
    let world = input.world;

    //TODO 2024 edition doesn't support it anymore. Need to find workaround
    //let doc = input.dsl.doc;
    let output = match input.name {
        Some(name) => quote! {
            //{
                //#doc
                (#world).observer_named::<#event_type, #iter_type>(#name)
                #(
                    #builder_calls
                )*
            //}
        },
        None => quote! {
            //{
                //#doc
                (#world).observer::<#event_type, #iter_type>()
                #(
                    #builder_calls
                )*
            //}
        },
    };

    ProcMacroTokenStream::from(output)
}

/// Generates a `<TraitName>Trait` component struct with helper methods for Flecs-based dynamic trait registration. You can then:
/// 1. Register a vtable for each implementor with `register_vtable::<T>()`.
/// 2. Cast back to a dynamic reference using `cast(entity, id)`.
///
/// # Example
/// ```ignore
/// use flecs_ecs::prelude::*;
///
/// pub trait Shapes {
///     fn calculate(&self) -> u64;
/// }
///
/// ecs_rust_trait!(Shapes);
///
/// #[derive(Component)]
/// pub struct Circle {
///     radius: f32,
/// }
///
/// impl Shapes for Circle {
///     fn calculate(&self) -> u64 {
///         1
///     }
/// }
///
/// #[derive(Component)]
/// pub struct Square {
///     side: f32,
/// }
///
/// impl Shapes for Square {
///     fn calculate(&self) -> u64 {
///         2
///     }
/// }
///
/// #[derive(Component)]
/// pub struct Triangle {
///     side: f32,
/// }
///
/// impl Shapes for Triangle {
///     fn calculate(&self) -> u64 {
///         3
///     }
/// }
///
/// let world = World::new();
///
/// // register the vtable per type that implements the trait
/// ShapesTrait::register_vtable::<Circle>(&world);
/// ShapesTrait::register_vtable::<Square>(&world);
/// ShapesTrait::register_vtable::<Triangle>(&world);
///
/// world.entity_named("circle").set(Circle { radius: 5.0 });
/// world.entity_named("square").set(Square { side: 5.0 });
/// world.entity_named("triangle").set(Triangle { side: 5.0 });
///
/// let query = world.query::<()>().with(ShapesTrait::id()).build();
///
/// query.run(|mut it| {
///     it.next();
///     while it.next() {
///         let world = it.world();
///         for i in it.iter() {
///             let e = it.entity(i).unwrap();
///             let id = it.id(0);
///             let shape = ShapesTrait::cast(e, id);
///             let calc = shape.calculate();
///             println!("{} - calc: {}", e.name(), calc);
///         }
///     }
/// });
///
/// // Output:
/// // circle - 34
/// // calc: 1
/// // square - 35
/// // calc: 2
/// // triangle - 36
/// // calc: 3
/// ```
#[proc_macro]
#[cfg(feature = "flecs_query_rust_traits")]
pub fn ecs_rust_trait(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let name = parse_macro_input!(input as Ident);

    let struct_name = format_ident!("{}Trait", name);

    let expanded = quote! {
        #[derive(Component, Default, Clone)]
        pub struct #struct_name {
            vtable: usize,
        }

        impl flecs_ecs::core::component_registration::registration_traits::RustTrait for #struct_name {}

        impl #struct_name {

            pub fn new(vtable: usize) -> Self {
                Self {
                    vtable
                }
            }

            pub fn register_vtable<T: #name + flecs_ecs::core::component_registration::registration_traits::ComponentId>(world: &flecs_ecs::core::World) -> usize {
                let trait_obj_ptr = std::ptr::NonNull::<T>::dangling() as std::ptr::NonNull<dyn #name>;
                let (_, vtable): (usize, usize) = unsafe { core::mem::transmute(trait_obj_ptr) };
                let id = world.component::<T>();
                let id_self = world.component::<Self>();
                id.is_a(id_self);
                id.set_id(Self::new(vtable), (id_self,id_self));
                vtable
            }

            pub fn cast<'a>(entity: flecs_ecs::core::EntityView, derived_id: flecs_ecs::core::IdView) -> &'a dyn #name {
                let data_ptr = entity.get_untyped(derived_id) as usize;
                let vtable_ptr = entity
                    .world()
                    .component_untyped_from(*derived_id)
                    .cloned::<&(Self,Self)>()
                    .vtable;

                unsafe { core::mem::transmute((data_ptr, vtable_ptr)) }
            }

            pub fn cast_mut<'a>(entity: flecs_ecs::core::EntityView, derived_id: flecs_ecs::core::IdView) -> &'a mut dyn #name {
                let data_ptr = entity.get_untyped_mut(derived_id) as usize;
                let vtable_ptr = entity
                    .world()
                    .component_untyped_from(*derived_id)
                    .cloned::<&(Self,Self)>()
                    .vtable;

                unsafe { core::mem::transmute((data_ptr, vtable_ptr)) }
            }
        }
    };

    ProcMacroTokenStream::from(expanded)
}

#[proc_macro]
#[cfg(not(feature = "flecs_query_rust_traits"))]
pub fn ecs_rust_trait(_: ProcMacroTokenStream) -> ProcMacroTokenStream {
    const {
        panic!(
            "The `flecs_query_rust_traits` feature must be enabled to use this procedural macro."
        )
    };
    ProcMacroTokenStream::new()
}
