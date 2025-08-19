//#![cfg_attr(not(feature = "std"), no_std)] // Enable `no_std` if `std` feature is disabled
//#![allow(dead_code, unused)]

extern crate proc_macro;

//#[cfg(feature = "std")]
//extern crate std;

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use proc_macro::TokenStream as ProcMacroTokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

use crate::tuples::Tuples;

mod component;
mod dsl;
mod tuples;

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
#[proc_macro_derive(Component, attributes(flecs_skip, on_registration, flecs))]
pub fn component_derive(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Collect #[flecs(...)] trait requests and options (e.g., meta) to apply on registration
    let (flecs_traits_calls, has_flecs_meta, flecs_name) =
        component::collect_flecs_traits_calls(&input);

    let has_repr_c = component::check_repr_c(&input);
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
            let is_tag = component::generate_tag_trait(has_fields);
            generated_impls.push(component::impl_cached_component_data_struct(
                &mut input,
                has_fields,
                &is_tag,
                has_on_registration,
                &flecs_traits_calls,
                &flecs_name,
            ));
        }
        Data::Enum(_) => {
            let is_tag = component::generate_tag_trait(!has_repr_c.0);
            if !has_repr_c.0 {
                generated_impls.push(component::impl_cached_component_data_struct(
                    &mut input,
                    true,
                    &is_tag,
                    has_on_registration,
                    &flecs_traits_calls,
                    &flecs_name,
                ));
            } else {
                generated_impls.push(component::impl_cached_component_data_enum(
                    &mut input,
                    has_on_registration,
                    has_repr_c.1,
                    &flecs_traits_calls,
                    &flecs_name,
                ));
            }
        }
        _ => return quote! { compile_error!("The type is neither a struct nor an enum!"); }.into(),
    };

    input.generics.make_where_clause();

    let meta_impl = component::impl_meta(&input, has_repr_c.0, input.ident.clone(), has_flecs_meta);

    let output = quote! {
        #( #generated_impls )*
        #meta_impl
    };

    output.into()
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
    let input = parse_macro_input!(input as dsl::Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = dsl::expand_dsl(&mut terms);
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
    let input = parse_macro_input!(input as dsl::Builder);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = dsl::expand_dsl(&mut terms);
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
    let input = parse_macro_input!(input as dsl::Observer);
    let mut terms = input.dsl.terms;

    let (iter_type, builder_calls) = dsl::expand_dsl(&mut terms);
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
///             let e = it.get_entity(i).unwrap();
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
    ProcMacroTokenStream::new()
}

/// Attribute macro that conditionally applies the appropriate extern ABI based on target platform.
///
/// For WASM targets (which don't support unwinding), it uses `extern "C"`.
/// For other targets, it uses `extern "C-unwind"`.
///
/// # Usage
///
/// ```ignore
/// use flecs_ecs_derive::extern_abi;
///
/// #[extern_abi]
/// fn my_function() {
///     // Function implementation
/// }
/// ```
///
/// This will expand to:
/// - `extern "C" fn my_function() { ... }` on WASM targets
/// - `extern "C-unwind" fn my_function() { ... }` on other targets
#[proc_macro_attribute]
pub fn extern_abi(
    _args: ProcMacroTokenStream,
    input: ProcMacroTokenStream,
) -> ProcMacroTokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;
    let fn_generics = &input_fn.sig.generics;
    let fn_where_clause = &input_fn.sig.generics.where_clause;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;

    // Check if there's already an extern specification
    if input_fn.sig.abi.is_some() {
        return quote! {
            compile_error!("Function already has an extern ABI specification. Remove it to use #[extern_abi].");
        }.into();
    }

    let output = quote! {
        #(#fn_attrs)*
        #[cfg(target_family = "wasm")]
        #fn_vis extern "C" fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause #fn_block

        #(#fn_attrs)*
        #[cfg(not(target_family = "wasm"))]
        #fn_vis extern "C-unwind" fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause #fn_block
    };

    ProcMacroTokenStream::from(output)
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
