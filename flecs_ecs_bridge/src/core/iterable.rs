use std::ptr;

use super::c_binding::bindings::{ecs_field_w_size, ecs_filter_desc_t};
use super::c_types::{IterT, WorldT};
use super::component_registration::CachedComponentData;
use super::entity::Entity;
use super::filter::Filter;
use super::utility::functions::ecs_field;
use super::world::World;

pub trait Iterable<F>: Sized
where
    F: FnMut(Entity, Self),
{
    fn get_data(it: &IterT, index: usize) -> Self;

    fn populate(filter: &mut Filter<Self, F>);

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t);
}

//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////
//these will be replaced with a macro later, for now I'm keeping it like this for easier testing / debugging / development
//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////
#[rustfmt::skip]
impl<F> Iterable<F> for ()
where
    F: FnMut(Entity, ()),
{
    fn get_data(_it: &IterT, _index: usize) -> Self {
        return ();
    }

    fn populate(filter : &mut Filter<Self, F>){}

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t){}
    
}
#[rustfmt::skip]
impl<'a, A, F> Iterable<F> for (&'a mut A,)
where
    F: FnMut(Entity, (&'a mut A,)),
    A: CachedComponentData,
{
    fn get_data(it: &IterT, index: usize) -> Self 
    {
        unsafe {
            let mut data = std::mem::MaybeUninit::<Self>::uninit();
            let data_ptr = data.as_mut_ptr();
            let ref_a = &mut(*ecs_field::<A>(it,1).add(index));
            ptr::write(data_ptr, (ref_a,));
            data.assume_init()
        }
    }

    fn populate(filter : &mut Filter<Self, F>)
    {
        let world = filter.world;
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t)
    {
        desc.terms[0].id = A::get_id(world);
    }
}

#[rustfmt::skip]
impl<'a, A, B, F> Iterable<F> for (&'a mut A, &'a mut B)
where
    F: FnMut(Entity, (&'a mut A, &'a mut B)),
    A: CachedComponentData,
    B: CachedComponentData,
{
    fn get_data(it: &IterT, index: usize) -> Self 
    {
        unsafe {
            let mut data = std::mem::MaybeUninit::<Self>::uninit();
            let data_ptr = data.as_mut_ptr();
            let ptr_a = ecs_field::<A>(it,1);
            let ptr_a_offset = ptr_a.add(index);
            let ref_a = &mut(*ptr_a_offset);
            let ref_b = &mut(*ecs_field::<B>(it,2).add(index));
            ptr::write(data_ptr, (ref_a,ref_b,));
            data.assume_init()
        }
    }

    fn populate(filter : &mut Filter<Self, F>)
    {
        let world = filter.world;
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t)
    {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, F> Iterable<F> for (&'a mut A, &'a mut B, &'a mut C)
where
    F: FnMut(Entity, (&'a mut A, &'a mut B, &'a mut C)),
    A: CachedComponentData,
    B: CachedComponentData,
    C: CachedComponentData,
{
    fn get_data(it: &IterT, index: usize) -> Self 
    {
        unsafe {
            let mut data = std::mem::MaybeUninit::<Self>::uninit();
            let data_ptr = data.as_mut_ptr();
            let ref_a = &mut(*ecs_field::<A>(it,1).add(index));
            let ref_b = &mut(*ecs_field::<B>(it,2).add(index));
            let ref_c = &mut(*ecs_field::<C>(it,3).add(index));
            ptr::write(data_ptr, (ref_a,ref_b,ref_c,));
            data.assume_init()
        }
    }

    fn populate(filter : &mut Filter<Self, F>)
    {
        let world = filter.world;
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t)
    {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[2].id = C::get_id(world);
    }
}

//pub fn apply_to_func<T, F>(entity: Entity, t: T, mut f: &mut F)
//where
//    F: FnMut(Entity, T),
//    T: Iterable<F>,
//{
//    t.apply(entity, &mut f);
//}

#[cfg(test)]
mod test {
    //use super::apply_to_func;
    use crate::core::c_types::*;
    use crate::core::component_registration::*;
    use crate::core::entity::Entity;
    use crate::core::filter::Filter;
    use crate::core::world::World;
    use flecs_ecs_bridge_derive::Component;
    use std::fmt::Display;
    use std::sync::OnceLock;

    #[derive(Debug, Default, Component, Clone)]
    struct Pos {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Default, Component, Clone)]
    struct Vel {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Default, Component, Clone)]
    struct Rot {
        pub x: f32,
        pub y: f32,
    }

    impl Display for Pos {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "(x: {}, y: {})", self.x, self.y)
        }
    }

    impl Display for Vel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "(x: {}, y: {})", self.x, self.y)
        }
    }

    impl Display for Rot {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "(x: {}, y: {})", self.x, self.y)
        }
    }

    #[test]
    fn test() {
        let world = World::default();
        let ee = world
            .new_entity()
            .add_component::<Pos>()
            .add_component::<Vel>();
        print!("test");
        //let mut filter = Filter::<(&mut Pos,), FnMut(Entity, (&mut Pos,))>::new(world.world);
        let mut filter = Filter::<(&mut Pos, &mut Vel), fn(_, _)>::new(world.world);
        //filter.each::<(&mut Pos, &mut Vel), _>(|e, (pos, vel)| {
        //    print!("xxxx");
        //    //println!("Pos: {}, Vel: {}", pos, vel);
        //});
        let entity = Entity::default();
        let mut pos = Pos::default();
        let mut vel = Vel::default();
        let mut rot = Rot::default();

        // apply_to_func(entity, &(&mut pos,), &mut |e, (a,)| {
        //     println!("One arg: {:?}", a.x);
        // });
        //
        // apply_to_func(entity, &(&mut pos, &mut vel), &mut |e, (a, b)| {
        //     println!("Two args: {} and {}", a, b);
        // });
        //
        // apply_to_func(
        //     entity,
        //     &(&mut pos, &mut vel, &mut rot),
        //     &mut |e, (a, b, c)| {
        //         println!("Three args: {}, {}, and {}", a, b, c);
        //     },
        // );
    }
}
