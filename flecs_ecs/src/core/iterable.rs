use crate::sys::{self, ecs_filter_desc_t, ecs_inout_kind_t, ecs_oper_kind_t};

use super::{
    c_types::{IterT, OperKind, TermT},
    component_registration::CachedComponentData,
    ecs_field, FilterBuilderImpl, InOutKind, WorldT,
};

pub trait Filterable: Sized + FilterBuilderImpl {
    fn current_term(&mut self) -> &mut TermT;
    fn next_term(&mut self);
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

struct Singleton<T>(T);

pub trait IterableTypeOperation {
    type CastType;
    type ActualType;
    type SliceType;
    type OnlyType: CachedComponentData;

    fn populate_term(term: &mut sys::ecs_term_t);
    fn get_tuple_data(array_components_data: *mut u8, index: usize) -> Self::ActualType;
    fn get_tuple_with_ref_data(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType;
    fn get_tuple_slice_data(array_components_data: *mut u8, count: usize) -> Self::SliceType;
    fn get_tuple_slices_with_ref_data(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType;
}

impl<'a, T> IterableTypeOperation for &'a T
where
    T: CachedComponentData,
{
    type CastType = *const T;
    type ActualType = &'a T;
    type SliceType = &'a [T];
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as ecs_inout_kind_t;
    }

    fn get_tuple_data(array_components_data: *mut u8, index: usize) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &*data_ptr.add(index) }
    }

    fn get_tuple_with_ref_data(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &*data_ptr.add(0)
            } else {
                &*data_ptr.add(index)
            }
        }
    }

    fn get_tuple_slice_data(array_components_data: *mut u8, count: usize) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { std::slice::from_raw_parts(data_ptr, count) }
    }

    fn get_tuple_slices_with_ref_data(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref_array_components {
                std::slice::from_raw_parts(data_ptr, 1)
            } else {
                std::slice::from_raw_parts(data_ptr, count)
            }
        }
    }
}

struct Const<T>(T);

impl<'a, T> IterableTypeOperation for &'a mut T
where
    T: CachedComponentData,
{
    type CastType = *mut T;
    type ActualType = &'a mut T;
    type SliceType = &'a mut [T];
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as ecs_inout_kind_t;
    }

    fn get_tuple_data(array_components_data: *mut u8, index: usize) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &mut *data_ptr.add(index) }
    }

    fn get_tuple_with_ref_data(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &mut *data_ptr.add(0)
            } else {
                &mut *data_ptr.add(index)
            }
        }
    }

    fn get_tuple_slice_data(array_components_data: *mut u8, count: usize) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { std::slice::from_raw_parts_mut(data_ptr, count) }
    }

    fn get_tuple_slices_with_ref_data(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref_array_components {
                std::slice::from_raw_parts_mut(data_ptr, 1)
            } else {
                std::slice::from_raw_parts_mut(data_ptr, count)
            }
        }
    }
}

impl<'a, T> IterableTypeOperation for Option<&'a T>
where
    T: CachedComponentData,
{
    type CastType = *const T;
    type ActualType = Option<&'a T>;
    type SliceType = Option<&'a [T]>;
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as ecs_inout_kind_t;
        term.oper = OperKind::Optional as ecs_oper_kind_t;
    }

    fn get_tuple_data(array_components_data: *mut u8, index: usize) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }

    fn get_tuple_with_ref_data(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &*data_ptr.add(0) })
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }

    fn get_tuple_slice_data(array_components_data: *mut u8, count: usize) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, count) })
        }
    }

    fn get_tuple_slices_with_ref_data(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref_array_components {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, 1) })
        } else {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, count) })
        }
    }
}

impl<'a, T> IterableTypeOperation for Option<&'a mut T>
where
    T: CachedComponentData,
{
    type CastType = *mut T;
    type ActualType = Option<&'a mut T>;
    type SliceType = Option<&'a mut [T]>;
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as ecs_inout_kind_t;
        term.oper = OperKind::Optional as ecs_oper_kind_t;
    }

    fn get_tuple_data(array_components_data: *mut u8, index: usize) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }

    fn get_tuple_with_ref_data(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &mut *data_ptr.add(0) })
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }

    fn get_tuple_slice_data(array_components_data: *mut u8, count: usize) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, count) })
        }
    }

    fn get_tuple_slices_with_ref_data(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref_array_components {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, 1) })
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, count) })
        }
    }
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

    fn register_ids_descriptor(_world: *mut WorldT,_desc: &mut ecs_filter_desc_t){}

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
    A: IterableTypeOperation,
{
    type TupleType = (A::ActualType,);
    type ComponentsArray = [*mut u8; 1];
    type BoolArray = [bool; 1];
    type TupleSliceType = (A::SliceType,);

    fn populate(filter: &mut impl Filterable) {

        let world = filter.get_world();
        filter.term_with_id(A::OnlyType::get_id(world));
        let term = filter.current_term();
        A::populate_term(term);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        let term = &mut desc.terms[0];
        term.id = A::OnlyType::get_id(world);
        A::populate_term(term);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A::OnlyType>(it, 1) as *mut u8]
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
            (A::get_tuple_data(array_components[0], index),)
    }

    // TODO since it's only one component, we don't need to check if it's a ref array or not, we can just return the first element of the array
    // I think this is the case for all tuples of size 1
    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        (A::get_tuple_with_ref_data(array_components[0], is_ref_array_components[0], index),)
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slice_data(array_components[0], count),)
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slices_with_ref_data(array_components[0], is_ref_array_components[0], count),)
    
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a> Iterable<'a> for (A, B)
where
    A: IterableTypeOperation,
    B: IterableTypeOperation,
{
    type TupleType = (A::ActualType, B::ActualType);
    type ComponentsArray = [*mut u8; 2];
    type BoolArray = [bool; 2];
    type TupleSliceType = (A::SliceType, B::SliceType);

    fn populate(filter : &mut impl Filterable)
    {
        let world = filter.get_world();
         filter.term_with_id(A::OnlyType::get_id(world));
        let term = filter.current_term();
        A::populate_term(term);
        filter.next_term();
         filter.term_with_id(B::OnlyType::get_id(world));
        let term = filter.current_term(); 
        B::populate_term(term);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT,desc: &mut ecs_filter_desc_t)
    {
        let term = &mut desc.terms[0];
        term.id = A::OnlyType::get_id(world);
        A::populate_term(term);
        let term = &mut desc.terms[1];
        term.id = B::OnlyType::get_id(world);
        B::populate_term(term);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self> {
        let array_components = unsafe {
            [ecs_field::<A::OnlyType>(it, 1) as *mut u8,
            ecs_field::<B::OnlyType>(it, 2) as *mut u8]
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
        (A::get_tuple_data(array_components[0], index),B::get_tuple_data(array_components[1], index),)
    }

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        (A::get_tuple_with_ref_data(array_components[0], is_ref_array_components[0], index),B::get_tuple_with_ref_data(array_components[1], is_ref_array_components[1], index),)
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slice_data(array_components[0], count),B::get_tuple_slice_data(array_components[1], count),)
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slices_with_ref_data(array_components[0], is_ref_array_components[0], count),B::get_tuple_slices_with_ref_data(array_components[1], is_ref_array_components[1], count),)
    }
}

#[rustfmt::skip]
impl<'a, A: 'a, B: 'a, C: 'a> Iterable<'a> for (A,B,C)
where
    A: IterableTypeOperation,
    B: IterableTypeOperation,
    C: IterableTypeOperation,
{
    type TupleType = (A::ActualType, B::ActualType, C::ActualType);
    type ComponentsArray = [*mut u8; 3];
    type BoolArray = [bool; 3];
    type TupleSliceType = (A::SliceType, B::SliceType, C::SliceType);

    fn populate(filter : &mut impl Filterable)
    {
        let world = filter.get_world();
        filter.term_with_id(A::OnlyType::get_id(world));
        let term = filter.current_term();
        A::populate_term(term);
        filter.next_term();
        unsafe { filter.term_with_id(B::OnlyType::get_id_unchecked()) } ;
        let term = filter.current_term();
        B::populate_term(term);
        filter.next_term();
        unsafe { filter.term_with_id(C::OnlyType::get_id_unchecked()) } ;
        let term = filter.current_term();
        C::populate_term(term);
        filter.next_term();
    }

    fn register_ids_descriptor(world: *mut WorldT,desc: &mut ecs_filter_desc_t)
    {
        let term = &mut desc.terms[0];
        term.id = A::OnlyType::get_id(world);
        A::populate_term(term);
        let term = &mut desc.terms[1];
        term.id = B::OnlyType::get_id(world);
        B::populate_term(term);
        let term = &mut desc.terms[2];
        term.id = C::OnlyType::get_id(world);
        C::populate_term(term);
    }

    fn get_array_ptrs_of_components(it: &IterT) -> ComponentsData<'a, Self>{
       let array_components = unsafe {
            [ecs_field::<A::OnlyType>(it, 1) as *mut u8,
            ecs_field::<B::OnlyType>(it, 2) as *mut u8,
            ecs_field::<C::OnlyType>(it, 3) as *mut u8]
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
        (A::get_tuple_data(array_components[0], index),B::get_tuple_data(array_components[1], index),C::get_tuple_data(array_components[2], index),)
    }

    fn get_tuple_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        index: usize,
    ) -> Self::TupleType {
        (A::get_tuple_with_ref_data(array_components[0], is_ref_array_components[0], index),B::get_tuple_with_ref_data(array_components[1], is_ref_array_components[1], index),C::get_tuple_with_ref_data(array_components[2], is_ref_array_components[2], index),)
    }

    fn get_tuple_slices(
        array_components: &Self::ComponentsArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slice_data(array_components[0], count),B::get_tuple_slice_data(array_components[1], count),C::get_tuple_slice_data(array_components[2], count),)
    }

    fn get_tuple_slices_with_ref(
        array_components: &Self::ComponentsArray,
        is_ref_array_components: &Self::BoolArray,
        count: usize,
    ) -> Self::TupleSliceType {
        (A::get_tuple_slices_with_ref_data(array_components[0], is_ref_array_components[0], count),B::get_tuple_slices_with_ref_data(array_components[1], is_ref_array_components[1], count),C::get_tuple_slices_with_ref_data(array_components[2], is_ref_array_components[2], count),)
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
        impl<'a, $($t: 'a + IterableTypeOperation),*> Iterable<'a> for ($($tuple_t,)*) {
            type TupleType = ($(
                $t::ActualType
            ),*);

            type TupleSliceType = ($(
                $t::SliceType
            ),*);
            type ComponentsArray = [*mut u8; tuple_count!($($t),*)];
            type BoolArray = [bool; tuple_count!($($t),*)];


            fn populate(filter: &mut impl Filterable) {
                let world = filter.get_world();
                $(
                    filter.term_with_id($t::OnlyType::get_id(world));
                    let term = filter.current_term();
                    $t::populate_term(term);
                    filter.next_term();
                )*
            }

            #[allow(unused)]
            fn register_ids_descriptor(world: *mut WorldT,desc: &mut ecs_filter_desc_t) {
                let mut term_index = 0;
                $(
                    let term = &mut desc.terms[term_index];
                    term.id = $t::OnlyType::get_id(world);
                    $t::populate_term(term);
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
                            let ptr = ecs_field::<$t::OnlyType>(it, index) as *mut u8;
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
                    let mut array_index = -1;
                    (
                        $(
                            {
                                array_index += 1;
                                $t::get_tuple_data(array_components[array_index as usize] /*as *mut $t*/, index)
                            },
                        )*
                    )
            }

            #[allow(unused)]
            fn get_tuple_with_ref(array_components: &Self::ComponentsArray, is_ref_array_components: &Self::BoolArray, index: usize) -> Self::TupleType {
                    let mut array_index = -1;
                    (
                        $(
                            {
                                array_index += 1;
                                $t::get_tuple_with_ref_data(array_components[array_index as usize] /*as *mut $t*/, is_ref_array_components[array_index as usize], index)
                            },
                        )*
                    )
            }

            #[allow(unused)]
            fn get_tuple_slices(
                array_components: &Self::ComponentsArray,
                count: usize,
            ) -> Self::TupleSliceType {
                    let mut array_index = -1;
                    (
                        $(
                            {
                                array_index += 1;
                                $t::get_tuple_slice_data(array_components[array_index as usize], count)
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
                    let mut array_index = -1;
                    (
                        $(
                            {
                                array_index += 1;
                                $t::get_tuple_slices_with_ref_data(array_components[array_index as usize], is_ref_array_components[array_index as usize], count)
                            },
                        )*
                    )
            }
        }
    }
}

impl_iterable!(A: A, B: B, C: C, D: D); //size 4
impl_iterable!(A: A, B: B, C: C, D: D, E: E); //size 5
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F); //size 6
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G); //size 7
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H); //size 8
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I); //size 9
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J); //size 10
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: K); //size 11
impl_iterable!(A: A, B: B, C: C, D: D, E: E, F: F, G: G, H: H, I: I, J: J, K: K, L: L); //size 12
