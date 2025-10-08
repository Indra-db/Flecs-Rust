//#![cfg_attr(not(feature = "std"), no_std)] // Enable `no_std` if `std` feature is disabled
//#![allow(dead_code, unused)]

extern crate proc_macro;

//#[cfg(feature = "std")]
//extern crate std;

#[macro_use]
extern crate alloc;

use proc_macro::TokenStream as ProcMacroTokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

use crate::tuples::Tuples;
#[cfg(feature = "flecs_query_rust_traits")]
use syn::Ident;

mod component;
mod dsl;
mod extern_abi;
#[cfg(feature = "flecs_query_rust_traits")]
mod rust_traits;
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
    let input = parse_macro_input!(input as DeriveInput);
    component::expand_component_derive(input).into()
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
    dsl::expand_query(input).into()
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
    dsl::expand_system(input).into()
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
    dsl::expand_observer(input).into()
}

/// Generates a `<TraitName>Trait` component struct with helper methods for Flecs-based dynamic trait registration.
///
/// See the [`rust_traits`] module for complete documentation, usage patterns, examples, and API reference.
///
/// [`rust_traits`]: crate::rust_traits
#[proc_macro]
pub fn ecs_rust_trait(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    #[cfg(feature = "flecs_query_rust_traits")]
    {
        let name = parse_macro_input!(input as Ident);
        rust_traits::expand_ecs_rust_trait(name).into()
    }

    #[cfg(not(feature = "flecs_query_rust_traits"))]
    {
        let _ = input;
        ProcMacroTokenStream::new()
    }
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
    let input_fn = parse_macro_input!(input as ItemFn);
    extern_abi::expand_extern_abi(input_fn).into()
}

/// Internal macro for generating tuple implementations.
///
/// This macro is used internally by the library and is not part of the public API.
#[doc(hidden)]
#[proc_macro]
pub fn tuples(input: ProcMacroTokenStream) -> ProcMacroTokenStream {
    let input = parse_macro_input!(input as Tuples);
    tuples::expand_tuples(input).into()
}
