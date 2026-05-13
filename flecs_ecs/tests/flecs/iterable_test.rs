#![allow(dead_code)]
#![allow(unused_imports)]
use crate::common_test::*;
use flecs_ecs::prelude::*;

// TODO: missing API: page_each — Query::page(skip, count) is not implemented in the Rust API.
// C++ test: q.page(1, 3).each([&](flecs::entity e, Self& self) { ... })
// #[test]
// fn iterable_page_each() {
//     let world = World::new();
//     let e1 = world.entity(); e1.set(SelfRef { value: e1.id() });
//     let e2 = world.entity(); e2.set(SelfRef { value: e2.id() });
//     let e3 = world.entity(); e3.set(SelfRef { value: e3.id() });
//     let e4 = world.entity(); e4.set(SelfRef { value: e4.id() });
//     let e5 = world.entity(); e5.set(SelfRef { value: e5.id() });
//     let q = world.new_query::<&SelfRef>();
//     let mut count = 0;
//     q.page(1, 3).each_entity(|e, self_| {
//         count += 1;
//         assert_ne!(e.id(), e1.id());
//         assert_ne!(e.id(), e5.id());
//         assert_eq!(e.id(), self_.value);
//     });
//     assert_eq!(count, 3);
// }

// TODO: missing API: page_iter — Query::page(skip, count).run() not implemented.
// #[test]
// fn iterable_page_iter() {
//     let world = World::new();
//     // ... same entities as above ...
//     let q = world.new_query::<&SelfRef>();
//     let mut count = 0;
//     q.page(1, 3).run(|mut it| {
//         while it.next() {
//             let self_ = it.field::<SelfRef>(0);
//             assert_eq!(it.count(), 3);
//             count += it.count() as i32;
//         }
//     });
//     assert_eq!(count, 3);
// }

// TODO: missing API: worker_each — Query::worker(index, total) is not implemented in the Rust API.
// C++ test: q.worker(0, 2).each([&](flecs::entity e, Self& self) { ... })
// #[test]
// fn iterable_worker_each() {
//     let world = World::new();
//     // ... same entities as above ...
//     let q = world.new_query::<&SelfRef>();
//     let mut count = 0;
//     q.worker(0, 2).each_entity(|e, self_| {
//         count += 1;
//         assert_ne!(e.id(), e4.id());
//         assert_ne!(e.id(), e5.id());
//         assert_eq!(e.id(), self_.value);
//     });
//     assert_eq!(count, 3);
// }

// TODO: missing API: worker_iter — Query::worker(index, total).run() not implemented.
// #[test]
// fn iterable_worker_iter() {
//     let world = World::new();
//     // ... same entities as above ...
//     let q = world.new_query::<&SelfRef>();
//     let mut count = 0;
//     q.worker(0, 2).run(|mut it| {
//         while it.next() {
//             assert_eq!(it.count(), 3);
//             count += it.count() as i32;
//         }
//     });
//     assert_eq!(count, 3);
//     count = 0;
//     q.worker(1, 2).run(|mut it| {
//         while it.next() {
//             assert_eq!(it.count(), 2);
//             count += it.count() as i32;
//         }
//     });
//     assert_eq!(count, 2);
// }
