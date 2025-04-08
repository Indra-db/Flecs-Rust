use std::mem::MaybeUninit;

use crate::core::fields_tuple::FieldsTuple;

use super::*;
pub struct Fields<'a, T: ComponentId, const NI: usize, const NM: usize, const Total: usize> {
    pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
    mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
}

impl<'a, T: ComponentId, const NI: usize, const NM: usize, const Total: usize>
    Fields<'a, T, NI, NM, Total>
{
    unsafe fn to_initialized_array<Field, const NR: usize>(
        array: [MaybeUninit<Field>; NR],
    ) -> [Field; NR] {
        unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
    }

    pub fn new(iter: &'a TableIter, immut_fields: [usize; NI], mut_fields: [usize; NM]) -> Self {
        let mut fields_mut_array: [MaybeUninit<FieldMut<'_, T::UnderlyingType, false>>; NM] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..NM {
            fields_mut_array[i] = MaybeUninit::new(
                iter.field_mut_lockless::<T>(mut_fields[i] as i8)
                    .expect("Field is not present or not correct type"),
            );
        }

        let mut_fields = unsafe { Self::to_initialized_array(fields_mut_array) };

        let mut fields_immut_array: [MaybeUninit<Field<'_, T::UnderlyingType, false>>; NI] =
            unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..NI {
            fields_immut_array[i] = MaybeUninit::new(
                iter.field_lockless::<T>(immut_fields[i] as i8)
                    .expect("Field is not present or not correct type"),
            );
        }

        let immut_fields = unsafe { Self::to_initialized_array(fields_immut_array) };

        Self {
            immut_fields,
            mut_fields,
        }
    }

    pub fn get<'f, F: FieldsTuple, Func: FnMut(F::TupleType<'f>)>(
        &self,
        fields: F,
        mut func: Func,
    ) {
        // let tuple = F::create_tuple();
        //func(tuple);
    }
}

#[crabtime::expression]
fn fields(typename: String, fields_immut: Vec<usize>, fields_mut: Vec<usize>) {
    let fields_immut_str = fields_immut
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let fields_mut_str = fields_mut.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let size_immut = fields_immut.len();
    let size_mut = fields_mut.len();
    let size_total = size_immut + size_mut;

    let arr_immut = fields_immut_str[0..size_immut].join(",");
    let arr_mut = fields_mut_str[0..size_mut].join(",");
    crabtime::output! {
       Fields::<{{typename}}, {{size_immut}}, {{size_mut}}, {{size_total}}>::new([{{arr_immut}}], [{{arr_mut}}])
    }
}
