use super::c_binding::bindings::ecs_filter_desc_t;
use super::c_types::{IterT, OperKind, TermT, WorldT};
use super::component_registration::CachedComponentData;

use super::utility::functions::ecs_field;

pub trait Filterable: Sized {
    fn current_term(&mut self) -> &mut TermT;
    fn next_term(&mut self);
    fn get_world(&self) -> *mut WorldT;
}

pub trait Iterable<'a>: Sized {
    type TupleType: 'a;
    type ArrayType: 'a;

    fn populate(filter: &mut impl Filterable);
    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t);
    fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType;
    fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType;
}

/////////////////////
// first three tuple sizes are implemented manually for easier debugging and testing and understanding.
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

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

pub struct Wrapper<T>(T);

pub trait TupleForm<'a, T, U> {
    type Tuple;
    const IS_OPTION: bool;

    fn return_type_for_tuple(array: *mut U, index: usize) -> Self::Tuple;
}

impl<'a, T: 'a> TupleForm<'a, T, T> for Wrapper<T> {
    type Tuple = &'a mut T;
    const IS_OPTION: bool = false;

    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe { &mut (*array.add(index)) }
    }
}

impl<'a, T: 'a> TupleForm<'a, Option<T>, T> for Wrapper<T> {
    type Tuple = Option<&'a mut T>;

    const IS_OPTION: bool = true;

    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }
}

macro_rules! tuple_count {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),*) => { 1 + tuple_count!($($tail),*) };
}

macro_rules! impl_iterable {
    ($($t:ident: $tuple_t:ty),*) => {
        impl<'a, $($t: 'a + CachedComponentData),*> Iterable<'a> for ($($tuple_t,)*) {
            type TupleType = ($(
                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::Tuple
            ),*);

            type ArrayType = [*mut u8; tuple_count!($($t),*)];

            fn populate(filter: &mut impl Filterable) {
                let world = filter.get_world();
                $(
                    let term = filter.current_term();
                    term.id = <$t as CachedComponentData>::get_id(world);
                    if <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::IS_OPTION {
                        term.oper = OperKind::Optional as i32;
                    }
                    filter.next_term();
                )*
            }

            #[allow(unused)]
            fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
                let mut term_index = 0;
                $(
                    desc.terms[term_index].id = <$t as CachedComponentData>::get_id(world);
                    if <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::IS_OPTION {
                        desc.terms[term_index].oper = OperKind::Optional as i32;
                    }

                    term_index += 1;
                )*
            }
            #[allow(unused)]
            fn get_array_ptrs_of_components(it: &IterT) -> Self::ArrayType
            {
                let mut index = 1;
                unsafe {

                    [ $(
                        {
                            let ptr = ecs_field::<$t>(it, index) as *mut u8;
                            index += 1;
                            ptr
                        },
                    )* ]
                }
            }

            #[allow(unused)]
            fn get_tuple(array_components: &Self::ArrayType, index: usize) -> Self::TupleType {
                    let mut array_index = 0;
                    (
                        $(
                            {
                                let ptr = array_components[array_index] as *mut $t;
                                array_index += 1;
                                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::return_type_for_tuple(ptr,index)
                            },
                        )*
                    )
            }
        }
    };
}

impl_iterable!(A: A, B: B, C: C, D: D); //size 4
impl_iterable!(A: A, B: B, C: C, D: Option<D>); //size 4
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>); //size 4
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>); //size 4
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>); //size 4

impl_iterable!(A: A, B: B, C: C, D: D, E: E); //size 5
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>); //size 5
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>); //size 5
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>); //size 5
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>); //size 5
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>); //size 5

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F); //size 6
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>); //size 6
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>); //size 6
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>); //size 6
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>); //size 6
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>); //size 6
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>); //size 6

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G); //size 7
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>); //size 7
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>); //size 7
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>); //size 7
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>); //size 7
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>); //size 7
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>); //size 7
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>); //size 7

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H); //size 8
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: Option<H>); //size 8
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>); //size 8
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>); //size 8

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>); //size 9

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>); //size 10

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: K); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>); //size 11

impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: K, L: L); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: K, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: D, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: C, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: B, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: A, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
impl_iterable!(A: Option<A>, B: Option<B>, C: Option<C>, D: Option<D>, E: Option<E>, F: Option<F>, G: Option<G>, H: Option<H>, I: Option<I>, J: Option<J>, K: Option<K>, L: Option<L>); //size 12
