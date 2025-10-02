use super::*;

/// Query mixin implementation
impl World {
    /// Create a new uncached [`Query`].
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # See also
    ///
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    /// * [`World::query_named()`]
    pub fn new_query<Components>(&self) -> Query<Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new(self).build()
    }

    /// Create a new named [`Query`].
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query.
    ///
    /// # See also
    ///
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    /// * [`World::query_named()`]
    pub fn new_query_named<Components>(&self, name: &str) -> Query<Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_named(self, name).build()
    }

    /// Create a new [`QueryBuilder`].
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query_named()`]
    pub fn query<Components>(&self) -> QueryBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new(self)
    }

    /// Create a new named [`QueryBuilder`].
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    pub fn query_named<'a, Components>(&'a self, name: &str) -> QueryBuilder<'a, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_named(self, name)
    }

    /// Create a query from a query description.
    ///
    /// # Safety
    ///
    /// Caller needs to ensure the query type is correct of the provided `desc`.
    pub unsafe fn query_from_desc<Components>(
        &self,
        desc: &mut sys::ecs_query_desc_t,
    ) -> QueryBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_from_desc(self, desc)
    }

    /// Attempts to convert an entity into a query.
    ///
    /// Returns the untyped query if the entity is alive and valid; otherwise, returns `None`.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # See Also
    ///
    /// * [`World::query_from`]
    pub fn try_query_from(&self, query_entity: impl Into<Entity>) -> Option<Query<()>> {
        Query::<()>::new_from_entity(self, query_entity)
    }

    /// Converts an entity into a query, automatically unwrapping the result.
    ///
    /// This method panics if the entity is not alive or not a valid query.
    /// For safer usage, consider using [`World::try_query_from`].
    ///
    /// # Panics
    ///
    /// Panics if the entity is not alive or not a valid query.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # See Also
    ///
    /// * [`World::try_query_from`]
    pub fn query_from(&self, query_entity: impl Into<Entity>) -> Query<()> {
        self.try_query_from(query_entity)
            .expect("entity / query is not alive or valid")
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * [`QueryAPI::each()`]
    /// * [`World::each_entity()`]
    pub fn each<Components>(&self, func: impl FnMut(Components::TupleType<'_>)) -> Query<Components>
    where
        Components: QueryTuple,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each(func);
        query
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * [`QueryAPI::each_entity()`]
    /// * [`World::each()`]
    pub fn each_entity<Components>(
        &self,
        func: impl FnMut(EntityView, Components::TupleType<'_>),
    ) -> Query<Components>
    where
        Components: QueryTuple,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each_entity(func);
        query
    }
}
