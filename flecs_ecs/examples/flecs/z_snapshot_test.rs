use flecs_ecs::prelude::*;

#[macro_export]
macro_rules! snapshot_test {
    () => {
        #[derive(Component)]
        pub struct Snap {
            pub str: Vec<String>,
        }

        impl Module for Snap {
            fn module(world: &World) {
                let snap = Snap::setup_snapshot_test();
                world.insert(snap);
            }
        }

        impl Snap {
            pub fn setup_snapshot_test() -> Snap {
                Snap { str: Vec::new() }
            }

            pub fn push(&mut self, str: String) {
                self.str.push(str);
            }

            pub fn count(&self) -> usize {
                self.str.len()
            }

            pub fn test(&self, str: String) {
                let mut settings = insta::Settings::clone_current();
                settings
                    ._private_inner_mut()
                    .filters((vec![(r"id: (\d+)\s", "[ID] ")]));
                settings.set_prepend_module_to_snapshot(false);
                settings.set_snapshot_suffix(str);
                settings.bind(|| {
                    insta::assert_yaml_snapshot!(self.str);
                })
            }

            pub fn from(world: &World) -> Self {
                Self {
                    str: std::mem::take(&mut world.get_mut::<Snap>().str),
                }
            }
        }

        macro_rules! fprintln {
            ($world:expr) => {{
                let world = ($world).world();
                world.get_mut::<Snap>().push(format!("\n"));
                println!();
            }};
            ($world:expr, $format_string:expr) => {{
                let world = ($world).world();
                world.get_mut::<Snap>().push(format!($format_string));
                println!($format_string);
            }};
            //recursive macro failed to work due to macro within macro declaration I suspect.
            ($world:expr, $format_string:expr, $arg1:expr) => {{
                let world = ($world).world();
                world.get_mut::<Snap>().push(format!($format_string, $arg1));
                println!($format_string, $arg1);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2));
                println!($format_string, $arg1, $arg2);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2, $arg3));
                println!($format_string, $arg1, $arg2, $arg3);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4: expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2, $arg3, $arg4));
                println!($format_string, $arg1, $arg2, $arg3, $arg4);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4: expr, $arg5:expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5));
                println!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4: expr, $arg5:expr, $arg6:expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5, $arg6));
                println!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5, $arg6);
            }};
            ($world:expr, $format_string:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4: expr, $arg5:expr, $arg6:expr, $arg7:expr) => {{
                let world = ($world).world();
                world
                    .get_mut::<Snap>()
                    .push(format!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5, $arg6, $arg7));
                println!($format_string, $arg1, $arg2, $arg3, $arg4, $arg5, $arg6, $arg7);
            }};
        }
    };
}

pub use snapshot_test;
