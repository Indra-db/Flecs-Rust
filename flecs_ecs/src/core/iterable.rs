use super::c_binding::bindings::ecs_filter_desc_t;
use super::c_types::{IterT, OperKind, TermT, WorldT};
use super::component_registration::CachedComponentData;

use super::utility::functions::ecs_field;

pub trait Filterable: Sized {
    fn current_term(&mut self) -> &mut TermT;
    fn next_term(&mut self);
    fn get_world(&self) -> *mut WorldT;
}

pub struct ArrayElement {
    pub ptr: *mut u8,
    pub is_ref: bool,
}

pub struct ComponentsData<'a, T: Iterable<'a>> {
    pub array_components: T::ComponentsArray,
    pub is_ref_array_components: T::BoolArray,
    pub is_any_array_a_ref: bool,
}

pub trait Iterable<'a>: Sized {
    type TupleType: 'a;
    type ComponentsArray: 'a + std::ops::Index<usize, Output = *mut u8> + std::ops::IndexMut<usize>;
    type BoolArray: 'a + std::ops::Index<usize, Output = bool> + std::ops::IndexMut<usize>;
    type TupleSliceType: 'a;

    fn populate(filter: &mut impl Filterable);
    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t);
    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self>;

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType;

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType;

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType;

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType;
}

/////////////////////
// first three tuple sizes are implemented manually for easier debugging and testing and understanding.
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

#[rustfmt::skip]
impl<'a> Iterable<'a> for ()
{
    type TupleType = ();
    type ComponentsArray = [*mut u8; 0];
    type BoolArray = [bool; 0];
    type TupleSliceType = ();

    fn populate(_filter : &mut impl Filterable){}

    fn register_ids_descriptor(_world: *mut WorldT, _desc: &mut ecs_filter_desc_t){}

    fn get_array_ptrs_of_components(_it: &IterT) -> ComponentsData<'a, Self> {
        ComponentsData {
            array_components: [],
            is_ref_array_components: [],
            is_any_array_a_ref: false,
        }
    }

    fn get_tuple(_array_components: &Self::ComponentsArray, _index: usize) -> Self::TupleType{}

    fn get_tuple_with_ref(
        _array_components: &Self::ComponentsArray,
        _is_ref_array_components: &Self::BoolArray,
        _index: usize,
    ) -> Self::TupleType {}

    fn get_tuple_slices(
        _array_components: &Self::ComponentsArray,
        _count: usize,
    ) -> Self::TupleSliceType {}

    fn get_tuple_slices_with_ref(
        _array_components: &Self::ComponentsArray,
        _is_ref_array_components: &Self::BoolArray,
        _count: usize,
    ) -> Self::TupleSliceType {}

}

#[rustfmt::skip]
impl<'a, A: 'a> Iterable<'a> for (A,)
where
    A: CachedComponentData,
{
    type TupleType = (&'a mut A,);
    type ComponentsArray = [*mut u8; 1];
    type BoolArray = [bool; 1];
    type TupleSliceType = (&'a mut [A],);

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8]
        };
        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0]
        }} else { [false] };

        let is_any_array_a_ref = is_ref_array_components[0];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let ref_a = &mut (*array_a.add(index));
            (ref_a,)
        }
    }

    // TODO since it's only one component, we don't need to check if it's a ref array or not, we can just return the first element of the array
    // I think this is the case for all tuples of size 1
    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };
            (ref_a,)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let slice_a = std::slice::from_raw_parts_mut(array_a, count);
            (slice_a,)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let slice_a = if is_ref_array_components[0] {
                std::slice::from_raw_parts_mut(array_a, 1)
            } else {
                std::slice::from_raw_parts_mut(array_a, count)
            };
            (slice_a,)
        }
    
    }
}

#[rustfmt::skip]
impl<'a, A: 'a> Iterable<'a> for (Option<A>,)
where
    A: CachedComponentData,
{
    type TupleType = (Option<&'a mut A>,);
    type ComponentsArray = [*mut u8; 1];
    type BoolArray = [bool; 1];
    type TupleSliceType = (Option<&'a mut [A]>,);

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0]
        }} else { [false] };

        let is_any_array_a_ref = is_ref_array_components[0];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    /// TODO I think potentially I can determine if the component array is null before the loop as well to reduce the branching per iteration.
    /// potentially by working with two arrays with indices, one containing the indices of the component arrays that are null, and one containing the indices of the component arrays that are not null.
    /// reducing overall branching and increasing performance and conditions for vectorization
    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;

            let option_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                Some(&mut (*array_a.add(0)))
            } else {
                Some(&mut (*array_a.add(index)))
            };

            (option_a,)
        }

    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A; // array_components is an array [*mut u8; 3];
            
            let option_a = if array_a.is_null() {
                None
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
            };

            (option_a,)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A; // array_components is an array [*mut u8; 3];
            
            let option_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                let slice_a = std::slice::from_raw_parts_mut(array_a, 1);
                Some(slice_a)
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
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
    type ComponentsArray = [*mut u8; 2];
    type BoolArray = [bool; 2];
    type TupleSliceType = (&'a mut [A], &'a mut [B]);

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

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0]
        }} else { [false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType
    {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = &mut (*array_a.add(index));
            let ref_b = &mut (*array_b.add(index));
            (ref_a, ref_b,)
        }
    }

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };
            let ref_b = if is_ref_array_components[1] {
                &mut (*array_b.add(0))
            } else {
                &mut (*array_b.add(index))
            };
            (ref_a, ref_b,)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let slice_a = std::slice::from_raw_parts_mut(array_a, count);
            let slice_b = std::slice::from_raw_parts_mut(array_b, count);
            (slice_a, slice_b,)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let slice_a = if is_ref_array_components[0] {
                std::slice::from_raw_parts_mut(array_a, 1)
            } else {
                std::slice::from_raw_parts_mut(array_a, count)
            };
            let slice_b = if is_ref_array_components[1] {
                std::slice::from_raw_parts_mut(array_b, 1)
            } else {
                std::slice::from_raw_parts_mut(array_b, count)
            };
            (slice_a, slice_b,)
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
    type ComponentsArray = [*mut u8; 2];
    type BoolArray = [bool; 2];
    type TupleSliceType = (&'a mut [A], Option<&'a mut [B]>);

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0]
        }} else { [false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };

            let option_b = if is_ref_array_components[1] {
                Some(&mut (*array_b.add(0)))
            } else {
                Some(&mut (*array_b.add(index)))
            };

            (ref_a, option_b)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let slice_a = std::slice::from_raw_parts_mut(array_a, count);

            let slice_b = if array_b.is_null() {
                None
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            (slice_a, slice_b)
        }
    
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let slice_a = std::slice::from_raw_parts_mut(array_a, count);

            let slice_b = if is_ref_array_components[1] {
                let slice_b = std::slice::from_raw_parts_mut(array_b, 1);
                Some(slice_b)
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            (slice_a, slice_b)
        }
    }
}

impl<'a, A: 'a, B: 'a> Iterable<'a> for (Option<A>, Option<B>)
where
    A: CachedComponentData,
    B: CachedComponentData,
{
    type TupleType = (Option<&'a mut A>, Option<&'a mut B>);
    type ComponentsArray = [*mut u8; 2];
    type BoolArray = [bool; 2];
    type TupleSliceType = (Option<&'a mut [A]>, Option<&'a mut [B]>);

    fn populate(filter: &mut impl Filterable) {
        let world = filter.get_world();

        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();

        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as u32;

        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [
                ecs_field::<A>(it, 1) as *mut u8,
                ecs_field::<B>(it, 2) as *mut u8,
            ]
        };

        let is_ref_array_components = if !it.sources.is_null() {
            unsafe { [*it.sources.add(0) != 0, *it.sources.add(1) != 0] }
        } else {
            [false, false]
        };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let option_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                Some(&mut (*array_a.add(0)))
            } else {
                Some(&mut (*array_a.add(index)))
            };

            let option_b = if array_b.is_null() {
                None
            } else if is_ref_array_components[1] {
                Some(&mut (*array_b.add(0)))
            } else {
                Some(&mut (*array_b.add(index)))
            };

            (option_a, option_b)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let slice_a = if array_a.is_null() {
                None
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
            };

            let slice_b = if array_b.is_null() {
                None
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            (slice_a, slice_b)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;

            let slice_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                let slice_a = std::slice::from_raw_parts_mut(array_a, 1);
                Some(slice_a)
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
            };

            let slice_b = if array_b.is_null() {
                None
            } else if is_ref_array_components[1] {
                let slice_b = std::slice::from_raw_parts_mut(array_b, 1);
                Some(slice_b)
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            (slice_a, slice_b)
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
    type ComponentsArray = [*mut u8; 3];
    type BoolArray = [bool; 3];
    type TupleSliceType = (&'a mut [A], &'a mut [B], &'a mut [C]);

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

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self>{
       let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8,
            ecs_field::<C>(it, 3) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0,
            *it.sources.add(2) != 0]
        }} else { [false, false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1] || is_ref_array_components[2];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };
            let ref_b = if is_ref_array_components[1] {
                &mut (*array_b.add(0))
            } else {
                &mut (*array_b.add(index))
            };
            let ref_c = if is_ref_array_components[2] {
                &mut (*array_c.add(0))
            } else {
                &mut (*array_c.add(index))
            };
            (ref_a, ref_b, ref_c,)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let slice_a = std::slice::from_raw_parts_mut(array_a, count);
            let slice_b = std::slice::from_raw_parts_mut(array_b, count);
            let slice_c = std::slice::from_raw_parts_mut(array_c, count);
            (slice_a, slice_b, slice_c,)
        }    
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let slice_a = if is_ref_array_components[0] {
                std::slice::from_raw_parts_mut(array_a, 1)
            } else {
                std::slice::from_raw_parts_mut(array_a, count)
            };
            let slice_b = if is_ref_array_components[1] {
                std::slice::from_raw_parts_mut(array_b, 1)
            } else {
                std::slice::from_raw_parts_mut(array_b, count)
            };
            let slice_c = if is_ref_array_components[2] {
                std::slice::from_raw_parts_mut(array_c, 1)
            } else {
                std::slice::from_raw_parts_mut(array_c, count)
            };
            (slice_a, slice_b, slice_c,)
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
    type ComponentsArray = [*mut u8; 3];
    type BoolArray = [bool; 3];
    type TupleSliceType = (&'a mut [A], &'a mut [B], Option<&'a mut [C]>);

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
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8,
            ecs_field::<C>(it, 3) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0,
            *it.sources.add(2) != 0]
        }} else { [false, false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1] || is_ref_array_components[2];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };
            let ref_b = if is_ref_array_components[1] {
                &mut (*array_b.add(0))
            } else {
                &mut (*array_b.add(index))
            };

            let option_c = if is_ref_array_components[2] {
                Some(&mut (*array_c.add(0)))
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (ref_a, ref_b, option_c,)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = std::slice::from_raw_parts_mut(array_a, count);
            let slice_b = std::slice::from_raw_parts_mut(array_b, count);

            let slice_c = if array_c.is_null() {
                None
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c,)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = if is_ref_array_components[0] {
                std::slice::from_raw_parts_mut(array_a, 1)
            } else {
                std::slice::from_raw_parts_mut(array_a, count)
            };

            let slice_b = if is_ref_array_components[1] {
                std::slice::from_raw_parts_mut(array_b, 1)
            } else {
                std::slice::from_raw_parts_mut(array_b, count)
            };

            let slice_c = if array_c.is_null() {
                None
            } else if is_ref_array_components[2] {
                let slice_c = std::slice::from_raw_parts_mut(array_c, 1);
                Some(slice_c)
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c,)
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
    type ComponentsArray = [*mut u8; 3];
    type BoolArray = [bool; 3];
    type TupleSliceType = (&'a mut [A], Option<&'a mut [B]>, Option<&'a mut [C]>);

    fn populate(filter : &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as u32;
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8,
            ecs_field::<C>(it, 3) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0,
            *it.sources.add(2) != 0]
        }} else { [false, false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1] || is_ref_array_components[2];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;
            let ref_a = if is_ref_array_components[0] {
                &mut (*array_a.add(0))
            } else {
                &mut (*array_a.add(index))
            };

            let option_b = if is_ref_array_components[1] {
                Some(&mut (*array_b.add(0)))
            } else {
                Some(&mut (*array_b.add(index)))
            };

            let option_c = if is_ref_array_components[2] {
                Some(&mut (*array_c.add(0)))
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (ref_a, option_b, option_c)
        }

    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = std::slice::from_raw_parts_mut(array_a, count);

            let slice_b = if array_b.is_null() {
                None
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            let slice_c = if array_c.is_null() {
                None
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c)
        }
    
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = if is_ref_array_components[0] {
                std::slice::from_raw_parts_mut(array_a, 1)
            } else {
                std::slice::from_raw_parts_mut(array_a, count)
            };

            let slice_b = if array_b.is_null() {
                None
            } else if is_ref_array_components[1] {
                let slice_b = std::slice::from_raw_parts_mut(array_b, 1);
                Some(slice_b)
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            let slice_c = if array_c.is_null() {
                None
            } else if is_ref_array_components[2] {
                let slice_c = std::slice::from_raw_parts_mut(array_c, 1);
                Some(slice_c)
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c)
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
    type ComponentsArray = [*mut u8; 3];
    type BoolArray = [bool; 3];
    type TupleSliceType = (Option<&'a mut [A]>, Option<&'a mut [B]>, Option<&'a mut [C]>);

    fn populate(filter : &mut impl Filterable) {
        let world = filter.get_world();
        let term = filter.current_term();
        term.id = A::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
        let term = filter.current_term();
        term.id = B::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
        let term = filter.current_term();
        term.id = C::get_id(world);
        term.oper = OperKind::Optional as u32;
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = A::get_id(world);
        desc.terms[0].oper = OperKind::Optional as u32;
        desc.terms[1].id = B::get_id(world);
        desc.terms[1].oper = OperKind::Optional as u32;
        desc.terms[2].id = C::get_id(world);
        desc.terms[2].oper = OperKind::Optional as u32;
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A>(it, 1) as *mut u8,
            ecs_field::<B>(it, 2) as *mut u8,
            ecs_field::<C>(it, 3) as *mut u8]
        };

        let is_ref_array_components = if !it.sources.is_null() { unsafe {
            [*it.sources.add(0) != 0,
            *it.sources.add(1) != 0,
            *it.sources.add(2) != 0]
        }} else { [false, false, false] };

        let is_any_array_a_ref = is_ref_array_components[0] || is_ref_array_components[1] || is_ref_array_components[2];

        ComponentsData {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
        }
    }

    fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        unsafe {
            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let option_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                Some(&mut (*array_a.add(0)))
            } else {
                Some(&mut (*array_a.add(index)))
            };

            let option_b = if array_b.is_null() {
                None
            } else if is_ref_array_components[1] {
                Some(&mut (*array_b.add(0)))
            } else {
                Some(&mut (*array_b.add(index)))
            };

            let option_c = if array_c.is_null() {
                None
            } else if is_ref_array_components[2] {
                Some(&mut (*array_c.add(0)))
            } else {
                Some(&mut (*array_c.add(index)))
            };

            (option_a, option_b, option_c)
        }
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = if array_a.is_null() {
                None
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
            };

            let slice_b = if array_b.is_null() {
                None
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            let slice_c = if array_c.is_null() {
                None
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c)
        }
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        unsafe {

            let array_a = array_components[0] as *mut A;
            let array_b = array_components[1] as *mut B;
            let array_c = array_components[2] as *mut C;

            let slice_a = if array_a.is_null() {
                None
            } else if is_ref_array_components[0] {
                let slice_a = std::slice::from_raw_parts_mut(array_a, 1);
                Some(slice_a)
            } else {
                let slice_a = std::slice::from_raw_parts_mut(array_a, count);
                Some(slice_a)
            };

            let slice_b = if array_b.is_null() {
                None
            } else if is_ref_array_components[1] {
                let slice_b = std::slice::from_raw_parts_mut(array_b, 1);
                Some(slice_b)
            } else {
                let slice_b = std::slice::from_raw_parts_mut(array_b, count);
                Some(slice_b)
            };

            let slice_c = if array_c.is_null() {
                None
            } else if is_ref_array_components[2] {
                let slice_c = std::slice::from_raw_parts_mut(array_c, 1);
                Some(slice_c)
            } else {
                let slice_c = std::slice::from_raw_parts_mut(array_c, count);
                Some(slice_c)
            };

            (slice_a, slice_b, slice_c)
        }
    }
}
pub struct Wrapper<T>(T);

pub trait TupleForm<'a, T, U> {
    type Tuple;
    type TupleSlice;
    const IS_OPTION: bool;

    fn return_type_for_tuple(array: *mut U, index: usize) -> Self::Tuple;
    fn return_type_for_tuple_with_ref(array: *mut U, is_ref: bool, index: usize) -> Self::Tuple;
    fn return_type_for_tuple_slices(array: *mut U, count: usize) -> Self::TupleSlice;
    fn return_type_for_tuple_slices_with_ref(
        array: *mut U,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice;
}

impl<'a, T: 'a> TupleForm<'a, T, T> for Wrapper<T> {
    type Tuple = &'a mut T;
    type TupleSlice = &'a mut [T];
    const IS_OPTION: bool = false;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe { &mut (*array.add(index)) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if is_ref {
                &mut (*array.add(0))
            } else {
                &mut (*array.add(index))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices(array: *mut T, count: usize) -> Self::TupleSlice {
        unsafe { std::slice::from_raw_parts_mut(array, count) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices_with_ref(
        array: *mut T,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice {
        unsafe {
            if is_ref {
                std::slice::from_raw_parts_mut(array, 1)
            } else {
                std::slice::from_raw_parts_mut(array, count)
            }
        }
    }
}

impl<'a, T: 'a> TupleForm<'a, Option<T>, T> for Wrapper<T> {
    type Tuple = Option<&'a mut T>;
    type TupleSlice = Option<&'a mut [T]>;
    const IS_OPTION: bool = true;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else if is_ref {
                Some(&mut (*array.add(0)))
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices(array: *mut T, count: usize) -> Self::TupleSlice {
        unsafe {
            if array.is_null() {
                None
            } else {
                let slice = std::slice::from_raw_parts_mut(array, count);
                Some(slice)
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices_with_ref(
        array: *mut T,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice {
        unsafe {
            if array.is_null() {
                None
            } else if is_ref {
                let slice = std::slice::from_raw_parts_mut(array, 1);
                Some(slice)
            } else {
                let slice = std::slice::from_raw_parts_mut(array, count);
                Some(slice)
            }
        }
    }
}

macro_rules! tuple_count {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),*) => { 1 + tuple_count!($($tail),*) };
}

macro_rules! ignore {
    ($_:tt) => {};
}

macro_rules! impl_iterable {
    ($($t:ident: $tuple_t:ty),*) => {
        impl<'a, $($t: 'a + CachedComponentData),*> Iterable<'a> for ($($tuple_t,)*) {
            type TupleType = ($(
                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::Tuple
            ),*);

            type TupleSliceType = ($(
                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::TupleSlice
            ),*);
            type ComponentsArray = [*mut u8; tuple_count!($($t),*)];
            type BoolArray = [bool; tuple_count!($($t),*)];


            fn populate(filter: &mut impl Filterable) {
                let world = filter.get_world();
                $(
                    let term = filter.current_term();
                    term.id = <$t as CachedComponentData>::get_id(world);
                    if <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::IS_OPTION {
                        term.oper = OperKind::Optional as u32;
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
                        desc.terms[term_index].oper = OperKind::Optional as u32;
                    }

                    term_index += 1;
                )*
            }
            #[allow(unused)]
            fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self>
            {
                let mut index = 1;
                let mut index_ref = 0;
                let mut index_is_any_ref = 0;

                unsafe {
                    let array_components = [ $(
                        {
                            let ptr = ecs_field::<$t>(it, index) as *mut u8;
                            index += 1;
                            ptr
                        },
                    )* ];

                    let array_components2 = [ $(
                        {
                            let ptr = ecs_field::<$t>(it, index) as *mut u8;
                            index += 1;
                            ptr
                        },
                    )* ];

                    let is_ref_array_components = if !it.sources.is_null() { unsafe {
                        [ $(
                            {
                                ignore!($t);
                                let is_ref = *it.sources.add(index_ref) != 0;
                                index_ref += 1;
                                is_ref
                            },
                        )* ]
                    }} else {
                        [false; tuple_count!($($t),*)]
                    };

                    let is_any_array_a_ref = $(
                        {
                            ignore!($t);
                            let is_ref = is_ref_array_components[index_is_any_ref];
                            index_is_any_ref += 1;
                            is_ref
                        } ||
                    )* false;

                    ComponentsData {
                        array_components,
                        is_ref_array_components,
                        is_any_array_a_ref,
                    }
                }

                }


            #[allow(unused)]
            fn get_tuple(array_components: &Self::ComponentsArray, index: usize) -> Self::TupleType {
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

            #[allow(unused)]
            fn get_tuple_with_ref(array_components: &Self::ComponentsArray, is_ref_array_components: &Self::BoolArray, index: usize) -> Self::TupleType {
                    let mut array_index = 0;
                    (
                        $(
                            {
                                let ptr = array_components[array_index] as *mut $t;
                                let is_ref = is_ref_array_components[array_index];
                                array_index += 1;
                                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::return_type_for_tuple_with_ref(ptr, is_ref, index)
                            },
                        )*
                    )
            }

            #[allow(unused)]
            fn get_tuple_slices(
                array_components: &Self::ComponentsArray,
                count: usize,
            ) -> Self::TupleSliceType {
                    let mut array_index = 0;
                    (
                        $(
                            {
                                let ptr = array_components[array_index] as *mut $t;
                                array_index += 1;
                                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::return_type_for_tuple_slices(ptr, count)
                            },
                        )*
                    )
            }

            #[allow(unused)]
            fn get_tuple_slices_with_ref(
                array_components: &Self::ComponentsArray,
                is_ref_array_components: &Self::BoolArray,
                count: usize,
            ) -> Self::TupleSliceType {
                    let mut array_index = 0;
                    (
                        $(
                            {
                                let ptr = array_components[array_index] as *mut $t;
                                let is_ref = is_ref_array_components[array_index];
                                array_index += 1;
                                <Wrapper::<$t> as TupleForm<'a, $tuple_t, $t>>::return_type_for_tuple_slices_with_ref(ptr, is_ref, count)
                            },
                        )*
                    )
            }
        }
    }
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
