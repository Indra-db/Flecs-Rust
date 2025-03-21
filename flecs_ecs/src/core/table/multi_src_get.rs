use std::marker::PhantomData;

use super::*;

// Can safely access multiple rows of the same column
pub struct GetMulti<'a, 't, const IS_RUN: bool, const N: usize, P, Q> {
    table_iter: &'a mut TableIter<'t, IS_RUN, P>,
    var_list: [&'static str; N],
    _phantom: PhantomData<Q>,
}

impl<'a, 't, const IS_RUN: bool, const N: usize, P, Q> GetMulti<'a, 't, IS_RUN, N, P, Q>
where
    P: ComponentId,
    Q: GetTuple,
{
    fn with_vars<const NEW_SIZE: usize>(
        self,
        vars: [&'static str; NEW_SIZE],
    ) -> GetMulti<'a, 't, IS_RUN, NEW_SIZE, P, Q> {
        GetMulti {
            table_iter: self.table_iter,
            var_list: vars,
            _phantom: PhantomData,
        }
    }

    // loops through the entities of each var
    fn each(&mut self, func: impl for<'q> FnMut(Q::TupleType<'q>)) {}

    // loops through the entities of each var at most once
    fn each_once(&mut self, func: impl for<'q> FnMut(Q::TupleType<'q>)) {}

    // tries to get the rows from all entities & destructures them into an array
    fn expand(&mut self, func: impl for<'q> FnMut([Q::TupleType<'q>; N])) {}

    // fallible version
    fn try_expand(&mut self, func: impl for<'q> FnMut([Q::TupleType<'q>; N])) {}
}

/*
iter.get_multi::<&mut Transform>()
    .with_vars(["$parent", "$child"])
    .expand(|[parent_transf, child_transf]| {
        // ..
    });

    .with_vars(["$p0", "$p1"])
.each_once::<Velocity>(|vel| vel += 20);
*/
