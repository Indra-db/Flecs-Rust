use std::ptr;

use super::c_binding::bindings::{ecs_field_w_size, ecs_filter_desc_t};
use super::c_types::{IterT, WorldT};
use super::component_registration::CachedComponentData;
use super::entity::Entity;
use super::filter::{Filter, Filterable};
use super::utility::functions::ecs_field;
use super::world::World;

pub trait Iterable: Sized {
    type TupleType;
    type ArrayType;

    fn populate(filter: &mut impl Filterable);

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t);

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType;

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType;
}

//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////
//these will be replaced with a macro later, for now I'm keeping it like this for easier testing / debugging / development
//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////
#[rustfmt::skip]
impl Iterable for ()
{
    type TupleType = ();
    type ArrayType = [*mut u8; 0];

    fn populate(filter : &mut impl Filterable){}

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t){}

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        return [];
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType
    {
        return ();
    }
    
}
#[rustfmt::skip]
impl<'a, A> Iterable for (&'a mut A,)
where
    A: CachedComponentData,
{
    type TupleType = (&'a mut A,);
    type ArrayType = [*mut u8; 1];

    fn populate(filter : &mut impl Filterable)
    {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t)
    {
        desc.terms[0].id = A::get_id(world);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType{
        unsafe { 
        return [ecs_field::<A>(it,1) as *mut u8];
        };
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType
    {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let ref_a = &mut (*array_a.add(index));
            return (ref_a,);
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B> Iterable for (&'a mut A, &'a mut B)
where
    A: CachedComponentData,
    B: CachedComponentData,
{
    type TupleType = (&'a mut A, &'a mut B);
    type ArrayType = [*mut u8; 2];

    fn populate(filter : &mut impl Filterable)
    {
        let world = filter.get_world();
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

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType{
        unsafe { 
            return [ecs_field::<A>(it,1) as *mut u8,
                    ecs_field::<B>(it,2) as *mut u8];
            };
    }
    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType
    {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = &mut (*array_a.add(index));
            let ref_b = &mut (*array_b.add(index));
            return (ref_a, ref_b,);
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C> Iterable for (&'a mut A, &'a mut B, &'a mut C)
where
    A: CachedComponentData,
    B: CachedComponentData,
    C: CachedComponentData,
{
    type TupleType = (&'a mut A, &'a mut B, &'a mut C);
    type ArrayType = [*mut u8; 3];

    fn populate(filter : &mut impl Filterable)
    {
        let world = filter.get_world();
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

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType{
        unsafe { 
            return [ecs_field::<A>(it,1) as *mut u8,
                    ecs_field::<B>(it,2) as *mut u8,
                    ecs_field::<C>(it,3) as *mut u8];
            };
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType
    {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = &mut (*array_a.add(index));
            let ref_b = &mut (*array_b.add(index));
            let ref_c = &mut (*array_c.add(index));
            return (ref_a, ref_b, ref_c,);
        }
    }
}
