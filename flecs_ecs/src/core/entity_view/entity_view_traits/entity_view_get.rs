use crate::core::{EntityView, GetComponentPointers, GetTuple, WorldProvider};
use crate::sys;

use super::EntityId;

pub trait EntityViewGet<'w, Return>: EntityId + WorldProvider<'w> {
    /// gets mutable or immutable component(s) and/or relationship(s) from an entity in a callback and return a value.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// - `try_get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   If it does not, it will not run the callback.
    ///   If unsure and you still want to have the callback be ran, use `Option` wrapper instead.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// # Returns
    ///
    /// - If the callback was run, the return value of the callback wrapped in [`Some`]
    /// - Otherwise, returns [`None`]
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = entity.try_get::<&Position>(|(pos)| pos.x);
    /// assert_eq!(val, Some(10.0));
    ///
    /// let val = entity.try_get::<&Velocity>(|(vel)| vel.x);
    /// assert_eq!(val, None);
    ///
    /// let has_run = entity
    ///     .try_get::<(Option<&Velocity>, &Position)>(|(tag, pos)| {
    ///         assert_eq!(pos.x, 10.0);
    ///         assert!(tag.is_none());
    ///     })
    ///     .is_some();
    /// assert!(has_run);
    ///
    /// let has_run = entity
    ///     .try_get::<(&mut (Tag, Position), &Position)>(|(tag_pos_rel, pos)| {
    ///         assert_eq!(pos.x, 10.0);
    ///         assert_eq!(tag_pos_rel.x, 30.0);
    ///     })
    ///     .is_some();
    /// assert!(has_run);
    /// ```
    fn try_get<T: GetTuple>(
        self,
        callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return,
    ) -> Option<Return>;

    /// gets mutable or immutable component(s) and/or relationship(s) from an entity in a callback and return a value.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// - `get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_get` function instead.
    ///   `try_get` does not run the callback if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world
    ///     .entity()
    ///     .set(Position { x: 10.0, y: 20.0 })
    ///     .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = entity.get::<&Position>(|(pos)| pos.x);
    /// assert_eq!(val, 10.0);
    ///
    /// entity.get::<(Option<&Velocity>, &Position)>(|(vel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert!(vel.is_none());
    /// });
    ///
    /// entity.get::<(&mut (Tag, Position), &Position)>(|(tag_pos_rel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert_eq!(tag_pos_rel.x, 30.0);
    /// });
    /// ```
    fn get<T: GetTuple>(self, callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return) -> Return;
}

impl<'a, Return> EntityViewGet<'a, Return> for EntityView<'a> {
    fn try_get<T: GetTuple>(
        self,
        callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return,
    ) -> Option<Return> {
        let record = unsafe { sys::ecs_record_find(self.world_ptr(), *self.entity_id()) };

        if unsafe { (*record).table.is_null() } {
            return None;
        }

        let tuple_data = T::create_ptrs::<false>(self.world, self.id, record);
        let has_all_components = tuple_data.has_all_components();

        if has_all_components {
            let tuple = tuple_data.get_tuple();
            self.world.defer_begin();
            let ret = callback(tuple);
            self.world.defer_end();
            Some(ret)
        } else {
            None
        }
    }

    fn get<T: GetTuple>(self, callback: impl for<'e> FnOnce(T::TupleType<'e>) -> Return) -> Return {
        let record = unsafe { sys::ecs_record_find(self.world.world_ptr(), *self.id) };

        if unsafe { (*record).table.is_null() } {
            panic!("Entity does not have any components");
        }

        let tuple_data = T::create_ptrs::<true>(self.world, self.id, record);
        let tuple = tuple_data.get_tuple();

        self.world.defer_begin();
        let ret = callback(tuple);
        self.world.defer_end();
        ret
    }
}
