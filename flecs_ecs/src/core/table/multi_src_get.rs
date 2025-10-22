//! Multi-source field access for table iterators.
//!
//! This module provides the [`TableIter::get()`] method which enables safe access to
//! fields from different sources (different tables), handling mutable aliasing checks
//! when the `flecs_safety_locks` feature is enabled.
//!
//! # Mutable Access
//!
//! To get mutable access to a field, wrap its index with `Mut()`:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # use flecs_ecs::core::table::Mut;
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # #[derive(Component)]
//! # struct Parent;
//! # let world = World::new();
//! # let parent = world.entity().set(Position { x: 0.0, y: 0.0 });
//! # world.entity().set(Position { x: 1.0, y: 1.0 }).child_of(parent);
//! # let query = world.query::<(&mut Position, &Position)>().term_at(1).parent().build();
//! query.run(|mut it| {
//!     while it.next() {
//!         // Get mutable field 0 and immutable field 1
//!         // Uses Mut(0) to indicate field 0 is mutable
//!         match it.get::<Position, _>((Mut(0), 1)) {
//!             (Ok(mut pos), Ok(parent_pos)) => {
//!                 for i in it.iter() {
//!                     pos[i].x = parent_pos[i].x;
//!                     pos[i].y = parent_pos[i].y;
//!                 }
//!             }
//!             _ => {}
//!         }
//!     }
//! });
//! ```
//!
//! when aliasing is possible, this can be handled with the (Ok(_), Err(_)) match arm.
//!
//! ```rust
//! # #[derive(Component, Debug)]
//! # struct TP {
//! #     x: i32,
//! # }
//! # #[derive(Component, Debug)]
//! # struct Relx;
//! # #[derive(Component, Debug)]
//! # struct Unit;
//! # let world = World::new();
//! let parent = world.entity().set(TP { x: 1 });
//! parent.add((id::<Relx>(), parent));
//! world
//!     .entity()
//!     .add(Unit)
//!     .set(TP { x: 4 })
//!     .add((id::<Relx>(), parent));
//! let mut ok_alias = false;
//! let mut ok_normal = false;
//! world
//!     .query::<(&mut TP, &TP)>()
//!     .with((id::<Relx>(), "$parent"))
//!     .term_at(1)
//!     .set_src("$parent")
//!     .build()
//!     .run(|mut it| {
//!         while it.next() {
//!             match it.get::<TP, _>((Mut(0), 1)) {
//!                 (Ok(_tp1), Ok(_tp2)) => {
//!                     // table no alias issues
//!                     ok_normal = true;
//!                 }
//!                 (Ok(_tp), Err(_)) => {
//!                     // aliasing detected, only one reference allowed & returned
//!                     ok_alias = true;
//!                 }
//!                 _ => {
//!                     unreachable!();
//!                 }
//!             }
//!         }
//!     });//!
//! assert!(ok_alias, "Expected to detect aliasing and return an error");
//! assert!(
//!     ok_normal,
//!     "Expected to get a valid result without aliasing error"
//! );
//! ```
//!
//! # Aliasing Detection
//!
//! When `flecs_safety_locks` is enabled, attempting to get mutable and immutable
//! access to the same table will return an error, preventing undefined behavior
//! from mutable aliasing.

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

/// Wrapper to indicate that a field should be accessed mutably.
///
/// Used with [`TableIter::get()`] to specify which fields require mutable access.
pub struct Mut(pub usize);

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

    #[derive(Component, Debug)]
    struct Unit;

    #[test]
    #[cfg(feature = "flecs_safety_locks")]
    fn multi_src_same_table_err() {
        let world = World::new();
        let parent = world.entity().set(TP { x: 1 });
        parent.add((id::<Relx>(), parent));
        world
            .entity()
            .add(Unit)
            .set(TP { x: 4 })
            .add((id::<Relx>(), parent));
        let mut ok_alias = false;
        let mut ok_normal = false;
        world
            .query::<(&mut TP, &TP)>()
            .with((id::<Relx>(), "$parent"))
            .term_at(1)
            .set_src("$parent")
            .build()
            .run(|mut it| {
                while it.next() {
                    match it.get::<TP, _>((Mut(0), 1)) {
                        (Ok(_tp1), Ok(_tp2)) => {
                            // table no alias issues
                            ok_normal = true;
                        }
                        (Ok(_tp), Err(_)) => {
                            // aliasing detected, only one reference allowed & returned
                            ok_alias = true;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            });

        assert!(ok_alias, "Expected to detect aliasing and return an error");
        assert!(
            ok_normal,
            "Expected to get a valid result without aliasing error"
        );
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
