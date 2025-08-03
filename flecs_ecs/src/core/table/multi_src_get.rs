use flecs_ecs::prelude::*;

use super::iter::FieldError;

impl<const IS_RUN: bool, P> TableIter<'_, IS_RUN, P>
where
    P: ComponentId,
{
    pub fn get<'w, T: ComponentId, F>(&'w self, fields_indices: F) -> F::TupleType<'w>
    where
        F: FieldsTuple<T> + 'w,
    {
        let world = self.world();
        fields_indices.get_tuple(&world, self)
    }
}

struct Mut(usize);

pub trait FieldsTuple<T: ComponentId>: Sized {
    type TupleType<'w>
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a>;
}

pub trait IterableTypeFieldOperation<T: 'static + ComponentId> {
    type ActualType<'w>
    where
        Self: 'w;

    fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::ActualType<'a>
    where
        Self: 'a;
}

impl<T: 'static + ComponentId> IterableTypeFieldOperation<T> for usize {
    type ActualType<'w>
        = Result<Field<'w, T::UnderlyingType, true>, FieldError>
    where
        Self: 'w;

    fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::ActualType<'a>
    where
        Self: 'a,
    {
        iter.field_result::<T>(self as i8)
    }
}

impl<T: 'static + ComponentId> IterableTypeFieldOperation<T> for Mut {
    type ActualType<'w>
        = Result<FieldMut<'w, T::UnderlyingType, true>, FieldError>
    where
        Self: 'w;

    fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::ActualType<'a>
    where
        Self: 'a,
    {
        iter.field_result_mut::<T>(self.0 as i8)
    }
}

// macro_rules! impl_fields_tuple {
//     ($($t:ident),*) => {
//         //impl<$($t: IterableTypeFieldOperation),*> FieldsTuple for ($($t,)*) {
//         impl<T: 'static + ComponentId, $($t),*> FieldsTuple<T> for ($($t,)*)
//         where

//             $($t: IterableTypeFieldOperation<T>),*
//         {
//             type TupleType<'w> = ($(
//                 < $t as IterableTypeFieldOperation<T>>::ActualType<'w>,
//             )*)
//             where
//                 Self: 'w;

//             fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
//                 self,
//                 _world: &WorldRef,
//                 iter: &'a TableIter<'a, IS_RUN, P>,
//             ) -> Self::TupleType<'a> {
//                 //idk
//             }
//         }
//     }
// }

impl<A, B, T> FieldsTuple<T> for (A, B)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (self.0.get_data(iter), self.1.get_data(iter))
    }
}

//for A B C
impl<A, B, C, T> FieldsTuple<T> for (A, B, C)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
        )
    }
}
impl<A, B, C, D, T> FieldsTuple<T> for (A, B, C, D)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
    D: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <D as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
            self.3.get_data(iter),
        )
    }
}
impl<A, B, C, D, E, T> FieldsTuple<T> for (A, B, C, D, E)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
    D: IterableTypeFieldOperation<T>,
    E: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <D as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <E as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
            self.3.get_data(iter),
            self.4.get_data(iter),
        )
    }
}
impl<A, B, C, D, E, F, T> FieldsTuple<T> for (A, B, C, D, E, F)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
    D: IterableTypeFieldOperation<T>,
    E: IterableTypeFieldOperation<T>,
    F: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <D as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <E as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <F as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
            self.3.get_data(iter),
            self.4.get_data(iter),
            self.5.get_data(iter),
        )
    }
}
impl<A, B, C, D, E, F, G, T> FieldsTuple<T> for (A, B, C, D, E, F, G)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
    D: IterableTypeFieldOperation<T>,
    E: IterableTypeFieldOperation<T>,
    F: IterableTypeFieldOperation<T>,
    G: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <D as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <E as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <F as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <G as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
            self.3.get_data(iter),
            self.4.get_data(iter),
            self.5.get_data(iter),
            self.6.get_data(iter),
        )
    }
}
impl<A, B, C, D, E, F, G, H, T> FieldsTuple<T> for (A, B, C, D, E, F, G, H)
where
    T: 'static + ComponentId,
    A: IterableTypeFieldOperation<T>,
    B: IterableTypeFieldOperation<T>,
    C: IterableTypeFieldOperation<T>,
    D: IterableTypeFieldOperation<T>,
    E: IterableTypeFieldOperation<T>,
    F: IterableTypeFieldOperation<T>,
    G: IterableTypeFieldOperation<T>,
    H: IterableTypeFieldOperation<T>,
{
    type TupleType<'w>
        = (
        <A as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <B as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <C as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <D as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <E as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <F as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <G as IterableTypeFieldOperation<T>>::ActualType<'w>,
        <H as IterableTypeFieldOperation<T>>::ActualType<'w>,
    )
    where
        Self: 'w;

    fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
        self,
        _world: &WorldRef,
        iter: &'a TableIter<'a, IS_RUN, P>,
    ) -> Self::TupleType<'a> {
        (
            self.0.get_data(iter),
            self.1.get_data(iter),
            self.2.get_data(iter),
            self.3.get_data(iter),
            self.4.get_data(iter),
            self.5.get_data(iter),
            self.6.get_data(iter),
            self.7.get_data(iter),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Debug)]
    struct TP {
        x: i32,
    }

    #[derive(Component, Debug)]
    struct Relx;

    #[test]
    #[should_panic]
    fn multi_src_same_table_err() {
        let world = World::new();

        let parent = world.entity().set(TP { x: 1 });
        parent.add((id::<Relx>(), parent));

        world.entity().set(TP { x: 4 }).add((id::<Relx>(), parent));

        world
            .query::<(&mut TP, &TP)>()
            .with((id::<Relx>(), "$parent"))
            .term_at(1)
            .set_src("$parent")
            .build()
            .run(|mut it| {
                while it.next() {
                    match it.get::<TP, _>((Mut(0), 1)) {
                        (Ok(_), Ok(_)) => {}
                        (Ok(_), Err(_)) => {
                            panic!();
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            });
    }

    #[test]
    fn multi_src_diff_table_ok() {
        let world = World::new();

        let parent = world.entity().set(TP { x: 1 });
        world.entity().set(TP { x: 4 }).child_of(parent);

        let mut ok = false;

        world
            .query::<(&mut TP, &TP)>()
            .term_at(1)
            .parent()
            .build()
            .run(|mut it| {
                while it.next() {
                    match it.get::<TP, _>((Mut(0), 1)) {
                        (Ok(_), Ok(_)) => {
                            ok = true;
                        }
                        (Ok(_), Err(_)) => {}
                        _ => {
                            unreachable!();
                        }
                    }
                }
            });

        assert!(ok, "Expected to get a valid result");
    }
}
