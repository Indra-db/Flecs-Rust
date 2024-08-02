pub mod type_equality {
    #![doc(hidden)]
    pub trait EqType {
        type Itself;
    }

    impl<T> EqType for T {
        type Itself = T;
    }

    pub fn ty_must_eq<T, U>(_: T)
    where
        T: EqType<Itself = U>,
    {
    }

    /// Assert that a struct field has a given type.
    ///
    /// Source: <https://stackoverflow.com/a/70978292> (with minor modifications)
    ///
    /// Usage: `assert_is_type!(Struct, field: FieldType)`
    ///
    /// # Examples
    /// ```
    /// # use flecs_ecs::assert_is_type;
    /// struct Test {
    ///     field: u32,
    /// }
    ///
    /// struct Test2(u32);
    ///
    /// assert_is_type!(Test, field: u32);
    /// assert_is_type!(Test2, 0: u32);
    /// ```
    ///
    /// ```compile_fail
    /// # use flecs_ecs::assert_is_type;
    /// struct Test {
    ///     field: u32,
    /// }
    ///
    /// assert_is_type!(Test, field: &u32);
    /// ```
    #[macro_export]
    macro_rules! assert_is_type {
        ($t:ty, $i:tt: $ti:ty) => {
            const _: () = {
                #[allow(unused)]
                fn dummy(v: $t) {
                    $crate::addons::meta::macros::type_equality::ty_must_eq::<_, $ti>(v.$i);
                }
            };
        };
    }
}

/// Like [`stringify!`](::core::stringify!) but omits whitespace around generics.
#[macro_export]
macro_rules! component_type_stringify {
    ($t:tt <$($generic:ty),*>) => {
        ::core::concat!(::core::stringify!($t), "<", $($crate::component_type_stringify!($generic)),*, ">")
    };
    ($t:ty) => {
        ::core::stringify!($t)
    };
}

/// Function-like macro for registering a component's field metadata.
///
/// Intended to be used by [`component!`](crate::component) but can be used standalone.
///
/// Currently aliased to [`member_ext!`](crate::member_ext!).
#[macro_export]
macro_rules! member {
    ($($tt:tt)*) => {
        $crate::member_ext!($($tt)*);
    };
}

/// Function-like macro for registering an external component's field metadata.
///
/// Intended to be used by [`component_ext!`](crate::component_ext) but can be used standalone.
///
/// Field types are verified using [`assert_is_type!`](crate::assert_is_type!).
///
/// **Known issue:** due to bugs in Flecs, fields must be specified such that the field with offset = 0 comes first.
///
/// # Examples
/// ```
/// # use flecs_ecs::prelude::*;
/// # let world = World::new();
/// struct Struct {
///     foo: u32,
///     bar: u32,
///     array: [u32; 3],
/// };
/// let component = component_ext!(&world, Struct);
///
/// // If the component has already been fetched, you can provide it
/// member_ext!(&world, component: Struct, foo: u32);
///
/// // Otherwise, the macro will fetch it for you
/// member_ext!(&world, Struct, bar: u32);
///
/// member_ext!(&world, Struct, array: [u32; 3]);
/// ```
#[macro_export]
macro_rules! member_ext {
    ($world:expr, $compvar:ident: $component:ty, $name:tt : [$type:ty; $n:literal]) => {
        $crate::assert_is_type!($component, $name: [$type; $n]);
        $compvar.member_id(::flecs_ecs::prelude::id!($world, $type), ::core::stringify!($name), $n, ::core::mem::offset_of!($component, $name).try_into().unwrap())
    };
    ($world:expr, $compvar:ident: $component:ty, $name:tt : $type:ty) => {
        $crate::assert_is_type!($component, $name: $type);
        $compvar.member_id(::flecs_ecs::prelude::id!($world, $type), ::core::stringify!($name), 1, ::core::mem::offset_of!($component, $name).try_into().unwrap())
    };
    ($world:expr, $component:ty, $name:tt : $($tail:tt)*) => {{
        let world = $world;
        let component = $crate::component!(world, $component);
        $crate::member_ext!(world, component: $component, $name : $($tail)*);
    }};
}

#[allow(dead_code, clippy::print_stdout)]
/// Run this to regenerate the tuple rules for [`component_ext!`]
fn codegen_tuple_struct_macro() {
    for i in 1..=12 {
        let items = (1..=i).map(|j| (j - 1, (b'a' + j - 1) as char));
        print!("($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?");
        print!(
            "({} $(,)?)",
            items
                .clone()
                .map(|(_, var)| { format!("${var}:tt$(<$(${var}g:ty),*>)?") })
                .collect::<Vec<_>>()
                .join(", ")
        );
        print!(
            ") => {{$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?"
        );
        print!(
            "{{ {} }}",
            items
                .map(|(idx, var)| { format!("{idx}: ${var}$(<$(${var}g),*>)?") })
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(")}};");
    }
}

/// Function-like macro for registering a component, optionally including field metadata.
///
/// Supports structs, fieldless enums and tuple structs (up to 12 items).
/// Arrays are translated to flecs arrays (`count`).
///
/// Field types are verified using [`assert_is_type!`](crate::assert_is_type!).
///
/// Returns the component.
///
/// Currently aliased to [`component_ext!`](crate::component_ext!).
#[macro_export]
macro_rules! component {
    ($($tt:tt)*) => {
        $crate::component_ext!($($tt)*)
    };
}

/// Function-like macro for registering an external component, optionally including field metadata.
///
/// Supports structs, fieldless enums and tuple structs (up to 12 items).
/// - Arrays are translated to flecs arrays (`count`).
/// - Generics are supported, but only type parameters can be passed. Use [`member_ext!`](crate::member_ext) directly if you need more complex types.
/// - Field types are verified using [`assert_is_type!`](crate::assert_is_type!).
///
/// Returns the component.
///
/// **Known issue:** due to bugs in Flecs, fields must be specified such that the field with offset = 0 comes first.
///
/// # Examples
/// ## Tuple structs
/// ```
/// # use flecs_ecs::prelude::*;
/// # let world = World::new();
/// struct TupleStruct(u32, u64);
///
/// component_ext!(&world, TupleStruct(u32, u64));
/// component_ext!(
///     &world,
///     #[name = "CustomName"]
///     TupleStruct(u32, u64)
/// );
/// ```
///
/// ## Structs
/// ```
/// # use flecs_ecs::prelude::*;
/// # let world = World::new();
/// struct Struct {
///     foo: u32,
///     bar: u64,
/// }
///
/// component_ext!(&world, Struct { foo: u32, bar: u64 });
/// component_ext!(
///     &world,
///     #[name = "CustomName"]
///     Struct { foo: u32, bar: u64 }
/// );
/// ```
///
/// ## Fieldless enums
/// ```
/// # use flecs_ecs::prelude::*;
/// # let world = World::new();
/// #[repr(u32)]
/// enum Enum {
///     Foo,
///     Bar,
///     Baz,
/// }
///
/// component_ext!(&world, Enum { Foo, Bar, Baz });
/// component_ext!(
///     &world,
///     #[name = "CustomName"]
///     Enum { Foo, Bar, Baz }
/// );
/// ```
///
/// ## Arrays
/// ```
/// # use flecs_ecs::prelude::*;
/// # let world = World::new();
/// struct TupleStruct([u32; 3]);
/// struct Struct {
///     field: [u32; 3],
/// }
///
/// component_ext!(&world, TupleStruct([u32; 3]));
/// component_ext!(&world, Struct { field: [u32; 3] });
/// ```
#[macro_export]
macro_rules! component_ext {
    // tuple struct
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0:    $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)?, $h:tt$(<$($hg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)?, 7: $h$(<$($hg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)?, $h:tt$(<$($hg:ty),*>)?, $i:tt$(<$($ig:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)?, 7: $h$(<$($hg),*>)?, 8: $i$(<$($ig),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)?, $h:tt$(<$($hg:ty),*>)?, $i:tt$(<$($ig:ty),*>)?, $j:tt$(<$($jg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)?, 7: $h$(<$($hg),*>)?, 8: $i$(<$($ig),*>)?, 9: $j$(<$($jg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)?, $h:tt$(<$($hg:ty),*>)?, $i:tt$(<$($ig:ty),*>)?, $j:tt$(<$($jg:ty),*>)?, $k:tt$(<$($kg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)?, 7: $h$(<$($hg),*>)?, 8: $i$(<$($ig),*>)?, 9: $j$(<$($jg),*>)?, 10: $k$(<$($kg),*>)? })};
    ($world:expr, $(#[name=$regname:literal])? $component:ident$(<$($generic:ty),*>)?($a:tt$(<$($ag:ty),*>)?, $b:tt$(<$($bg:ty),*>)?, $c:tt$(<$($cg:ty),*>)?, $d:tt$(<$($dg:ty),*>)?, $e:tt$(<$($eg:ty),*>)?, $f:tt$(<$($fg:ty),*>)?, $g:tt$(<$($gg:ty),*>)?, $h:tt$(<$($hg:ty),*>)?, $i:tt$(<$($ig:ty),*>)?, $j:tt$(<$($jg:ty),*>)?, $k:tt$(<$($kg:ty),*>)?, $l:tt$(<$($lg:ty),*>)? $(,)?)) => {$crate::component_ext!($world, $(#[name=$regname])? $component$(<$($generic),*>)?{ 0: $a$(<$($ag),*>)?, 1: $b$(<$($bg),*>)?, 2: $c$(<$($cg),*>)?, 3: $d$(<$($dg),*>)?, 4: $e$(<$($eg),*>)?, 5: $f$(<$($fg),*>)?, 6: $g$(<$($gg),*>)?, 7: $h$(<$($hg),*>)?, 8: $i$(<$($ig),*>)?, 9: $j$(<$($jg),*>)?, 10: $k$(<$($kg),*>)?, 11: $l$(<$($lg),*>)? })};

    // struct
    ($world:expr, #[name=$regname:literal] $component:ty $({$($name:tt : $type:tt$(<$($generic:ty),*>)?),* $(,)?})?) => {{
        let world = $world;
        let component = world.component_named_ext(::flecs_ecs::prelude::id!(world, $component), $regname);
        $($($crate::member_ext!(world, component: $component, $name: $type$(<$($generic),*>)?);)*)?
        component
    }};

    ($world:expr, $component:ty $({$($name:tt : $type:tt$(<$($generic:ty),*>)?),* $(,)?})?) => {{
        let world = $world;
        let component = world.component_named_ext(::flecs_ecs::prelude::id!(world, $component), $crate::component_type_stringify!($component));
        $($($crate::member_ext!(world, component: $component, $name: $type$(<$($generic),*>)?);)*)?
        component
    }};

    // fieldless enum
    ($world:expr, #[name=$regname:literal] $component:ty {$($name:tt),* $(,)?}) => {{
        let world = $world;
        let component = world.component_named_ext(::flecs_ecs::prelude::id!(world, $component), $regname);
        const _: () = assert!(::core::mem::size_of::<$component>() == 4, "Flecs demands that enums are 4 bytes");
        $(component.constant(::core::stringify!($name), <$component>::$name as i32);)*
        component
    }};

    ($world:expr, $component:ty {$($name:tt),* $(,)?}) => {{
        let world = $world;
        let component = world.component_named_ext(::flecs_ecs::prelude::id!(world, $component), $crate::component_type_stringify!($component));
        const _: () = assert!(::core::mem::size_of::<$component>() == 4, "Flecs demands that enums are 4 bytes");
        $(component.constant(::core::stringify!($name), <$component>::$name as i32);)*
        component
    }};
}
