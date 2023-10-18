use super::c_binding::bindings::ecs_filter_desc_t;
use super::c_types::{IterT, OperKind, WorldT};
use super::component_registration::CachedComponentData;

use super::filter::Filterable;
use super::utility::functions::ecs_field;

//pub trait Iterable<'a>: Sized {
//    type TupleType: 'a;
//    type ArrayType: 'a;
//
//    fn populate(filter: &mut impl Filterable);
//    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t);
//    fn get_array_ptrs_of_components(it: &IterT);
//    fn get_tuple(array_components: &Self::ArrayType, index: usize);
//}

pub trait Iterable<'a>: Sized {
    type TupleType: 'a;
    type ArrayType: 'a;

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
impl<'a> Iterable<'a> for ()
{
    type TupleType = ();
    type ArrayType = [*mut u8; 0];

    fn populate(filter : &mut impl Filterable){}

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t){}

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        []
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType{}

}
#[rustfmt::skip]
impl<'a, A: 'a> Iterable<'a> for (A,)
where
    A: CachedComponentData,
{
    type TupleType = (&'a mut A,);
    type ArrayType = [*mut u8; 1];

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe {
            [ecs_field::<A>(it, 1) as *mut u8]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let ref_a = &mut (*array_a.add(index));
            (ref_a,)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a> Iterable<'a> for (Option<A>,)
where
    A: CachedComponentData,
{
    type TupleType = (Option<&'a mut A>,);
    type ArrayType = [*mut u8; 1];

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as i32; 
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe {
            [ecs_field::<A>(it, 1) as *mut u8]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;

            let option_a = if array_a.is_null() {
                None
            } else {
                Some(&mut (*array_a.add(index)))
            };

            (option_a,)
            
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a> Iterable<'a> for (A, B)
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
            [ecs_field::<A>(it,1) as *mut u8, 
            ecs_field::<B>(it,2) as *mut u8]
            }
    }
    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType
    {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = &mut (*array_a.add(index));
            let ref_b = &mut (*array_b.add(index));
            (ref_a, ref_b,)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a> Iterable<'a> for (A, Option<B>)
where
    A: CachedComponentData,
    B: CachedComponentData,
{
    type TupleType = (&'a mut A, Option<&'a mut B>);
    type ArrayType = [*mut u8; 2];

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe {
            [
                ecs_field::<A>(it, 1) as *mut u8,
                ecs_field::<B>(it, 2) as *mut u8,
            ]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = &mut (*array_a.add(index));

            let option_b = if array_b.is_null() {
                None
            } else {
                Some(&mut (*array_b.add(index)))
            };

            (ref_a, option_b)
        }
    }
}

impl<'a, A: 'a, B: 'a> Iterable<'a> for (Option<A>, Option<B>)
where
    A: CachedComponentData,
    B: CachedComponentData,
{
    type TupleType = (Option<&'a mut A>, Option<&'a mut B>);
    type ArrayType = [*mut u8; 2];

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();

        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();

        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as i32;

        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe {
            [
                ecs_field::<A>(it, 1) as *mut u8,
                ecs_field::<B>(it, 2) as *mut u8,
            ]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let option_a = if array_a.is_null() {
                None
            } else {
                Some(&mut (*array_a.add(index)))
            };

            let option_b = if array_b.is_null() {
                None
            } else {
                Some(&mut (*array_b.add(index)))
            };

            (option_a, option_b)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a, C: 'a> Iterable<'a> for (A,B,C)
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
            [ecs_field::<A>(it,1) as *mut u8,
                    ecs_field::<B>(it,2) as *mut u8,
                    ecs_field::<C>(it,3) as *mut u8]
            }
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
            (ref_a, ref_b, ref_c,)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a, C: 'a> Iterable<'a> for (A, B, Option<C>)
where
    A: CachedComponentData,
    B: CachedComponentData,
    C: CachedComponentData,
{
    type TupleType = (&'a mut A, &'a mut B, Option<&'a mut C>);
    type ArrayType = [*mut u8; 3];

    fn populate(filter : &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe { 
            [ecs_field::<A>(it, 1) as *mut u8, 
            ecs_field::<B>(it, 2) as *mut u8, 
            ecs_field::<C>(it, 3) as *mut u8]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = &mut (*array_a.add(index));
            let ref_b = &mut (*array_b.add(index));

            let option_c = if array_c.is_null() {
                None
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (ref_a, ref_b, option_c,)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a, C: 'a> Iterable<'a> for (A, Option<B>, Option<C>)
where
    A: CachedComponentData,
    B: CachedComponentData,
    C: CachedComponentData,
{
    type TupleType = (&'a mut A, Option<&'a mut B>, Option<&'a mut C>);
    type ArrayType = [*mut u8; 3];

    fn populate(filter : &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as i32;
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe { 
            [ecs_field::<A>(it, 1) as *mut u8, 
            ecs_field::<B>(it, 2) as *mut u8, 
            ecs_field::<C>(it, 3) as *mut u8]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = &mut (*array_a.add(index));

            let option_b = if array_b.is_null() {
                None
            } else {
                Some(&mut (*array_b.add(index)))
            };

            let option_c = if array_c.is_null() {
                None
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (ref_a, option_b, option_c)
        }
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a, C: 'a> Iterable<'a> for (Option<A>, Option<B>, Option<C>)
where
    A: CachedComponentData,
    B: CachedComponentData,
    C: CachedComponentData,
{
    type TupleType = (Option<&'a mut A>, Option<&'a mut B>, Option<&'a mut C>);
    type ArrayType = [*mut u8; 3];

    fn populate(filter : &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        term.oper = OperKind::Optional as i32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as i32;
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as i32;
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as i32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType {
        unsafe { 
            [ecs_field::<A>(it, 1) as *mut u8, 
            ecs_field::<B>(it, 2) as *mut u8, 
            ecs_field::<C>(it, 3) as *mut u8]
        }
    }

    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let option_a = if array_a.is_null() {
                None
            } else {
                Some(&mut (*array_a.add(index)))
            };

            let option_b = if array_b.is_null() {
                None
            } else {
                Some(&mut (*array_b.add(index)))
            };

            let option_c = if array_c.is_null() {
                None
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (option_a, option_b, option_c)
        }
    }
}