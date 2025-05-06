use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use flecs_ecs_derive::Component;
use flecs_ecs_sys::ecs_table_t;

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
    // Create a new instance with different variables
    pub fn with_vars<const NEW_SIZE: usize>(
        self,
        vars: [&'static str; NEW_SIZE],
    ) -> GetMulti<'a, 't, IS_RUN, NEW_SIZE, P, Q> {
        GetMulti {
            table_iter: self.table_iter,
            var_list: vars,
            _phantom: PhantomData,
        }
    }

    // Check if all variables refer to different entities
    // This is crucial for ensuring Rust's safety rules when working with mutable references
    fn ensure_unique_entities(&self) -> Result<(), String> {
        let mut entities = HashSet::new();

        for var_name in &self.var_list {
            let entity = self.table_iter.get_var_by_name(var_name);
            if !entities.insert(entity.id()) {
                return Err(format!(
                    "Variable '{}' refers to an entity that's already been referenced by another variable",
                    var_name
                ));
            }
        }

        Ok(())
    }

    // // loops through the entities of each var
    // pub fn each(&mut self, mut func: impl for<'q> FnMut(Q::TupleType<'q>)) {
    //     // Safety check to ensure we don't have aliasing references
    //     if let Err(err) = self.ensure_unique_entities() {
    //         panic!("Safety violation in GetMulti::each: {}", err);
    //     }

    //     // Get entity for each variable
    //     let mut entities = Vec::with_capacity(N);
    //     for var_name in &self.var_list {
    //         entities.push(self.table_iter.get_var_by_name(var_name));
    //     }

    //     // Extract tuples and call function for each entity's components
    //     for entity in entities {
    //         let tuple = Q::get_from_entity(entity);
    //         if let Some(tuple) = tuple {
    //             func(tuple);
    //         }
    //     }
    // }

    // // loops through the entities of each var at most once
    // pub fn each_once(&mut self, mut func: impl for<'q> FnMut(Q::TupleType<'q>)) {
    //     // Safety check to ensure we don't have aliasing references
    //     if let Err(err) = self.ensure_unique_entities() {
    //         panic!("Safety violation in GetMulti::each_once: {}", err);
    //     }

    //     // Get unique entities from the variables
    //     let mut entities = Vec::with_capacity(N);
    //     let mut processed = HashSet::new();

    //     for var_name in &self.var_list {
    //         let entity = self.table_iter.get_var_by_name(var_name);
    //         if processed.insert(entity.id()) {
    //             entities.push(entity);
    //         }
    //     }

    //     // Process each unique entity
    //     for entity in entities {
    //         let tuple = Q::get_from_entity(entity);
    //         if let Some(tuple) = tuple {
    //             func(tuple);
    //         }
    //     }
    // }

    // // tries to get the rows from all entities & destructures them into an array
    // pub fn expand(&mut self, mut func: impl for<'q> FnMut([Q::TupleType<'q>; N])) {
    //     // Safety check to ensure we don't have aliasing references
    //     if let Err(err) = self.ensure_unique_entities() {
    //         panic!("Safety violation in GetMulti::expand: {}", err);
    //     }

    //     // Collect all entities and their components
    //     let mut entity_tuples = Vec::with_capacity(N);

    //     for var_name in &self.var_list {
    //         let entity = self.table_iter.get_var_by_name(var_name);
    //         if let Some(tuple) = Q::get_from_entity(entity) {
    //             entity_tuples.push(tuple);
    //         } else {
    //             // If any entity is missing required components, we can't continue
    //             return;
    //         }
    //     }

    //     // Convert Vec to array and call the function
    //     // This requires const generic array conversion, using transmute safely
    //     if entity_tuples.len() == N {
    //         // Safety: We've verified the length matches N
    //         let array = unsafe {
    //             std::mem::transmute_copy::<Vec<Q::TupleType<'_>>, [Q::TupleType<'_>; N]>(
    //                 &entity_tuples,
    //             )
    //         };
    //         func(array);
    //     }
    // }

    // // fallible version
    // pub fn try_expand(
    //     &mut self,
    //     mut func: impl for<'q> FnMut([Q::TupleType<'q>; N]),
    // ) -> Result<(), String> {
    //     // Safety check to ensure we don't have aliasing references
    //     self.ensure_unique_entities()?;

    //     // Collect all entities and their components
    //     let mut entity_tuples = Vec::with_capacity(N);
    //     let mut missing = None;

    //     for (i, var_name) in self.var_list.iter().enumerate() {
    //         let entity = self.table_iter.get_var_by_name(var_name);
    //         if let Some(tuple) = Q::get_from_entity(entity) {
    //             entity_tuples.push(tuple);
    //         } else {
    //             missing = Some((i, var_name));
    //             break;
    //         }
    //     }

    //     // If any entity is missing required components, return error
    //     if let Some((i, var_name)) = missing {
    //         return Err(format!(
    //             "Entity from variable '{}' at index {} is missing required components",
    //             var_name, i
    //         ));
    //     }

    //     // Convert Vec to array and call the function
    //     if entity_tuples.len() == N {
    //         // Safety: We've verified the length matches N
    //         let array = unsafe {
    //             std::mem::transmute_copy::<Vec<Q::TupleType<'_>>, [Q::TupleType<'_>; N]>(
    //                 &entity_tuples,
    //             )
    //         };
    //         func(array);
    //         Ok(())
    //     } else {
    //         Err(format!(
    //             "Expected {} entities with components but found {}",
    //             N,
    //             entity_tuples.len()
    //         ))
    //     }
    // }
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

// pub struct Fields<'a, T: ComponentId, const READ_SIZE: usize, const WRITE_SIZE: usize> {
//     pub(crate) fields: [Field<'a, T, false>; READ_SIZE],
//     pub(crate) fields_mut: [FieldMut<'a, T, false>; WRITE_SIZE],
// }

// impl<'a, T: ComponentId, const READ_SIZE: usize, const WRITE_SIZE: usize>
//     Fields<'a, T, READ_SIZE, WRITE_SIZE>
// {
//     pub fn new(fields: [i8; READ_SIZE], fields_mut: [i8; WRITE_SIZE]) -> Self {
//         Fields {
//             fields: fields.map(|f| Field::<T, false>::new(f)),
//             fields_mut: fields_mut.map(|f| FieldMut::new(f)),
//         }
//     }
// }

pub struct Fields<const READ_SIZE: usize, const WRITE_SIZE: usize> {
    pub(crate) fields: [bool; READ_SIZE],
    pub(crate) fields_mut: [bool; WRITE_SIZE],
}

impl<const READ_SIZE: usize, const WRITE_SIZE: usize> Fields<READ_SIZE, WRITE_SIZE> {
    pub fn new(fields: [bool; READ_SIZE], fields_mut: [bool; WRITE_SIZE]) -> Self {
        Fields {
            fields: fields,
            fields_mut: fields_mut,
        }
    }
}

#[test]
fn test_optnal() {
    use flecs_ecs::prelude::*;

    let world = World::new();

    #[derive(Component, Debug)]
    struct A {
        x: i32,
    }
    #[derive(Component, Debug)]
    struct B {
        y: i32,
    }

    world.entity().set(A { x: 1 });
    world.entity().set(A { x: 3 }).set(B { y: 4 });

    let q = world.query::<(&A, Option<&B>)>().build();

    q.run(|mut it| {
        while it.next() {
            let a = it.field::<A>(0).unwrap();
            let b = it.field::<B>(1).unwrap();

            for i in it.iter() {
                println!("a: {:?}", a[i]);
                if it.is_set(1) {
                    println!("b: {:?}", b[i]);
                } else {
                    println!("b is None");
                }
            }
        }
    });
}

#[test]
fn test_so() {
    use flecs_ecs::prelude::*;

    #[derive(Component)]
    pub struct PlaysFor;

    #[derive(Component)]
    pub struct BasedIn;

    #[derive(Component, Debug)]
    pub struct Transform {
        x: f32,
        y: f32,
    }

    let world = World::new();

    //a city
    let new_york = world
        .entity_named("New York")
        .set(Transform { x: 300.0, y: 200.0 });

    // a team that is based in a city.
    let a_team = world
        .entity_named("a_team")
        .add_first::<BasedIn>(new_york)
        .set(Transform { x: 40.0, y: -74.0 });

    // a player that plays for the team.
    world
        .entity_named("John")
        .add_first::<PlaysFor>(a_team)
        .set(Transform { x: 20.0, y: -40.0 });

    let rule = world
        .query::<()>()
        .with_first_name::<&PlaysFor>("$team")
        .with_first_name::<&BasedIn>("$city")
        .set_src_name("$team")
        .with::<&mut Transform>() //this
        .with::<&mut Transform>()
        .set_src_name("$team")
        .with::<&mut Transform>()
        .set_src_name("$city")
        .build();

    rule.run(|mut it| {
        while it.next() {
            let player_transforms = it.field_mut::<Transform>(2).unwrap();
            let team_transforms = it.field::<Transform>(3).unwrap();
            let city_transforms = it.field::<Transform>(4).unwrap();

            //for all the unique tables, lock the columns
            // let fields = Fields::new([false, true], [false]);

            // let fields = if true {
            //     Fields::new([true, true], [false])
            // } else {
            //     Fields::new([false], [false])
            // };

            for i in it.iter() {
                // // lock that it accessed individual fields in Transform
                // let (team_transform, city_transform) =
                //     it.get(((team_transforms, i), (city_transforms, i)));

                // //so that we cannot do, this will panic
                // let city_transform = it.get(((city_transforms, i)));
                // let team_transform = it.get(((team_transforms, i)));

                //old approach
                let player_transform = &player_transforms[i];
                let team_transform = &team_transforms[i];
                let city_transform = &city_transforms[i];

                println!("transform of player: {:?}", player_transform);
                println!("transform of team: {:?}", team_transform);
                println!("transform of city: {:?}", city_transform);
            }
        }
    });
}

#[test]
fn test_something_final2() {
    impl<'a, const IS_RUN: bool, P> TableIter<'a, IS_RUN, P>
    where
        P: ComponentId,
    {
        pub fn getx2<'w, T: ComponentId, F>(&'w self, fields_indices: F) -> F::TupleType<'w>
        where
            F: FieldsTuple2<T> + 'w,
        {
            let world = self.world();
            fields_indices.get_tuple(&world, self)
        }

        pub fn getx3<'w, T: ComponentId, F>(&'w self, fields_indices: F) -> F::TupleType<'w>
        where
            F: FieldsTuple2<T> + 'w,
        {
            let world = self.world();
            fields_indices.get_tuple(&world, self)
        }
    }

    #[derive(Component, Debug)]
    struct TP {
        x: i32,
    }

    #[derive(Component, Debug)]
    struct Relx;

    let world = World::new();

    let parent = world.entity().set(TP { x: 1 });
    parent.add_first::<Relx>(parent);
    let e = world.entity().set(TP { x: 4 }).add_first::<Relx>(parent);
    world
        .query::<(&mut TP, &TP)>()
        .with_first_name::<Relx>("$parent")
        .term_at(1)
        .set_src_name("$parent")
        .build()
        .run(|mut it| {
            while it.next() {
                match it.getx2::<TP, _>((Mut(0), 1)) {
                    (Ok(val), Ok(val2)) => {
                        println!("val: {:?}, val2: {:?}", val, val2);
                    }
                    (Ok(val), Err(e)) => {
                        println!("val: {:?}, Error: {:?}", val, e);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
        });

    pub struct Fields<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    >
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
        mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
        has_any_tables_the_same: bool,
        duplicate_table_ids: smallvec::SmallVec<[u64; HALF_TOTAL]>,
    }

    pub(crate) trait FieldContainer<'a, T: ComponentId> {
        const TOTAL: usize;

        fn lock_tables(&self, world: &WorldRef);

        fn unlock_tables(&self);

        fn duplicate_table_ids(&self) -> &[u64];

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType;

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType;

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false>;

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false>;
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Drop for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        fn drop(&mut self) {
            self.unlock_tables();
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > FieldContainer<'a, T> for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        const TOTAL: usize = Total;

        fn lock_tables(&self, world: &WorldRef) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.lock_table(world);
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.lock_table(world);
                }
            }
        }

        fn unlock_tables(&self) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.unlock_table();
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.unlock_table();
                }
            }
        }

        fn duplicate_table_ids(&self) -> &[u64] {
            &self.duplicate_table_ids
        }

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType {
            &self.immut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType {
            &mut self.mut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false> {
            &self.immut_fields[index_field]
        }

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false> {
            &mut self.mut_fields[index_field]
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        unsafe fn to_initialized_array<Field, const NR: usize>(
            array: [MaybeUninit<Field>; NR],
        ) -> [Field; NR] {
            unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
        }

        pub fn new(
            iter: &'a TableIter,
            immut_fields: [usize; NI],
            mut_fields: [usize; NM],
        ) -> Self {
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
            let mut immut_fields_table_ids = [0; NI];
            for i in 0..NI {
                immut_fields_table_ids[i] = immut_fields[i].table_id();
            }

            let mut mut_fields_table_ids = [0; NM];
            for i in 0..NM {
                mut_fields_table_ids[i] = mut_fields[i].table_id();
            }

            // check all table ids in the mutable and immutable fields and store the duplicates, make sure not to store duplicates twice
            // we have to check the table ids of the two arrays as well as itself the array
            let mut duplicate_table_ids = smallvec::SmallVec::<[u64; HALF_TOTAL]>::new();
            let mut has_any_tables_the_same = false;

            for i in 0..NI {
                for j in (i + 1)..NI {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    if immut_field_table_id == immut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NM {
                for j in (i + 1)..NM {
                    let mut_field_table_id = mut_fields_table_ids[i];
                    if mut_field_table_id == mut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&mut_field_table_id) {
                            duplicate_table_ids.push(mut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NI {
                for j in 0..NM {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    let mut_field_table_id = mut_fields_table_ids[j];
                    if immut_field_table_id == mut_field_table_id {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_field_table_id);
                        }
                    }
                }
            }

            let fields = Self {
                immut_fields,
                mut_fields,
                has_any_tables_the_same,
                duplicate_table_ids,
            };

            fields.lock_tables(&iter.world());

            fields
        }

        pub fn get<'w, F, Func, const IS_RUN: bool, P: ComponentId>(
            &self,
            iter: &'w TableIter<'_, IS_RUN, P>,
            fields_indices: F,
            mut func: Func,
        ) where
            F: FieldsTuple2<T> + 'w,
            Func: FnMut(F::TupleType<'w>) + 'w,
        {
            let world = iter.world();
            let tuple = fields_indices.get_tuple(&world, iter);
            func(tuple);
        }

        // pub fn get<'f, F: FieldsTuple<T>, Func: FnMut(F::TupleType<'f>)>(
        //     &self,
        //     fields: F,
        //     mut func: Func,
        // ) {
        //     // let tuple = F::create_tuple();
        //     //func(tuple);
        // }
    }

    struct Mut(usize);

    impl<A, B, T> FieldsTuple2<T> for (A, B)
    where
        T: 'static + ComponentId,
        A: IterableTypeFieldOperation2<T>,
        B: IterableTypeFieldOperation2<T>,
    {
        type TupleType<'w>
            = (
            <A as IterableTypeFieldOperation2<T>>::ActualType<'w>,
            <B as IterableTypeFieldOperation2<T>>::ActualType<'w>,
        )
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::TupleType<'a> {
            (self.0.get_data(iter), self.1.get_data(iter))
            // const {
            //     // if FIELDS::TOTAL != 2 {
            //     //     panic!("total indices should be {}", FIELDS::TOTAL);
            //     // }
            // }
            // let duplicates = fields.duplicate_table_ids();
            // let any_duplicates = !duplicates.is_empty();
            // unimplemented!()
            // if !any_duplicates {
            //     // self.0.lock_column(world);
            //     // self.1.lock_column(world);
            //     return (self.0.get_data(0, fields), self.1.get_data(1, fields));
            // } else {
            //     let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
            //         smallvec::SmallVec::new();
            //     let sources = iter.sources();
            //     let entities = iter.entities();
            //     let table_id = self.0.table_id(fields, 0);
            //     let field_index = self.0.field_index(fields, 0);
            //     let src_field_index = sources[field_index as usize];
            //     let entity_id = if src_field_index == 0 {
            //         entities[field_index as usize]
            //     } else {
            //         src_field_index
            //     };

            //     if duplicates.contains(&table_id) {
            //         let was_locked = locked_entities.iter().any(|&id| id == entity_id);

            //         if was_locked {
            //             panic!("Entity already locked");
            //         }
            //         locked_entities.push(entity_id);
            //     }
            // }
            // (self.0.get_data(0, fields), self.1.get_data(1, fields))
        }
    }

    pub trait FieldsTuple2<T: ComponentId>: Sized {
        type TupleType<'w>
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::TupleType<'a>;
    }

    pub trait IterableTypeFieldOperation2<T: 'static + ComponentId> {
        type ActualType<'w>
        where
            Self: 'w;

        fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::ActualType<'a>
        where
            Self: 'a;

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64;

        // fn field_index<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> i8;
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for usize {
        type ActualType<'w>
            = Result<Field<'w, T::UnderlyingType, true>, ()>
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

        // fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
        //     &self,
        //     fields: &'w mut FIELDS,
        //     index: usize,
        // ) -> i8 {
        //     fields.get_field(index).field_index
        // }

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64 {
        //     fields.get_field(index).table_id()
        // }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_read_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for Mut {
        type ActualType<'w>
            = Result<FieldMut<'w, T::UnderlyingType, true>, ()>
        where
            Self: 'w;

        fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            iter.field_mut_result::<T>(self.0 as i8)
        }

        // fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
        //     &self,
        //     fields: &'w mut FIELDS,
        //     index: usize,
        // ) -> i8 {
        //     fields.get_field_mut(index).field_index
        // }

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64 {
        //     fields.get_field_mut(index).table_id()
        // }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_write_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }
}

#[test]
fn test_something_final() {
    impl<'a, const IS_RUN: bool, P> TableIter<'a, IS_RUN, P>
    where
        P: ComponentId,
    {
        pub fn getx<'w, T: ComponentId, F>(
            &'w self,
            fields_indices: F,
            mut func: impl FnMut(F::TupleType<'w>) + 'w,
        ) where
            F: FieldsTuple2<T> + 'w,
        {
            let world = self.world();
            let tuple = fields_indices.get_tuple(&world, self);
            func(tuple);
        }
    }
    #[derive(Component)]
    struct TP {
        x: i32,
    }

    let world = World::new();

    let e = world.entity().set(TP { x: 4 });

    world.query::<&TP>().build().run(|mut it| {
        while it.next() {
            it.getx::<TP, _>((0, 1), |(val, val2)| {
                //let valx = val[0];
            });
        }
    });

    pub struct Fields<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    >
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
        mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
        has_any_tables_the_same: bool,
        duplicate_table_ids: smallvec::SmallVec<[u64; HALF_TOTAL]>,
    }

    pub(crate) trait FieldContainer<'a, T: ComponentId> {
        const TOTAL: usize;

        fn lock_tables(&self, world: &WorldRef);

        fn unlock_tables(&self);

        fn duplicate_table_ids(&self) -> &[u64];

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType;

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType;

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false>;

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false>;
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Drop for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        fn drop(&mut self) {
            self.unlock_tables();
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > FieldContainer<'a, T> for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        const TOTAL: usize = Total;

        fn lock_tables(&self, world: &WorldRef) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.lock_table(world);
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.lock_table(world);
                }
            }
        }

        fn unlock_tables(&self) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.unlock_table();
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.unlock_table();
                }
            }
        }

        fn duplicate_table_ids(&self) -> &[u64] {
            &self.duplicate_table_ids
        }

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType {
            &self.immut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType {
            &mut self.mut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false> {
            &self.immut_fields[index_field]
        }

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false> {
            &mut self.mut_fields[index_field]
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        unsafe fn to_initialized_array<Field, const NR: usize>(
            array: [MaybeUninit<Field>; NR],
        ) -> [Field; NR] {
            unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
        }

        pub fn new(
            iter: &'a TableIter,
            immut_fields: [usize; NI],
            mut_fields: [usize; NM],
        ) -> Self {
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
            let mut immut_fields_table_ids = [0; NI];
            for i in 0..NI {
                immut_fields_table_ids[i] = immut_fields[i].table_id();
            }

            let mut mut_fields_table_ids = [0; NM];
            for i in 0..NM {
                mut_fields_table_ids[i] = mut_fields[i].table_id();
            }

            // check all table ids in the mutable and immutable fields and store the duplicates, make sure not to store duplicates twice
            // we have to check the table ids of the two arrays as well as itself the array
            let mut duplicate_table_ids = smallvec::SmallVec::<[u64; HALF_TOTAL]>::new();
            let mut has_any_tables_the_same = false;

            for i in 0..NI {
                for j in (i + 1)..NI {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    if immut_field_table_id == immut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NM {
                for j in (i + 1)..NM {
                    let mut_field_table_id = mut_fields_table_ids[i];
                    if mut_field_table_id == mut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&mut_field_table_id) {
                            duplicate_table_ids.push(mut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NI {
                for j in 0..NM {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    let mut_field_table_id = mut_fields_table_ids[j];
                    if immut_field_table_id == mut_field_table_id {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_field_table_id);
                        }
                    }
                }
            }

            let fields = Self {
                immut_fields,
                mut_fields,
                has_any_tables_the_same,
                duplicate_table_ids,
            };

            fields.lock_tables(&iter.world());

            fields
        }

        pub fn get<'w, F, Func, const IS_RUN: bool, P: ComponentId>(
            &self,
            iter: &'w TableIter<'_, IS_RUN, P>,
            fields_indices: F,
            mut func: Func,
        ) where
            F: FieldsTuple2<T> + 'w,
            Func: FnMut(F::TupleType<'w>) + 'w,
        {
            let world = iter.world();
            let tuple = fields_indices.get_tuple(&world, iter);
            func(tuple);
        }

        // pub fn get<'f, F: FieldsTuple<T>, Func: FnMut(F::TupleType<'f>)>(
        //     &self,
        //     fields: F,
        //     mut func: Func,
        // ) {
        //     // let tuple = F::create_tuple();
        //     //func(tuple);
        // }
    }

    struct Mut(usize);

    impl<A, B, T> FieldsTuple2<T> for (A, B)
    where
        T: 'static + ComponentId,
        A: IterableTypeFieldOperation2<T>,
        B: IterableTypeFieldOperation2<T>,
    {
        type TupleType<'w>
            = (
            <A as IterableTypeFieldOperation2<T>>::ActualType<'w>,
            <B as IterableTypeFieldOperation2<T>>::ActualType<'w>,
        )
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::TupleType<'a> {
            (self.0.get_data(iter), self.1.get_data(iter))
            // const {
            //     // if FIELDS::TOTAL != 2 {
            //     //     panic!("total indices should be {}", FIELDS::TOTAL);
            //     // }
            // }
            // let duplicates = fields.duplicate_table_ids();
            // let any_duplicates = !duplicates.is_empty();
            // unimplemented!()
            // if !any_duplicates {
            //     // self.0.lock_column(world);
            //     // self.1.lock_column(world);
            //     return (self.0.get_data(0, fields), self.1.get_data(1, fields));
            // } else {
            //     let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
            //         smallvec::SmallVec::new();
            //     let sources = iter.sources();
            //     let entities = iter.entities();
            //     let table_id = self.0.table_id(fields, 0);
            //     let field_index = self.0.field_index(fields, 0);
            //     let src_field_index = sources[field_index as usize];
            //     let entity_id = if src_field_index == 0 {
            //         entities[field_index as usize]
            //     } else {
            //         src_field_index
            //     };

            //     if duplicates.contains(&table_id) {
            //         let was_locked = locked_entities.iter().any(|&id| id == entity_id);

            //         if was_locked {
            //             panic!("Entity already locked");
            //         }
            //         locked_entities.push(entity_id);
            //     }
            // }
            // (self.0.get_data(0, fields), self.1.get_data(1, fields))
        }
    }

    pub trait FieldsTuple2<T: ComponentId>: Sized {
        type TupleType<'w>
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::TupleType<'a>;
    }

    pub trait IterableTypeFieldOperation2<T: 'static + ComponentId> {
        type ActualType<'w>
        where
            Self: 'w;

        fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::ActualType<'a>
        where
            Self: 'a;

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64;

        // fn field_index<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> i8;
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for usize {
        type ActualType<'w>
            = Field<'w, T::UnderlyingType, false>
        where
            Self: 'w;

        fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            iter.field_lockless::<T>(self as i8)
                .expect("user should valid check before using")
        }

        // fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
        //     &self,
        //     fields: &'w mut FIELDS,
        //     index: usize,
        // ) -> i8 {
        //     fields.get_field(index).field_index
        // }

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64 {
        //     fields.get_field(index).table_id()
        // }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_read_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for Mut {
        type ActualType<'w>
            = FieldMut<'w, T::UnderlyingType, false>
        where
            Self: 'w;

        fn get_data<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            iter: &'a TableIter<'a, IS_RUN, P>,
        ) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            iter.field_mut_lockless::<T>(self.0 as i8)
                .expect("user should valid check before using")
        }

        // fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
        //     &self,
        //     fields: &'w mut FIELDS,
        //     index: usize,
        // ) -> i8 {
        //     fields.get_field_mut(index).field_index
        // }

        // fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
        //     &self,
        //     fields: &'a mut FIELDS,
        //     index: usize,
        // ) -> u64 {
        //     fields.get_field_mut(index).table_id()
        // }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_write_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }
}

#[test]
fn test_something_not_final() {
    pub struct Fields<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    >
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
        mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
        has_any_tables_the_same: bool,
        duplicate_table_ids: smallvec::SmallVec<[u64; HALF_TOTAL]>,
    }

    pub(crate) trait FieldContainer<'a, T: ComponentId> {
        const TOTAL: usize;

        fn lock_tables(&self, world: &WorldRef);

        fn unlock_tables(&self);

        fn duplicate_table_ids(&self) -> &[u64];

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType;

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType;

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false>;

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false>;
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Drop for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        fn drop(&mut self) {
            self.unlock_tables();
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > FieldContainer<'a, T> for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        const TOTAL: usize = Total;

        fn lock_tables(&self, world: &WorldRef) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.lock_table(world);
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.lock_table(world);
                }
            }
        }

        fn unlock_tables(&self) {
            let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
            let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

            for i in 0..NI {
                let field = &self.immut_fields[i];
                let table_id = field.table_id();
                if !fields_tables.contains(&table_id) {
                    fields_tables.push(field.table_id());
                    field.unlock_table();
                }
            }

            for i in 0..NM {
                let field = &self.mut_fields[i];
                let table_id = field.table_id();
                if !fields_tables_mut.contains(&table_id) {
                    fields_tables_mut.push(field.table_id());
                    field.unlock_table();
                }
            }
        }

        fn duplicate_table_ids(&self) -> &[u64] {
            &self.duplicate_table_ids
        }

        fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType {
            &self.immut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_mut(
            &mut self,
            index_field: usize,
            index_slice_components: usize,
        ) -> &mut T::UnderlyingType {
            &mut self.mut_fields[index_field].slice_components[index_slice_components]
        }

        fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false> {
            &self.immut_fields[index_field]
        }

        fn get_field_mut(
            &mut self,
            index_field: usize,
        ) -> &mut FieldMut<'a, T::UnderlyingType, false> {
            &mut self.mut_fields[index_field]
        }
    }

    impl<
        'a,
        T: ComponentId,
        const NI: usize,
        const NM: usize,
        const Total: usize,
        const HALF_TOTAL: usize,
    > Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    where
        [u64; HALF_TOTAL]: smallvec::Array,
        [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
        [u64; NI]: smallvec::Array<Item = u64>,
        [u64; NM]: smallvec::Array<Item = u64>,
    {
        unsafe fn to_initialized_array<Field, const NR: usize>(
            array: [MaybeUninit<Field>; NR],
        ) -> [Field; NR] {
            unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
        }

        pub fn new(
            iter: &'a TableIter,
            immut_fields: [usize; NI],
            mut_fields: [usize; NM],
        ) -> Self {
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
            let mut immut_fields_table_ids = [0; NI];
            for i in 0..NI {
                immut_fields_table_ids[i] = immut_fields[i].table_id();
            }

            let mut mut_fields_table_ids = [0; NM];
            for i in 0..NM {
                mut_fields_table_ids[i] = mut_fields[i].table_id();
            }

            // check all table ids in the mutable and immutable fields and store the duplicates, make sure not to store duplicates twice
            // we have to check the table ids of the two arrays as well as itself the array
            let mut duplicate_table_ids = smallvec::SmallVec::<[u64; HALF_TOTAL]>::new();
            let mut has_any_tables_the_same = false;

            for i in 0..NI {
                for j in (i + 1)..NI {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    if immut_field_table_id == immut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NM {
                for j in (i + 1)..NM {
                    let mut_field_table_id = mut_fields_table_ids[i];
                    if mut_field_table_id == mut_fields_table_ids[j] {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&mut_field_table_id) {
                            duplicate_table_ids.push(mut_fields_table_ids[i]);
                        }
                    }
                }
            }

            for i in 0..NI {
                for j in 0..NM {
                    let immut_field_table_id = immut_fields_table_ids[i];
                    let mut_field_table_id = mut_fields_table_ids[j];
                    if immut_field_table_id == mut_field_table_id {
                        has_any_tables_the_same = true;
                        if !duplicate_table_ids.contains(&immut_field_table_id) {
                            duplicate_table_ids.push(immut_field_table_id);
                        }
                    }
                }
            }

            let fields = Self {
                immut_fields,
                mut_fields,
                has_any_tables_the_same,
                duplicate_table_ids,
            };

            fields.lock_tables(&iter.world());

            fields
        }

        pub fn get<'w, F, Func, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'w, T>>(
            &self,
            iter: &'w TableIter<'_, IS_RUN, P>,
            fields: &'w mut FIELDS,
            fields_indices: F,
            mut func: Func,
        ) where
            F: FieldsTuple2<T> + 'w,
            Func: FnMut(F::TupleType<'w>) + 'w,
        {
            let world = iter.world();
            let tuple = fields_indices.get_tuple(&world, iter, fields);
            func(tuple);
        }

        // pub fn get<'f, F: FieldsTuple<T>, Func: FnMut(F::TupleType<'f>)>(
        //     &self,
        //     fields: F,
        //     mut func: Func,
        // ) {
        //     // let tuple = F::create_tuple();
        //     //func(tuple);
        // }
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
        let half_size_total = (size_total + 1) / 2;

        let arr_immut = fields_immut_str[0..size_immut].join(",");
        let arr_mut = fields_mut_str[0..size_mut].join(",");
        crabtime::output! {
           Fields::<{{typename}}, {{size_immut}}, {{size_mut}}, {{size_total}}, {{half_size_total}}>::new([{{arr_immut}}], [{{arr_mut}}])
        }
    }

    struct Mut(usize);

    impl<A, B, T> FieldsTuple2<T> for (A, B)
    where
        T: 'static + ComponentId,
        A: IterableTypeFieldOperation2<T>,
        B: IterableTypeFieldOperation2<T>,
    {
        type TupleType<'w>
            = (
            <A as IterableTypeFieldOperation2<T>>::ActualType<'w>,
            <B as IterableTypeFieldOperation2<T>>::ActualType<'w>,
        )
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
            fields: &'a mut FIELDS,
        ) -> Self::TupleType<'a> {
            unimplemented!();
            const {
                // if FIELDS::TOTAL != 2 {
                //     panic!("total indices should be {}", FIELDS::TOTAL);
                // }
            }
            let duplicates = fields.duplicate_table_ids();
            let any_duplicates = !duplicates.is_empty();
            if !any_duplicates {
                // self.0.lock_column(world);
                // self.1.lock_column(world);
                return (self.0.get_data(0, fields), self.1.get_data(1, fields));
            } else {
                let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
                    smallvec::SmallVec::new();
                let sources = iter.sources();
                let entities = iter.entities();
                let table_id = self.0.table_id(fields, 0);
                let field_index = self.0.field_index(fields, 0);
                let src_field_index = sources[field_index as usize];
                let entity_id = if src_field_index == 0 {
                    entities[field_index as usize]
                } else {
                    src_field_index
                };

                if duplicates.contains(&table_id) {
                    let was_locked = locked_entities.iter().any(|&id| id == entity_id);

                    if was_locked {
                        panic!("Entity already locked");
                    }
                    locked_entities.push(entity_id);
                }
            }
            (self.0.get_data(0, fields), self.1.get_data(1, fields))
        }
    }

    pub trait FieldsTuple2<T: ComponentId>: Sized {
        type TupleType<'w>
        where
            Self: 'w;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
            fields: &'a mut FIELDS,
        ) -> Self::TupleType<'a>;
    }

    pub trait IterableTypeFieldOperation2<T: 'static + ComponentId> {
        type ActualType<'w>
        where
            Self: 'w;
        fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
            self,
            index_field: usize,
            fields: &'a mut FIELDS,
        ) -> Self::ActualType<'a>
        where
            Self: 'a;

        fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
            &self,
            fields: &'a mut FIELDS,
            index: usize,
        ) -> u64;

        fn field_index<'a, FIELDS: FieldContainer<'a, T>>(
            &self,
            fields: &'a mut FIELDS,
            index: usize,
        ) -> i8;
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for usize {
        type ActualType<'w>
            = &'w T::UnderlyingType
        where
            Self: 'w;

        fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
            self,
            index_field: usize,
            fields: &'a mut FIELDS,
        ) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            fields.get(index_field, self)
        }

        fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
            &self,
            fields: &'w mut FIELDS,
            index: usize,
        ) -> i8 {
            fields.get_field(index).field_index
        }

        fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
            &self,
            fields: &'a mut FIELDS,
            index: usize,
        ) -> u64 {
            fields.get_field(index).table_id()
        }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_read_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }

    impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for Mut {
        type ActualType<'w>
            = &'w mut T::UnderlyingType
        where
            Self: 'w;

        fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
            self,
            index_field: usize,
            fields: &'a mut FIELDS,
        ) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            fields.get_mut(index_field, self.0)
        }

        fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
            &self,
            fields: &'w mut FIELDS,
            index: usize,
        ) -> i8 {
            fields.get_field_mut(index).field_index
        }

        fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
            &self,
            fields: &'a mut FIELDS,
            index: usize,
        ) -> u64 {
            fields.get_field_mut(index).table_id()
        }

        // fn lock_column(&self, world: &WorldRef) {
        //     get_table_column_lock_write_begin(
        //         world,
        //         self.0.table.as_ptr(),
        //         self.0.column_index,
        //         self.0.stage_id,
        //     );
        // }
    }
}

#[test]
fn test_something_not_final2() {
    //     pub struct Fields<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     >
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
    //         mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
    //         has_any_tables_the_same: bool,
    //         duplicate_table_ids: smallvec::SmallVec<[u64; HALF_TOTAL]>,
    //     }

    //     pub(crate) trait FieldContainer<'a, T: ComponentId> {
    //         const TOTAL: usize;

    //         fn lock_tables(&self, world: &WorldRef);

    //         fn unlock_tables(&self);

    //         fn duplicate_table_ids(&self) -> &[u64];

    //         fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType;

    //         fn get_mut(
    //             &mut self,
    //             index_field: usize,
    //             index_slice_components: usize,
    //         ) -> &mut T::UnderlyingType;

    //         fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false>;

    //         fn get_field_mut(
    //             &mut self,
    //             index_field: usize,
    //         ) -> &mut FieldMut<'a, T::UnderlyingType, false>;
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > Drop for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         fn drop(&mut self) {
    //             self.unlock_tables();
    //         }
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > FieldContainer<'a, T> for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array,
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         const TOTAL: usize = Total;

    //         fn lock_tables(&self, world: &WorldRef) {
    //             let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
    //             let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

    //             for i in 0..NI {
    //                 let field = &self.immut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables.contains(&table_id) {
    //                     fields_tables.push(field.table_id());
    //                     field.lock_table(world);
    //                 }
    //             }

    //             for i in 0..NM {
    //                 let field = &self.mut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables_mut.contains(&table_id) {
    //                     fields_tables_mut.push(field.table_id());
    //                     field.lock_table(world);
    //                 }
    //             }
    //         }

    //         fn unlock_tables(&self) {
    //             let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
    //             let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

    //             for i in 0..NI {
    //                 let field = &self.immut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables.contains(&table_id) {
    //                     fields_tables.push(field.table_id());
    //                     field.unlock_table();
    //                 }
    //             }

    //             for i in 0..NM {
    //                 let field = &self.mut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables_mut.contains(&table_id) {
    //                     fields_tables_mut.push(field.table_id());
    //                     field.unlock_table();
    //                 }
    //             }
    //         }

    //         fn duplicate_table_ids(&self) -> &[u64] {
    //             &self.duplicate_table_ids
    //         }

    //         fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType {
    //             &self.immut_fields[index_field].slice_components[index_slice_components]
    //         }

    //         fn get_mut(
    //             &mut self,
    //             index_field: usize,
    //             index_slice_components: usize,
    //         ) -> &mut T::UnderlyingType {
    //             &mut self.mut_fields[index_field].slice_components[index_slice_components]
    //         }

    //         fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false> {
    //             &self.immut_fields[index_field]
    //         }

    //         fn get_field_mut(
    //             &mut self,
    //             index_field: usize,
    //         ) -> &mut FieldMut<'a, T::UnderlyingType, false> {
    //             &mut self.mut_fields[index_field]
    //         }
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array,
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         unsafe fn to_initialized_array<Field, const NR: usize>(
    //             array: [MaybeUninit<Field>; NR],
    //         ) -> [Field; NR] {
    //             unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
    //         }

    //         pub fn new(
    //             iter: &'a TableIter,
    //             immut_fields: [usize; NI],
    //             mut_fields: [usize; NM],
    //         ) -> Self {
    //             let mut fields_mut_array: [MaybeUninit<FieldMut<'_, T::UnderlyingType, false>>; NM] =
    //                 unsafe { MaybeUninit::uninit().assume_init() };

    //             for i in 0..NM {
    //                 fields_mut_array[i] = MaybeUninit::new(
    //                     iter.field_mut_lockless::<T>(mut_fields[i] as i8)
    //                         .expect("Field is not present or not correct type"),
    //                 );
    //             }

    //             let mut_fields = unsafe { Self::to_initialized_array(fields_mut_array) };

    //             let mut fields_immut_array: [MaybeUninit<Field<'_, T::UnderlyingType, false>>; NI] =
    //                 unsafe { MaybeUninit::uninit().assume_init() };
    //             for i in 0..NI {
    //                 fields_immut_array[i] = MaybeUninit::new(
    //                     iter.field_lockless::<T>(immut_fields[i] as i8)
    //                         .expect("Field is not present or not correct type"),
    //                 );
    //             }

    //             let immut_fields = unsafe { Self::to_initialized_array(fields_immut_array) };
    //             let mut immut_fields_table_ids = [0; NI];
    //             for i in 0..NI {
    //                 immut_fields_table_ids[i] = immut_fields[i].table_id();
    //             }

    //             let mut mut_fields_table_ids = [0; NM];
    //             for i in 0..NM {
    //                 mut_fields_table_ids[i] = mut_fields[i].table_id();
    //             }

    //             // check all table ids in the mutable and immutable fields and store the duplicates, make sure not to store duplicates twice
    //             // we have to check the table ids of the two arrays as well as itself the array
    //             let mut duplicate_table_ids = smallvec::SmallVec::<[u64; HALF_TOTAL]>::new();
    //             let mut has_any_tables_the_same = false;

    //             for i in 0..NI {
    //                 for j in (i + 1)..NI {
    //                     let immut_field_table_id = immut_fields_table_ids[i];
    //                     if immut_field_table_id == immut_fields_table_ids[j] {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&immut_field_table_id) {
    //                             duplicate_table_ids.push(immut_fields_table_ids[i]);
    //                         }
    //                     }
    //                 }
    //             }

    //             for i in 0..NM {
    //                 for j in (i + 1)..NM {
    //                     let mut_field_table_id = mut_fields_table_ids[i];
    //                     if mut_field_table_id == mut_fields_table_ids[j] {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&mut_field_table_id) {
    //                             duplicate_table_ids.push(mut_fields_table_ids[i]);
    //                         }
    //                     }
    //                 }
    //             }

    //             for i in 0..NI {
    //                 for j in 0..NM {
    //                     let immut_field_table_id = immut_fields_table_ids[i];
    //                     let mut_field_table_id = mut_fields_table_ids[j];
    //                     if immut_field_table_id == mut_field_table_id {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&immut_field_table_id) {
    //                             duplicate_table_ids.push(immut_field_table_id);
    //                         }
    //                     }
    //                 }
    //             }

    //             let fields = Self {
    //                 immut_fields,
    //                 mut_fields,
    //                 has_any_tables_the_same,
    //                 duplicate_table_ids,
    //             };

    //             fields.lock_tables(&iter.world());

    //             fields
    //         }

    //         pub fn get<'w, F, Func, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             iter: &'w TableIter<'_, IS_RUN, P>,
    //             fields: &'w mut FIELDS,
    //             fields_indices: F,
    //             mut func: Func,
    //         ) where
    //             F: FieldsTuple2<T> + 'w,
    //             Func: FnMut(F::TupleType<'w>) + 'w,
    //         {
    //             let world = iter.world();
    //             let tuple = fields_indices.get_tuple(&world, iter, fields);
    //             func(tuple);
    //         }

    //         // pub fn get<'f, F: FieldsTuple<T>, Func: FnMut(F::TupleType<'f>)>(
    //         //     &self,
    //         //     fields: F,
    //         //     mut func: Func,
    //         // ) {
    //         //     // let tuple = F::create_tuple();
    //         //     //func(tuple);
    //         // }
    //     }

    //     #[crabtime::expression]
    //     fn fields(typename: String, fields_immut: Vec<usize>, fields_mut: Vec<usize>) {
    //         let fields_immut_str = fields_immut
    //             .iter()
    //             .map(|x| x.to_string())
    //             .collect::<Vec<_>>();

    //         let fields_mut_str = fields_mut.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    //         let size_immut = fields_immut.len();
    //         let size_mut = fields_mut.len();
    //         let size_total = size_immut + size_mut;
    //         let half_size_total = (size_total + 1) / 2;

    //         let arr_immut = fields_immut_str[0..size_immut].join(",");
    //         let arr_mut = fields_mut_str[0..size_mut].join(",");
    //         crabtime::output! {
    //            Fields::<{{typename}}, {{size_immut}}, {{size_mut}}, {{size_total}}, {{half_size_total}}>::new([{{arr_immut}}], [{{arr_mut}}])
    //         }
    //     }

    //     struct Mut(usize);

    //     impl<A, B, T> FieldsTuple2<T> for (A, B)
    //     where
    //         T: 'static + ComponentId,
    //         A: IterableTypeFieldOperation2<T>,
    //         B: IterableTypeFieldOperation2<T>,
    //     {
    //         type TupleType<'w>
    //             = (
    //             <A as IterableTypeFieldOperation2<T>>::ActualType<'w>,
    //             <B as IterableTypeFieldOperation2<T>>::ActualType<'w>,
    //         )
    //         where
    //             Self: 'w;

    //         fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             world: &WorldRef,
    //             iter: &TableIter<'_, IS_RUN, P>,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::TupleType<'a> {
    //             const {
    //                 // if FIELDS::TOTAL != 2 {
    //                 //     panic!("total indices should be {}", FIELDS::TOTAL);
    //                 // }
    //             }
    //             let duplicates = fields.duplicate_table_ids();
    //             let any_duplicates = !duplicates.is_empty();
    //             if !any_duplicates {
    //                 // self.0.lock_column(world);
    //                 // self.1.lock_column(world);
    //                 return (self.0.get_data(0, fields), self.1.get_data(1, fields));
    //             } else {
    //                 let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
    //                     smallvec::SmallVec::new();
    //                 let sources = iter.sources();
    //                 let entities = iter.entities();
    //                 let table_id = self.0.table_id(fields, 0);
    //                 let field_index = self.0.field_index(fields, 0);
    //                 let src_field_index = sources[field_index as usize];
    //                 let entity_id = if src_field_index == 0 {
    //                     entities[field_index as usize]
    //                 } else {
    //                     src_field_index
    //                 };

    //                 if duplicates.contains(&table_id) {
    //                     let was_locked = locked_entities.iter().any(|&id| id == entity_id);

    //                     if was_locked {
    //                         panic!("Entity already locked");
    //                     }
    //                     locked_entities.push(entity_id);
    //                 }
    //             }
    //             (self.0.get_data(0, fields), self.1.get_data(1, fields))
    //         }
    //     }

    //     pub trait FieldsTuple2<T: ComponentId>: Sized {
    //         type TupleType<'w>
    //         where
    //             Self: 'w;

    //         fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             world: &WorldRef,
    //             iter: &TableIter<'_, IS_RUN, P>,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::TupleType<'a>;
    //     }

    //     pub trait IterableTypeFieldOperation2<T: 'static + ComponentId> {
    //         type ActualType<'w>
    //         where
    //             Self: 'w;
    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a;

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64;

    //         fn field_index<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> i8;
    //     }

    //     impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for usize {
    //         type ActualType<'w>
    //             = &'w T::UnderlyingType
    //         where
    //             Self: 'w;

    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a,
    //         {
    //             fields.get(index_field, self)
    //         }

    //         fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             fields: &'w mut FIELDS,
    //             index: usize,
    //         ) -> i8 {
    //             fields.get_field(index).field_index
    //         }

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64 {
    //             fields.get_field(index).table_id()
    //         }

    //         // fn lock_column(&self, world: &WorldRef) {
    //         //     get_table_column_lock_read_begin(
    //         //         world,
    //         //         self.0.table.as_ptr(),
    //         //         self.0.column_index,
    //         //         self.0.stage_id,
    //         //     );
    //         // }
    //     }

    //     impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for Mut {
    //         type ActualType<'w>
    //             = &'w mut T::UnderlyingType
    //         where
    //             Self: 'w;

    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a,
    //         {
    //             fields.get_mut(index_field, self.0)
    //         }

    //         fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             fields: &'w mut FIELDS,
    //             index: usize,
    //         ) -> i8 {
    //             fields.get_field_mut(index).field_index
    //         }

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64 {
    //             fields.get_field_mut(index).table_id()
    //         }

    //         // fn lock_column(&self, world: &WorldRef) {
    //         //     get_table_column_lock_write_begin(
    //         //         world,
    //         //         self.0.table.as_ptr(),
    //         //         self.0.column_index,
    //         //         self.0.stage_id,
    //         //     );
    //         // }
    //     }
    // }

    // #[test]
    // fn test_something_not_final() {
    //     pub struct Fields<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     >
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         pub(crate) immut_fields: [Field<'a, T::UnderlyingType, false>; NI],
    //         mut_fields: [FieldMut<'a, T::UnderlyingType, false>; NM],
    //         has_any_tables_the_same: bool,
    //         duplicate_table_ids: smallvec::SmallVec<[u64; HALF_TOTAL]>,
    //     }

    //     pub(crate) trait FieldContainer<'a, T: ComponentId> {
    //         const TOTAL: usize;

    //         fn lock_tables(&self, world: &WorldRef);

    //         fn unlock_tables(&self);

    //         fn duplicate_table_ids(&self) -> &[u64];

    //         fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType;

    //         fn get_mut(
    //             &mut self,
    //             index_field: usize,
    //             index_slice_components: usize,
    //         ) -> &mut T::UnderlyingType;

    //         fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false>;

    //         fn get_field_mut(
    //             &mut self,
    //             index_field: usize,
    //         ) -> &mut FieldMut<'a, T::UnderlyingType, false>;
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > Drop for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         fn drop(&mut self) {
    //             self.unlock_tables();
    //         }
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > FieldContainer<'a, T> for Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array,
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         const TOTAL: usize = Total;

    //         fn lock_tables(&self, world: &WorldRef) {
    //             let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
    //             let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

    //             for i in 0..NI {
    //                 let field = &self.immut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables.contains(&table_id) {
    //                     fields_tables.push(field.table_id());
    //                     field.lock_table(world);
    //                 }
    //             }

    //             for i in 0..NM {
    //                 let field = &self.mut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables_mut.contains(&table_id) {
    //                     fields_tables_mut.push(field.table_id());
    //                     field.lock_table(world);
    //                 }
    //             }
    //         }

    //         fn unlock_tables(&self) {
    //             let mut fields_tables = smallvec::SmallVec::<[u64; NI]>::new();
    //             let mut fields_tables_mut = smallvec::SmallVec::<[u64; NM]>::new();

    //             for i in 0..NI {
    //                 let field = &self.immut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables.contains(&table_id) {
    //                     fields_tables.push(field.table_id());
    //                     field.unlock_table();
    //                 }
    //             }

    //             for i in 0..NM {
    //                 let field = &self.mut_fields[i];
    //                 let table_id = field.table_id();
    //                 if !fields_tables_mut.contains(&table_id) {
    //                     fields_tables_mut.push(field.table_id());
    //                     field.unlock_table();
    //                 }
    //             }
    //         }

    //         fn duplicate_table_ids(&self) -> &[u64] {
    //             &self.duplicate_table_ids
    //         }

    //         fn get(&self, index_field: usize, index_slice_components: usize) -> &T::UnderlyingType {
    //             &self.immut_fields[index_field].slice_components[index_slice_components]
    //         }

    //         fn get_mut(
    //             &mut self,
    //             index_field: usize,
    //             index_slice_components: usize,
    //         ) -> &mut T::UnderlyingType {
    //             &mut self.mut_fields[index_field].slice_components[index_slice_components]
    //         }

    //         fn get_field(&self, index_field: usize) -> &Field<'a, T::UnderlyingType, false> {
    //             &self.immut_fields[index_field]
    //         }

    //         fn get_field_mut(
    //             &mut self,
    //             index_field: usize,
    //         ) -> &mut FieldMut<'a, T::UnderlyingType, false> {
    //             &mut self.mut_fields[index_field]
    //         }
    //     }

    //     impl<
    //         'a,
    //         T: ComponentId,
    //         const NI: usize,
    //         const NM: usize,
    //         const Total: usize,
    //         const HALF_TOTAL: usize,
    //     > Fields<'a, T, NI, NM, Total, HALF_TOTAL>
    //     where
    //         [u64; HALF_TOTAL]: smallvec::Array,
    //         [u64; HALF_TOTAL]: smallvec::Array<Item = u64>,
    //         [u64; NI]: smallvec::Array<Item = u64>,
    //         [u64; NM]: smallvec::Array<Item = u64>,
    //     {
    //         unsafe fn to_initialized_array<Field, const NR: usize>(
    //             array: [MaybeUninit<Field>; NR],
    //         ) -> [Field; NR] {
    //             unsafe { array.as_ptr().cast::<[Field; NR]>().read() }
    //         }

    //         pub fn new(
    //             iter: &'a TableIter,
    //             immut_fields: [usize; NI],
    //             mut_fields: [usize; NM],
    //         ) -> Self {
    //             let mut fields_mut_array: [MaybeUninit<FieldMut<'_, T::UnderlyingType, false>>; NM] =
    //                 unsafe { MaybeUninit::uninit().assume_init() };

    //             for i in 0..NM {
    //                 fields_mut_array[i] = MaybeUninit::new(
    //                     iter.field_mut_lockless::<T>(mut_fields[i] as i8)
    //                         .expect("Field is not present or not correct type"),
    //                 );
    //             }

    //             let mut_fields = unsafe { Self::to_initialized_array(fields_mut_array) };

    //             let mut fields_immut_array: [MaybeUninit<Field<'_, T::UnderlyingType, false>>; NI] =
    //                 unsafe { MaybeUninit::uninit().assume_init() };
    //             for i in 0..NI {
    //                 fields_immut_array[i] = MaybeUninit::new(
    //                     iter.field_lockless::<T>(immut_fields[i] as i8)
    //                         .expect("Field is not present or not correct type"),
    //                 );
    //             }

    //             let immut_fields = unsafe { Self::to_initialized_array(fields_immut_array) };
    //             let mut immut_fields_table_ids = [0; NI];
    //             for i in 0..NI {
    //                 immut_fields_table_ids[i] = immut_fields[i].table_id();
    //             }

    //             let mut mut_fields_table_ids = [0; NM];
    //             for i in 0..NM {
    //                 mut_fields_table_ids[i] = mut_fields[i].table_id();
    //             }

    //             // check all table ids in the mutable and immutable fields and store the duplicates, make sure not to store duplicates twice
    //             // we have to check the table ids of the two arrays as well as itself the array
    //             let mut duplicate_table_ids = smallvec::SmallVec::<[u64; HALF_TOTAL]>::new();
    //             let mut has_any_tables_the_same = false;

    //             for i in 0..NI {
    //                 for j in (i + 1)..NI {
    //                     let immut_field_table_id = immut_fields_table_ids[i];
    //                     if immut_field_table_id == immut_fields_table_ids[j] {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&immut_field_table_id) {
    //                             duplicate_table_ids.push(immut_fields_table_ids[i]);
    //                         }
    //                     }
    //                 }
    //             }

    //             for i in 0..NM {
    //                 for j in (i + 1)..NM {
    //                     let mut_field_table_id = mut_fields_table_ids[i];
    //                     if mut_field_table_id == mut_fields_table_ids[j] {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&mut_field_table_id) {
    //                             duplicate_table_ids.push(mut_fields_table_ids[i]);
    //                         }
    //                     }
    //                 }
    //             }

    //             for i in 0..NI {
    //                 for j in 0..NM {
    //                     let immut_field_table_id = immut_fields_table_ids[i];
    //                     let mut_field_table_id = mut_fields_table_ids[j];
    //                     if immut_field_table_id == mut_field_table_id {
    //                         has_any_tables_the_same = true;
    //                         if !duplicate_table_ids.contains(&immut_field_table_id) {
    //                             duplicate_table_ids.push(immut_field_table_id);
    //                         }
    //                     }
    //                 }
    //             }

    //             let fields = Self {
    //                 immut_fields,
    //                 mut_fields,
    //                 has_any_tables_the_same,
    //                 duplicate_table_ids,
    //             };

    //             fields.lock_tables(&iter.world());

    //             fields
    //         }

    //         pub fn get<'w, F, Func, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             iter: &'w TableIter<'_, IS_RUN, P>,
    //             fields: &'w mut FIELDS,
    //             fields_indices: F,
    //             mut func: Func,
    //         ) where
    //             F: FieldsTuple2<T> + 'w,
    //             Func: FnMut(F::TupleType<'w>) + 'w,
    //         {
    //             let world = iter.world();
    //             let tuple = fields_indices.get_tuple(&world, iter, fields);
    //             func(tuple);
    //         }

    //         // pub fn get<'f, F: FieldsTuple<T>, Func: FnMut(F::TupleType<'f>)>(
    //         //     &self,
    //         //     fields: F,
    //         //     mut func: Func,
    //         // ) {
    //         //     // let tuple = F::create_tuple();
    //         //     //func(tuple);
    //         // }
    //     }

    //     #[crabtime::expression]
    //     fn fields(typename: String, fields_immut: Vec<usize>, fields_mut: Vec<usize>) {
    //         let fields_immut_str = fields_immut
    //             .iter()
    //             .map(|x| x.to_string())
    //             .collect::<Vec<_>>();

    //         let fields_mut_str = fields_mut.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    //         let size_immut = fields_immut.len();
    //         let size_mut = fields_mut.len();
    //         let size_total = size_immut + size_mut;
    //         let half_size_total = (size_total + 1) / 2;

    //         let arr_immut = fields_immut_str[0..size_immut].join(",");
    //         let arr_mut = fields_mut_str[0..size_mut].join(",");
    //         crabtime::output! {
    //            Fields::<{{typename}}, {{size_immut}}, {{size_mut}}, {{size_total}}, {{half_size_total}}>::new([{{arr_immut}}], [{{arr_mut}}])
    //         }
    //     }

    //     struct Mut(usize);

    //     impl<A, B, T> FieldsTuple2<T> for (A, B)
    //     where
    //         T: 'static + ComponentId,
    //         A: IterableTypeFieldOperation2<T>,
    //         B: IterableTypeFieldOperation2<T>,
    //     {
    //         type TupleType<'w>
    //             = (
    //             <A as IterableTypeFieldOperation2<T>>::ActualType<'w>,
    //             <B as IterableTypeFieldOperation2<T>>::ActualType<'w>,
    //         )
    //         where
    //             Self: 'w;

    //         fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             world: &WorldRef,
    //             iter: &TableIter<'_, IS_RUN, P>,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::TupleType<'a> {
    //             const {
    //                 // if FIELDS::TOTAL != 2 {
    //                 //     panic!("total indices should be {}", FIELDS::TOTAL);
    //                 // }
    //             }
    //             let duplicates = fields.duplicate_table_ids();
    //             let any_duplicates = !duplicates.is_empty();
    //             if !any_duplicates {
    //                 // self.0.lock_column(world);
    //                 // self.1.lock_column(world);
    //                 return (self.0.get_data(0, fields), self.1.get_data(1, fields));
    //             } else {
    //                 let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
    //                     smallvec::SmallVec::new();
    //                 let sources = iter.sources();
    //                 let entities = iter.entities();
    //                 let table_id = self.0.table_id(fields, 0);
    //                 let field_index = self.0.field_index(fields, 0);
    //                 let src_field_index = sources[field_index as usize];
    //                 let entity_id = if src_field_index == 0 {
    //                     entities[field_index as usize]
    //                 } else {
    //                     src_field_index
    //                 };

    //                 if duplicates.contains(&table_id) {
    //                     let was_locked = locked_entities.iter().any(|&id| id == entity_id);

    //                     if was_locked {
    //                         panic!("Entity already locked");
    //                     }
    //                     locked_entities.push(entity_id);
    //                 }
    //             }
    //             (self.0.get_data(0, fields), self.1.get_data(1, fields))
    //         }
    //     }

    //     pub trait FieldsTuple2<T: ComponentId>: Sized {
    //         type TupleType<'w>
    //         where
    //             Self: 'w;

    //         fn get_tuple<'a, const IS_RUN: bool, P: ComponentId, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             world: &WorldRef,
    //             iter: &TableIter<'_, IS_RUN, P>,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::TupleType<'a>;
    //     }

    //     pub trait IterableTypeFieldOperation2<T: 'static + ComponentId> {
    //         type ActualType<'w>
    //         where
    //             Self: 'w;
    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a;

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64;

    //         fn field_index<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> i8;
    //     }

    //     impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for usize {
    //         type ActualType<'w>
    //             = &'w T::UnderlyingType
    //         where
    //             Self: 'w;

    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a,
    //         {
    //             fields.get(index_field, self)
    //         }

    //         fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             fields: &'w mut FIELDS,
    //             index: usize,
    //         ) -> i8 {
    //             fields.get_field(index).field_index
    //         }

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64 {
    //             fields.get_field(index).table_id()
    //         }

    //         // fn lock_column(&self, world: &WorldRef) {
    //         //     get_table_column_lock_read_begin(
    //         //         world,
    //         //         self.0.table.as_ptr(),
    //         //         self.0.column_index,
    //         //         self.0.stage_id,
    //         //     );
    //         // }
    //     }

    //     impl<T: 'static + ComponentId> IterableTypeFieldOperation2<T> for Mut {
    //         type ActualType<'w>
    //             = &'w mut T::UnderlyingType
    //         where
    //             Self: 'w;

    //         fn get_data<'a, FIELDS: FieldContainer<'a, T>>(
    //             self,
    //             index_field: usize,
    //             fields: &'a mut FIELDS,
    //         ) -> Self::ActualType<'a>
    //         where
    //             Self: 'a,
    //         {
    //             fields.get_mut(index_field, self.0)
    //         }

    //         fn field_index<'w, FIELDS: FieldContainer<'w, T>>(
    //             &self,
    //             fields: &'w mut FIELDS,
    //             index: usize,
    //         ) -> i8 {
    //             fields.get_field_mut(index).field_index
    //         }

    //         fn table_id<'a, FIELDS: FieldContainer<'a, T>>(
    //             &self,
    //             fields: &'a mut FIELDS,
    //             index: usize,
    //         ) -> u64 {
    //             fields.get_field_mut(index).table_id()
    //         }

    //         // fn lock_column(&self, world: &WorldRef) {
    //         //     get_table_column_lock_write_begin(
    //         //         world,
    //         //         self.0.table.as_ptr(),
    //         //         self.0.column_index,
    //         //         self.0.stage_id,
    //         //     );
    //         // }
    //     }

    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////
    ///////////////////////////////////////////////////

    // Define the trait with appropriate lifetime bounds
    pub trait FieldsTuple<T>: Sized {
        type TupleType<'w>
        where
            Self: 'w;

        type ArrayDuplicateTables;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
        ) -> Self::TupleType<'a>;

        fn any_table_the_same(&self) -> Self::ArrayDuplicateTables;
    }

    pub trait IterableTypeFieldOperation<T: 'static> {
        type ActualType<'w>
        where
            Self: 'w;
        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a;

        fn table_id(&self) -> u64;

        fn field_index(&self) -> i8;

        fn lock_column(&self, world: &WorldRef);
    }

    impl<T: 'static> IterableTypeFieldOperation<T> for (&'_ Field<'_, T, false>, usize) {
        type ActualType<'w>
            = &'w T
        where
            Self: 'w;

        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            &self.0.slice_components[self.1]
        }

        fn field_index(&self) -> i8 {
            self.0.field_index
        }

        fn table_id(&self) -> u64 {
            unsafe { sys::ecs_rust_table_id(self.0.table.as_ptr()) }
        }

        fn lock_column(&self, world: &WorldRef) {
            get_table_column_lock_read_begin(
                world,
                self.0.table.as_ptr(),
                self.0.column_index,
                self.0.stage_id,
            );
        }
    }

    impl<T: 'static> IterableTypeFieldOperation<T> for (&'_ mut FieldMut<'_, T, false>, usize) {
        type ActualType<'w>
            = &'w mut T
        where
            Self: 'w;

        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            &mut self.0.slice_components[self.1]
        }

        fn field_index(&self) -> i8 {
            self.0.field_index
        }

        fn table_id(&self) -> u64 {
            unsafe { sys::ecs_rust_table_id(self.0.table.as_ptr()) }
        }

        fn lock_column(&self, world: &WorldRef) {
            get_table_column_lock_write_begin(
                world,
                self.0.table.as_ptr(),
                self.0.column_index,
                self.0.stage_id,
            );
        }
    }

    impl<A, B, T> FieldsTuple<T> for (A, B)
    where
        T: 'static,
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

        type ArrayDuplicateTables = (bool, [u64; 1]);

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
        ) -> Self::TupleType<'a> {
            let duplicates = self.any_table_the_same();
            if !duplicates.0 {
                self.0.lock_column(world);
                self.1.lock_column(world);
                return (self.0.get_data(), self.1.get_data());
            } else {
                let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
                    smallvec::SmallVec::new();
                let sources = iter.sources();
                let entities = iter.entities();
                let table_id = self.0.table_id();
                let field_index = self.0.field_index();
                let src_field_index = sources[field_index as usize];
                let entity_id = if src_field_index == 0 {
                    entities[field_index as usize]
                } else {
                    src_field_index
                };

                if duplicates.1.contains(&table_id) {
                    let was_locked = locked_entities.iter().any(|&id| id == entity_id);

                    if was_locked {
                        panic!("Entity already locked");
                    }
                    locked_entities.push(entity_id);
                }
            }
            (self.0.get_data(), self.1.get_data())
        }

        fn any_table_the_same(&self) -> Self::ArrayDuplicateTables {
            let has_duplicate = self.0.table_id() == self.1.table_id();
            if has_duplicate {
                return (true, [self.0.table_id()]);
            } else {
                return (false, [0]);
            }
        }
    }

    #[derive(Debug, Clone, Copy, Component)]
    struct Transform {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, Component)]
    struct Scale {
        x: f32,
    }
    impl<const IS_RUN: bool, P> TableIter<'_, IS_RUN, P>
    where
        P: ComponentId,
    {
        pub fn get<'a, T, F, Func>(&'a self, fields: F, mut func: Func)
        where
            F: FieldsTuple<T> + 'a,
            Func: FnMut(F::TupleType<'a>) + 'a,
        {
            let world = self.world();
            let tuple = fields.get_tuple(&world, self);
            func(tuple);
        }
    }

    let world = World::new();

    let parent = world.entity().set(Transform { x: 1.0, y: 2.0 });
    world
        .entity()
        .set(Transform { x: 3.0, y: 4.0 })
        .set(Scale { x: 2.0 })
        .child_of_id(parent);

    world
        .query::<(&Transform, &mut Transform)>()
        .term_at(0)
        .parent()
        .build()
        .run(|mut it| {
            while it.next() {
                let parent_transform = it.field_lockless::<Transform>(0).unwrap();
                let mut child_transform = it.field_mut_lockless::<Transform>(1).unwrap();
                for i in it.iter() {
                    it.get(
                        ((&parent_transform, i), (&mut child_transform, i)),
                        |(f1, f2)| {
                            println!("f1: {:?}", f1);
                            println!("f2: {:?}", f2);
                        },
                    );
                }
            }
        });
}

#[test]
fn test_something() {
    // Define the trait with appropriate lifetime bounds
    pub trait FieldsTuple<T>: Sized {
        type TupleType<'w>
        where
            Self: 'w;

        type ArrayDuplicateTables;

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
        ) -> Self::TupleType<'a>;

        fn any_table_the_same(&self) -> Self::ArrayDuplicateTables;
    }

    pub trait IterableTypeFieldOperation<T: 'static> {
        type ActualType<'w>
        where
            Self: 'w;
        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a;

        fn table_id(&self) -> u64;

        fn field_index(&self) -> i8;

        fn lock_column(&self, world: &WorldRef);
    }

    impl<T: 'static> IterableTypeFieldOperation<T> for (&'_ Field<'_, T, false>, usize) {
        type ActualType<'w>
            = &'w T
        where
            Self: 'w;

        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            &self.0.slice_components[self.1]
        }

        fn field_index(&self) -> i8 {
            self.0.field_index
        }

        fn table_id(&self) -> u64 {
            unsafe { sys::ecs_rust_table_id(self.0.table.as_ptr()) }
        }

        fn lock_column(&self, world: &WorldRef) {
            get_table_column_lock_read_begin(
                world,
                self.0.table.as_ptr(),
                self.0.column_index,
                self.0.stage_id,
            );
        }
    }

    impl<T: 'static> IterableTypeFieldOperation<T> for (&'_ mut FieldMut<'_, T, false>, usize) {
        type ActualType<'w>
            = &'w mut T
        where
            Self: 'w;

        fn get_data<'a>(self) -> Self::ActualType<'a>
        where
            Self: 'a,
        {
            &mut self.0.slice_components[self.1]
        }

        fn field_index(&self) -> i8 {
            self.0.field_index
        }

        fn table_id(&self) -> u64 {
            unsafe { sys::ecs_rust_table_id(self.0.table.as_ptr()) }
        }

        fn lock_column(&self, world: &WorldRef) {
            get_table_column_lock_write_begin(
                world,
                self.0.table.as_ptr(),
                self.0.column_index,
                self.0.stage_id,
            );
        }
    }

    impl<A, B, T> FieldsTuple<T> for (A, B)
    where
        T: 'static,
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

        type ArrayDuplicateTables = (bool, [u64; 1]);

        fn get_tuple<'a, const IS_RUN: bool, P: ComponentId>(
            self,
            world: &WorldRef,
            iter: &TableIter<'_, IS_RUN, P>,
        ) -> Self::TupleType<'a> {
            let duplicates = self.any_table_the_same();
            if !duplicates.0 {
                self.0.lock_column(world);
                self.1.lock_column(world);
                return (self.0.get_data(), self.1.get_data());
            } else {
                let mut locked_entities: smallvec::SmallVec<[Entity; 2]> =
                    smallvec::SmallVec::new();
                let sources = iter.sources();
                let entities = iter.entities();
                let table_id = self.0.table_id();
                let field_index = self.0.field_index();
                let src_field_index = sources[field_index as usize];
                let entity_id = if src_field_index == 0 {
                    entities[field_index as usize]
                } else {
                    src_field_index
                };

                if duplicates.1.contains(&table_id) {
                    let was_locked = locked_entities.iter().any(|&id| id == entity_id);

                    if was_locked {
                        panic!("Entity already locked");
                    }
                    locked_entities.push(entity_id);
                }
            }
            (self.0.get_data(), self.1.get_data())
        }

        fn any_table_the_same(&self) -> Self::ArrayDuplicateTables {
            let has_duplicate = self.0.table_id() == self.1.table_id();
            if has_duplicate {
                return (true, [self.0.table_id()]);
            } else {
                return (false, [0]);
            }
        }
    }

    #[derive(Debug, Clone, Copy, Component)]
    struct Transform {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, Component)]
    struct Scale {
        x: f32,
    }
    // impl<const IS_RUN: bool, P> TableIter<'_, IS_RUN, P>
    // where
    //     P: ComponentId,
    // {
    //     pub fn get<'a, T, F, Func>(&'a self, fields: F, mut func: Func)
    //     where
    //         F: FieldsTuple<T> + 'a,
    //         Func: FnMut(F::TupleType<'a>) + 'a,
    //     {
    //         let world = self.world();
    //         let tuple = fields.get_tuple(&world, self);
    //         func(tuple);
    //     }
    // }

    let world = World::new();

    let parent = world.entity().set(Transform { x: 1.0, y: 2.0 });
    world
        .entity()
        .set(Transform { x: 3.0, y: 4.0 })
        .set(Scale { x: 2.0 })
        .child_of_id(parent);

    world
        .query::<(&Transform, &mut Transform)>()
        .term_at(0)
        .parent()
        .build()
        .run(|mut it| {
            while it.next() {
                let parent_transform = it.field_lockless::<Transform>(0).unwrap();
                let mut child_transform = it.field_mut_lockless::<Transform>(1).unwrap();
                for i in it.iter() {
                    it.get(
                        ((&parent_transform, i), (&mut child_transform, i)),
                        |(f1, f2)| {
                            println!("f1: {:?}", f1);
                            println!("f2: {:?}", f2);
                        },
                    );
                }
            }
        });

    // #[derive(Clone, Copy)]
    // pub struct Fields<T> {
    //     pub(crate) _phantom: PhantomData<T>,
    // }

    // impl<T> Fields<T> {
    //     pub fn new() -> Self {
    //         Fields {
    //             _phantom: PhantomData,
    //         }
    //     }

    //     pub fn get<'a, F, Func>(&'a self, fields: F, mut func: Func)
    //     where
    //         F: FieldsTuple<T> + 'a,
    //         Func: FnMut(F::TupleType<'a>),
    //     {
    //         let tuple = fields.get_tuple();
    //         func(tuple);
    //     }
    // }
}

#[test]
fn test_something2() {
    pub trait FieldsTuple<'a, T>: Sized {
        type TupleType<'w>;

        fn get_tuple(self) -> Self::TupleType<'a>;
    }

    #[derive(Clone, Copy)]
    pub struct Fields<T> {
        pub(crate) _phantom: PhantomData<T>,
    }

    pub struct Field<'a, T> {
        array: &'a [T],
        pub(crate) _phantom: PhantomData<T>,
    }

    pub struct FieldMut<'a, T> {
        pub array: &'a mut [T],
        pub(crate) _phantom: PhantomData<T>,
    }

    pub trait IterableTypeFieldOperation<'a, T: 'static> {
        type ActualType<'w>;

        fn get_data(self) -> Self::ActualType<'a>;
    }

    impl<'a, T: 'static> IterableTypeFieldOperation<'a, T> for (&'a Field<'a, T>, usize) {
        type ActualType<'w> = &'w T;

        fn get_data(self) -> Self::ActualType<'a> {
            &self.0.array[self.1]
        }
    }
    impl<'a, T: 'static> IterableTypeFieldOperation<'a, T> for (&'a mut FieldMut<'a, T>, usize) {
        type ActualType<'w> = &'w mut T;

        fn get_data(self) -> Self::ActualType<'a> {
            &mut self.0.array[self.1]
        }
    }

    impl<'a, A, B, T> FieldsTuple<'a, T> for (A, B)
    where
        T: 'static,
        A: IterableTypeFieldOperation<'a, T>,
        B: IterableTypeFieldOperation<'a, T>,
    {
        type TupleType<'w> = (
            <A as IterableTypeFieldOperation<'a, T>>::ActualType<'w>,
            <B as IterableTypeFieldOperation<'a, T>>::ActualType<'w>,
        );

        fn get_tuple(self) -> Self::TupleType<'a> {
            (self.0.get_data(), self.1.get_data())
        }
    }

    impl<T> Fields<T> {
        pub fn new() -> Self {
            Fields {
                _phantom: PhantomData,
            }
        }

        // pub fn get<'w, F, Func>(&self, fields: &F, mut func: Func)
        // where
        //     F: FieldsTuple<'w, T>,
        //     Func: for<'b> FnMut(F::TupleType<'b>),
        // {
        //     let tuple = fields.get_tuple();
        //     func(tuple);
        // }
    }

    #[derive(Debug, Clone, Copy)]
    struct Transform {
        x: f32,
        y: f32,
    }

    let transform_array = [Transform { x: 1.0, y: 1.0 }; 10];
    let mut transform_array2 = [Transform { x: 3.0, y: 4.0 }; 10];
    let fields = Fields::<Transform>::new();
    let field = Field {
        array: &transform_array,
        _phantom: PhantomData,
    };
    let mut field2 = FieldMut {
        array: &mut transform_array2,
        _phantom: PhantomData,
    };

    // [0, 1, 2].iter().fold((&field, &mut field2), |(f1, f2), i| {
    //     fields.get(&((f1, *i as usize), (f2, *i as usize)), |(f1, f2)| {
    //         println!("f1: {:?}", f1);
    //         println!("f2: {:?}", f2);
    //     });
    //     (f1, f2)
    // });

    // fields.get(((&field, i), (&mut field2, i)), |(f1, f2)| {
    //     println!("f1: {:?}", f1);
    //     println!("f2: {:?}", f2);
    // });
}

// #[test]
// fn someing3() {
//     use std::marker::PhantomData;

//     // The two field types
//     pub struct Field<'a, T> {
//         pub array: &'a [T],
//         _phantom: PhantomData<T>,
//     }

//     pub struct FieldMut<'a, T> {
//         pub array: &'a mut [T],
//         _phantom: PhantomData<T>,
//     }

//     // A trait to “convert” a tuple of (field, index) pairs into the desired tuple of references.
//     pub trait FieldsTuple<'a>: Sized {
//         type TupleType<'w>
//         where
//             Self: 'w;

//         fn get_tuple(self) -> Self::TupleType<'a>;
//     }

//     // For our fixed case of one immutable and one mutable field:
//     impl<'a, T: 'static> FieldsTuple<'a>
//         for ((&'a Field<'a, T>, usize), (&'a mut FieldMut<'a, T>, usize))
//     {
//         type TupleType<'w>
//             = (&'a T, &'a mut T)
//         where
//             Self: 'w,
//             T: 'w;

//         // type TupleType<'w>
//         //     = (
//         //     <A as IterableTypeFieldOperation<'a>>::ActualType<'a>,
//         //     <B as IterableTypeFieldOperation<'a>>::ActualType<'a>,
//         // )
//         // where
//         //     A: 'w,
//         //     B: 'w;

//         fn get_tuple(self) -> Self::TupleType<'a> {
//             // Use the provided indices to index into the arrays
//             let (immut_pair, mut_pair) = self;
//             let (immut_field, immut_idx) = immut_pair;
//             let (mut_field, mut_idx) = mut_pair;
//             (&immut_field.array[immut_idx], &mut mut_field.array[mut_idx])
//         }
//     }

//     // The Fields type holds an array of immutable fields and an array of mutable fields.
//     // Here we use const generics for a fixed number of each.
//     pub struct Fields<'a, T, const NI: usize, const NM: usize> {
//         immut_fields: [Field<'a, T>; NI],
//         mut_fields: [FieldMut<'a, T>; NM],
//     }

//     impl<'a, T, const NI: usize, const NM: usize> Fields<'a, T, NI, NM> {
//         pub fn new(immut_fields: [Field<'a, T>; NI], mut_fields: [FieldMut<'a, T>; NM]) -> Self {
//             Self {
//                 immut_fields,
//                 mut_fields,
//             }
//         }

//         /// For this example (NI = 1, NM = 1) the indices array must have 2 elements:
//         /// the first for the immutable field and the second for the mutable field.
//         pub fn get<F>(&mut self, indices: &[usize]) -> F::TupleType<'a>
//         where
//             F: FieldsTuple<'a>,
//         {
//             // For NI = 1 and NM = 1 the split is straightforward.
//             let immut_idx = indices[0];
//             let mut_idx = indices[1];

//             // Build a tuple of pairs: one pair for the immutable field and one for the mutable field.
//             // (In a more general solution you would “zip” over the two arrays according to the provided indices.)
//             let tuple = (
//                 (&self.immut_fields[0], immut_idx),
//                 (&mut self.mut_fields[0], mut_idx),
//             );
//             tuple.get_tuple()
//         }
//     }

//     #[derive(Debug, Clone, Copy)]
//     struct Transform {
//         x: f32,
//         y: f32,
//     }

//     #[test]
//     fn test_something2() {
//         // Sample arrays for the two kinds of fields
//         let transform_array = [Transform { x: 1.0, y: 1.0 }; 10];
//         let mut transform_array2 = [Transform { x: 3.0, y: 4.0 }; 10];

//         // Construct one Field and one FieldMut.
//         let field = Field {
//             array: &transform_array,
//             _phantom: PhantomData,
//         };
//         let field_mut = FieldMut {
//             array: &mut transform_array2,
//             _phantom: PhantomData,
//         };

//         // Build our Fields container (for one immutable and one mutable field)
//         let mut fields = Fields::<Transform, 1, 1>::new([field], [field_mut]);

//         // Call `get` with an indices array whose length is NI + NM = 2.
//         // Here, the first index selects the element from the immutable field,
//         // and the second index selects the element from the mutable field.
//         let (f1, f2) = fields.get::<(
//             (&Field<Transform>, usize),
//             (&mut FieldMut<Transform>, usize),
//         )>([0, 1]);

//         println!("f1: {:?}", f1);
//         println!("f2: {:?}", f2);
//     }
// }

// #[test]
// fn test_something3() {
//     // Define the trait with appropriate lifetime bounds
//     pub trait FieldsTuple<'a>: Sized {
//         type TupleType<'w>
//         where
//             Self: 'w;

//         fn get_tuple(self) -> Self::TupleType<'a>;
//     }

//     pub struct Fields<'a, T, const NI: usize, const NM: usize, const Total: usize> {
//         pub(crate) immut_fields: [Field<'a, T>; NI],
//         mut_fields: [FieldMut<'a, T>; NM],
//     }

//     pub trait ConstSum<const NI: usize, const NM: usize> {
//         const TOTAL: usize = NI + NM;
//     }

//     impl<'a, T, const NI: usize, const NM: usize, const TOTAL: usize> Fields<'a, T, NI, NM, TOTAL> {
//         pub fn new(immut_fields: [Field<'a, T>; NI], mut_fields: [FieldMut<'a, T>; NM]) -> Self {
//             // let arr: [*mut c_void; TOTAL] = const { [core::ptr::null_mut(); ] };

//             Self {
//                 immut_fields,
//                 mut_fields,
//             }
//         }

//         pub fn get<F: FieldsTuple<'a> + 'a, Func: FnMut(F::TupleType<'a>) + 'a>(
//             &self,
//             fields: &[usize],
//             mut func: Func,
//         ) {
//             //let arr: [*mut c_void; fields.len()] = const { [core::ptr::null_mut(); TOTAL] };
//         }
//     }

//     pub struct Field<'a, T> {
//         array: &'a [T],
//         pub(crate) _phantom: PhantomData<T>,
//     }

//     pub struct FieldMut<'a, T> {
//         pub array: &'a mut [T],
//         pub(crate) _phantom: PhantomData<T>,
//     }

//     pub trait IterableTypeFieldOperation<'a> {
//         type ActualType<'w>;

//         fn get_data(self) -> Self::ActualType<'a>;
//     }

//     impl<'a, T: 'static> IterableTypeFieldOperation<'a> for (&'a Field<'a, T>, usize) {
//         type ActualType<'w> = &'w T;

//         fn get_data(self) -> Self::ActualType<'a> {
//             &self.0.array[self.1]
//         }
//     }
//     impl<'a, T: 'static> IterableTypeFieldOperation<'a> for (&'a mut FieldMut<'a, T>, usize) {
//         type ActualType<'w> = &'w mut T;

//         fn get_data(self) -> Self::ActualType<'a> {
//             &mut self.0.array[self.1]
//         }
//     }

//     impl<'a, A, B> FieldsTuple<'a> for (A, B)
//     where
//         A: IterableTypeFieldOperation<'a> + 'a,
//         B: IterableTypeFieldOperation<'a> + 'a,
//     {
//         type TupleType<'w>
//             = (
//             <A as IterableTypeFieldOperation<'a>>::ActualType<'a>,
//             <B as IterableTypeFieldOperation<'a>>::ActualType<'a>,
//         )
//         where
//             A: 'w,
//             B: 'w;

//         fn get_tuple(self) -> Self::TupleType<'a> {
//             (self.0.get_data(), self.1.get_data())
//         }
//     }

//     #[derive(Debug, Clone, Copy)]
//     struct Transform {
//         x: f32,
//         y: f32,
//     }

//     let transform_array = [Transform { x: 1.0, y: 1.0 }; 10];
//     let mut transform_array2 = [Transform { x: 3.0, y: 4.0 }; 10];

//     let field = Field {
//         array: &transform_array,
//         _phantom: PhantomData,
//     };
//     let mut field2 = FieldMut {
//         array: &mut transform_array2,
//         _phantom: PhantomData,
//     };
//     let fields = Fields::new([field], [field2]);

//     fields.get(&[0, 1], |(f1, f2)| {
//         println!("f1: {:?}", f1);
//         println!("f2: {:?}", f2);
//     });
// }
