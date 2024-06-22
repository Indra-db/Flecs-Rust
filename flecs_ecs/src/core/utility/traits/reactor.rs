use std::ffi::c_void;

use crate::core::*;

pub trait ReactorAPI<'a, P, T>: Builder<'a> + private::internal_ReactorAPI<'a, P, T>
where
    T: QueryTuple,
    P: ComponentId,
{
    /// Set context
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::ctx`
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "observer_builder_i::ctx")]
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self;

    fn each<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(T::TupleType<'_>) + 'static,
    {
        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        self.set_callback_binding_context(each_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));
        self.set_desc_callback(Some(
            Self::execute_each::<false, Func> as unsafe extern "C" fn(_),
        ));

        self.set_instanced(true);

        self.build()
    }

    fn each_entity<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(EntityView, T::TupleType<'_>) + 'static,
    {
        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        self.set_callback_binding_context(each_entity_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));
        self.set_desc_callback(Some(
            Self::execute_each_entity::<false, Func> as unsafe extern "C" fn(_),
        ));

        self.set_instanced(true);

        self.build()
    }

    fn each_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(Iter<false, P>, usize, T::TupleType<'_>) + 'static,
    {
        let each_iter_func = Box::new(func);
        let each_iter_static_ref = Box::leak(each_iter_func);

        self.set_callback_binding_context(each_iter_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_instanced(true);

        self.set_desc_callback(Some(
            Self::execute_each_iter::<Func> as unsafe extern "C" fn(_),
        ));

        self.set_instanced(true);

        self.build()
    }

    /// Run iterator. This operation expects manual iteration over the tables with `iter.next()` and `iter.iter()`.
    ///
    /// The "run" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Tag, &Position)>().run(move |mut it| {
    ///     println!("start operations");
    ///     while it.next() {
    ///         *count_tables_ref.borrow_mut() += 1;
    ///         let pos = it.field::<&Position>(1).unwrap(); //at index 1 in (&Tag, &Position)
    ///         for i in it.iter() {
    ///             *count_entities_ref.borrow_mut() += 1;
    ///             let entity = it.entity(i);
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     }
    ///     println!("end operations");
    /// });
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity: Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(Iter<true, P>) + 'static,
    {
        let run = Box::new(func);
        let run_static_ref = Box::leak(run);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));
        self.build()
    }

    /// run iter iterator. loops the iterator automatically compared to `run()`
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 3, y: 4 });
    ///
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 1, y: 2 });
    ///
    /// world
    ///     .entity()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 3, y: 4 });
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world
    ///     .system::<(&mut Position, &Velocity)>()
    ///     .run_iter(move |it, (pos, vel)| {
    ///         *count_tables_ref.borrow_mut() += 1;
    ///         for i in it.iter() {
    ///             *count_entities_ref.borrow_mut() += 1;
    ///             let entity = it.entity(i);
    ///             pos[i].x += vel[i].x;
    ///             pos[i].y += vel[i].y;
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     });
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// // Entity name:  -- id: 508 -- archetype: flecs_ecs.Tag, flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 3, y: 4 }
    /// // Entity name:  -- id: 510 -- archetype: flecs_ecs.Tag, flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 1, y: 2 }
    /// // Entity name:  -- id: 511 -- archetype: flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 3, y: 4 }
    /// ```
    fn run_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(Iter<false, P>, T::TupleSliceType<'_>) + 'static,
    {
        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);

        self.set_callback_binding_context(iter_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));
        self.set_desc_callback(Some(
            Self::execute_run_iter::<Func> as unsafe extern "C" fn(_),
        ));
        //TODO are we sure this shouldn't be instanced?
        self.build()
    }

    /// Run iterator with each forwarding. This operation expects manual
    /// iteration over the tables with `iter.next()` and `iter.each()`
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - `func`: (it: &mut Iter) + `func_each`: (comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    ///
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Tag, &Position)>().run_each(
    ///     move |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             *count_tables_ref.borrow_mut() += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     move |(tag, pos)| {
    ///         *count_entities_ref.borrow_mut() += 1;
    ///         println!("{:?}, {:?}", tag, pos);
    ///     },
    /// );
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each<Func, FuncEach>(
        &mut self,
        func: Func,
        func_each: FuncEach,
    ) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(Iter<true, P>) + 'static,
        FuncEach: FnMut(T::TupleType<'_>) + 'static,
    {
        let run_func = Box::new(func);
        let run_static_ref = Box::leak(run_func);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));

        let each_func = Box::new(func_each);
        let each_static_ref = Box::leak(each_func);

        self.set_callback_binding_context(each_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<FuncEach>));

        self.set_instanced(true);

        self.set_desc_callback(Some(
            Self::execute_each::<true, FuncEach> as unsafe extern "C" fn(_),
        ));

        self.build()
    }

    /// Run iterator with each entity forwarding. This operation expects manual
    /// iteration over the tables with `iter.next()` and `iter.each()`  
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    /// - `func`: (it: &mut Iter) + `func_each_entity`: (entity: `EntityView`, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    ///
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Tag, &Position)>().run_each_entity(
    ///     move |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             *count_tables_ref.borrow_mut() += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     move |e, (tag, pos)| {
    ///         *count_entities_ref.borrow_mut() += 1;
    ///         println!("{:?}: {:?}, {:?}", e, tag, pos);
    ///     },
    /// );
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity: Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each_entity<Func, FuncEachEntity>(
        &mut self,
        func: Func,
        func_each_entity: FuncEachEntity,
    ) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(Iter<true, P>) + 'static,
        FuncEachEntity: FnMut(EntityView, T::TupleType<'_>) + 'static,
    {
        let run_func = Box::new(func);
        let run_static_ref = Box::leak(run_func);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));

        let each_entity_func = Box::new(func_each_entity);
        let each_entity_static_ref = Box::leak(each_entity_func);

        self.set_callback_binding_context(each_entity_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<FuncEachEntity>));

        self.set_instanced(true);

        self.set_desc_callback(Some(
            Self::execute_each_entity::<true, FuncEachEntity> as unsafe extern "C" fn(_),
        ));

        self.build()
    }
}

macro_rules! implement_reactor_api {
    ($param:ty, $type:ty) => {
        impl<'a, T> internal_ReactorAPI<'a, $param, T> for $type
        where
            T: QueryTuple,
        {
            fn set_instanced(&mut self, instanced: bool) {
                self.is_instanced = instanced;
            }

            fn set_callback_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.callback_ctx = binding_ctx;
                self
            }

            fn set_callback_binding_context_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.callback_ctx_free = binding_ctx_free;
                self
            }

            fn set_run_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.run_ctx = binding_ctx;
                self
            }

            fn set_run_binding_context_free(
                &mut self,
                run_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.run_ctx_free = run_ctx_free;
                self
            }

            fn desc_binding_context(&self) -> *mut c_void {
                self.desc.callback_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }

            fn set_desc_run(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut sys::ecs_iter_t)>,
            ) {
                self.desc.run = callback;
            }
        }

        impl<'a, T> ReactorAPI<'a, $param, T> for $type
        where
            T: QueryTuple,
        {
            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
    ($type:ty) => {
        impl<'a, P, T> internal_ReactorAPI<'a, P, T> for $type
        where
            T: QueryTuple,
            P: ComponentId,
        {
            fn set_instanced(&mut self, instanced: bool) {
                self.is_instanced = instanced;
            }

            fn set_callback_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.callback_ctx = binding_ctx;
                self
            }

            fn set_callback_binding_context_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.callback_ctx_free = binding_ctx_free;
                self
            }

            fn set_run_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.run_ctx = binding_ctx;
                self
            }

            fn set_run_binding_context_free(
                &mut self,
                run_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.run_ctx_free = run_ctx_free;
                self
            }

            fn desc_binding_context(&self) -> *mut c_void {
                self.desc.callback_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }

            fn set_desc_run(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut sys::ecs_iter_t)>,
            ) {
                self.desc.run = callback;
            }
        }

        impl<'a, P, T> ReactorAPI<'a, P, T> for $type
        where
            T: QueryTuple,
            P: ComponentId,
        {
            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
}

pub(crate) use implement_reactor_api;
