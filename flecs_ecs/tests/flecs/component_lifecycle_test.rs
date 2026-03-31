//#![allow(dead_code)]
#![deny(dead_code)]
use crate::common_test::*;

#[test]
fn main_component_lifecycle() {
    // POD lifecycle
    ctor_on_add();
    dtor_on_remove();
    move_on_add();
    move_on_remove();
    copy_on_set();
    copy_on_override();
    set_singleton();
    set_pod_singleton();
    drop_on_remove();
    drop_on_world_delete();
    set_multiple_times();
    implicit_component();
    implicit_component_after_query();
    default_init();
    test_2_components_add_remove();

    // Struct with String lifecycle
    struct_w_string_add();
    struct_w_string_remove();
    struct_w_string_set();
    struct_w_string_override();
    struct_w_string_add_2_remove();
    struct_w_string_set_2_remove();
    struct_w_string_add_2_remove_w_tag();
    struct_w_string_set_2_remove_w_tag();
    non_trivial_implicit_move();

    // Struct with Vector lifecycle
    struct_w_vector_add();
    struct_w_vector_remove();
    struct_w_vector_set();
    struct_w_vector_override();
    struct_w_vector_add_2_remove();
    struct_w_vector_set_2_remove();
    struct_w_vector_add_2_remove_w_tag();
    struct_w_vector_set_2_remove_w_tag();

    // No Copy
    deleted_copy();

    // No Default
    no_default_ctor_invoked_set();
    no_default_set_deferred();
    no_default_ctor_set();
    grow_no_default_invoked();
    grow_no_default_invoked_w_tag();
    grow_no_default_invoked_w_component();
    delete_no_default_ctor();

    // Hooks
    on_add_hook();
    on_remove_hook();
    on_set_hook();
    on_add_hook_w_entity();
    on_remove_hook_w_entity();
    on_set_hook_w_entity();
    set_w_on_add();
    set_w_on_add_existing();
    on_replace_hook();

    // Sparse Hooks
    on_add_hook_sparse();
    on_remove_hook_sparse();
    on_set_hook_sparse();
    on_add_hook_sparse_w_entity();
    on_remove_hook_sparse_w_entity();
    on_set_hook_sparse_w_entity();

    // Chained Hooks
    chained_hooks();

    // Multiple Worlds
    ctor_w_2_worlds();
    ctor_w_2_worlds_explicit_registration();

    // No Copy Pair/Override Tests
    set_pair_w_entity_no_copy();
    set_pair_second_no_copy();
    set_override_no_copy();
    set_override_pair_no_copy();
    set_override_pair_w_entity_no_copy();

    // Deferred and Relation Destructor Tests
    dtor_after_defer_set();
    dtor_with_relation();
    dtor_relation_target();

    // Deferred Set Test
    defer_set();

    // No Copy Pair Tests
    set_pair_no_copy();

    // Sparse Component Tests
    sparse_component();

    // Count in Hook Tests
    count_in_add_hook();
    count_in_remove_hook();

    // Multiple Hooks Configuration
    set_multiple_hooks();
}

static POD_CTOR_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static POD_CLONE_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static POD_DROP_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

static STRUCT_W_STRING_CTOR_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static STRUCT_W_STRING_CLONE_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static STRUCT_W_STRING_DROP_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);

static STRUCT_W_VECTOR_CTOR_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static STRUCT_W_VECTOR_CLONE_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static STRUCT_W_VECTOR_DROP_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);

static NO_COPY_CTOR_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static NO_COPY_DROP_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

static NO_DEFAULT_CTOR_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static NO_DEFAULT_CLONE_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static NO_DEFAULT_DROP_INVOKED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

static NO_DEFAULT_INVOKED_CTOR_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static NO_DEFAULT_INVOKED_CLONE_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
static NO_DEFAULT_INVOKED_DROP_INVOKED: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);

fn reset_pod_counters() {
    POD_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    POD_CLONE_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    POD_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}
fn reset_struct_w_string_counters() {
    STRUCT_W_STRING_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    STRUCT_W_STRING_CLONE_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    STRUCT_W_STRING_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}

fn reset_struct_w_vector_counters() {
    STRUCT_W_VECTOR_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    STRUCT_W_VECTOR_CLONE_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    STRUCT_W_VECTOR_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}

fn reset_no_copy_counters() {
    NO_COPY_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    NO_COPY_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}

fn reset_no_default_counters() {
    NO_DEFAULT_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    NO_DEFAULT_CLONE_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    NO_DEFAULT_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}

fn reset_count_no_default_counters() {
    NO_DEFAULT_INVOKED_CTOR_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    NO_DEFAULT_INVOKED_CLONE_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
    NO_DEFAULT_INVOKED_DROP_INVOKED.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[track_caller]
fn test_pod_ctor(value: i32) {
    assert_eq!(
        POD_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "constructed count mismatch pod"
    );
}

#[track_caller]
fn test_pod_clone(value: i32) {
    assert_eq!(
        POD_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "cloned count mismatch pod"
    );
}

#[track_caller]
fn test_pod_drop(value: i32) {
    assert_eq!(
        POD_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "dropped count mismatch pod"
    );
}

#[track_caller]
fn test_string_ctor(value: i32) {
    assert_eq!(
        STRUCT_W_STRING_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "constructed count mismatch struct w/ string"
    );
}

#[track_caller]
fn test_string_clone(value: i32) {
    assert_eq!(
        STRUCT_W_STRING_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "cloned count mismatch struct w/ string"
    );
}

#[track_caller]
fn test_string_drop(value: i32) {
    assert_eq!(
        STRUCT_W_STRING_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "dropped count mismatch struct w/ string"
    );
}

#[track_caller]
fn test_vector_ctor(value: i32) {
    assert_eq!(
        STRUCT_W_VECTOR_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "constructed count mismatch struct w/ vector"
    );
}

#[track_caller]
fn test_vector_clone(value: i32) {
    assert_eq!(
        STRUCT_W_VECTOR_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "cloned count mismatch struct w/ vector"
    );
}

#[track_caller]
fn test_vector_drop(value: i32) {
    assert_eq!(
        STRUCT_W_VECTOR_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "dropped count mismatch struct w/ vector"
    );
}

#[track_caller]
fn test_no_default_invoked_ctor(value: i32) {
    assert_eq!(
        NO_DEFAULT_INVOKED_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "constructed count mismatch no_default_invoked"
    );
}

#[track_caller]
fn test_no_default_invoked_clone(value: i32) {
    assert_eq!(
        NO_DEFAULT_INVOKED_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "cloned count mismatch no_default_invoked"
    );
}

#[track_caller]
fn test_no_default_invoked_drop(value: i32) {
    assert_eq!(
        NO_DEFAULT_INVOKED_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "dropped count mismatch no_default_invoked"
    );
}

#[track_caller]
fn test_no_default_ctor(value: i32) {
    assert_eq!(
        NO_DEFAULT_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "constructed count mismatch no_default"
    );
}

#[track_caller]
fn test_no_default_clone(value: i32) {
    assert_eq!(
        NO_DEFAULT_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "cloned count mismatch no_default"
    );
}

#[track_caller]
fn test_no_default_drop(value: i32) {
    assert_eq!(
        NO_DEFAULT_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
        value,
        "dropped count mismatch no_default"
    );
}

struct WorldGuard {
    world: core::mem::ManuallyDrop<World>,
}

#[track_caller]
fn assert_lifecycle_counts() {
    let ctor_count = POD_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let clone_count = POD_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let drop_count = POD_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        ctor_count + clone_count,
        drop_count,
        "lifecycle counts do not match pod: ctor {} + clone {} != drop {}",
        ctor_count,
        clone_count,
        drop_count
    );

    let str_ctor_count = STRUCT_W_STRING_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let str_clone_count = STRUCT_W_STRING_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let str_drop_count = STRUCT_W_STRING_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        str_ctor_count + str_clone_count,
        str_drop_count,
        "lifecycle counts do not match struct w/ string: ctor {} + clone {} != drop {}",
        str_ctor_count,
        str_clone_count,
        str_drop_count
    );

    let vec_ctor_count = STRUCT_W_VECTOR_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let vec_clone_count = STRUCT_W_VECTOR_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let vec_drop_count = STRUCT_W_VECTOR_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        vec_ctor_count + vec_clone_count,
        vec_drop_count,
        "lifecycle counts do not match struct w/ vector: ctor {} + clone {} != drop {}",
        vec_ctor_count,
        vec_clone_count,
        vec_drop_count
    );

    let no_copy_ctor_count = NO_COPY_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let no_copy_drop_count = NO_COPY_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        no_copy_ctor_count, no_copy_drop_count,
        "lifecycle counts do not match no_copy: ctor {} != drop {}",
        no_copy_ctor_count, no_copy_drop_count
    );

    let no_default_ctor_count = NO_DEFAULT_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let no_default_clone_count = NO_DEFAULT_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let no_default_drop_count = NO_DEFAULT_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        no_default_ctor_count + no_default_clone_count,
        no_default_drop_count,
        "lifecycle counts do not match no_default: ctor {} + clone {} != drop {}",
        no_default_ctor_count,
        no_default_clone_count,
        no_default_drop_count
    );

    let count_no_default_ctor_count =
        NO_DEFAULT_INVOKED_CTOR_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let count_no_default_clone_count =
        NO_DEFAULT_INVOKED_CLONE_INVOKED.load(std::sync::atomic::Ordering::SeqCst);
    let count_no_default_drop_count =
        NO_DEFAULT_INVOKED_DROP_INVOKED.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        count_no_default_ctor_count + count_no_default_clone_count,
        count_no_default_drop_count,
        "lifecycle counts do not match count_no_default: ctor {} + clone {} != drop {}",
        count_no_default_ctor_count,
        count_no_default_clone_count,
        count_no_default_drop_count
    );
}

impl Drop for WorldGuard {
    #[track_caller]
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.world);
        }
        assert_lifecycle_counts();
    }
}

impl std::ops::Deref for WorldGuard {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

fn world_new() -> WorldGuard {
    reset_pod_counters();
    reset_struct_w_string_counters();
    reset_struct_w_vector_counters();
    reset_no_copy_counters();
    reset_no_default_counters();
    reset_count_no_default_counters();
    WorldGuard {
        world: core::mem::ManuallyDrop::new(World::new()),
    }
}

#[derive(Component)]
pub struct PodDefaultCloneDrop {
    pub value: i32,
}

impl Default for PodDefaultCloneDrop {
    fn default() -> Self {
        POD_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        PodDefaultCloneDrop { value: 10 }
    }
}

impl PodDefaultCloneDrop {
    #[allow(dead_code)]
    pub fn new(value: i32) -> Self {
        POD_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        PodDefaultCloneDrop { value }
    }
}

impl Clone for PodDefaultCloneDrop {
    fn clone(&self) -> Self {
        POD_CLONE_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        PodDefaultCloneDrop { value: self.value }
    }
}

impl Drop for PodDefaultCloneDrop {
    fn drop(&mut self) {
        POD_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Component)]
pub struct StructWithString {
    value: String,
}

impl StructWithString {
    pub fn new(value: &str) -> Self {
        STRUCT_W_STRING_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithString {
            value: value.to_string(),
        }
    }
}

impl Default for StructWithString {
    fn default() -> Self {
        STRUCT_W_STRING_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithString {
            value: String::new(),
        }
    }
}

impl Clone for StructWithString {
    fn clone(&self) -> Self {
        STRUCT_W_STRING_CLONE_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithString {
            value: self.value.clone(),
        }
    }
}

impl Drop for StructWithString {
    fn drop(&mut self) {
        STRUCT_W_STRING_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Component)]
pub struct StructWithVector {
    value: Vec<i32>,
}

impl StructWithVector {
    pub fn new(value: &[i32]) -> Self {
        STRUCT_W_VECTOR_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithVector {
            value: value.to_vec(),
        }
    }
}

impl Default for StructWithVector {
    fn default() -> Self {
        STRUCT_W_VECTOR_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithVector {
            value: Vec::default(),
        }
    }
}

impl Clone for StructWithVector {
    fn clone(&self) -> Self {
        STRUCT_W_VECTOR_CLONE_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        StructWithVector {
            value: self.value.clone(),
        }
    }
}

impl Drop for StructWithVector {
    fn drop(&mut self) {
        STRUCT_W_VECTOR_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Component)]
pub struct NoCopy {
    #[allow(dead_code)]
    value: i32,
}

impl Default for NoCopy {
    fn default() -> Self {
        NO_COPY_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoCopy { value: 10 }
    }
}

impl NoCopy {
    #[allow(dead_code)]
    pub fn new(value: i32) -> Self {
        NO_COPY_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoCopy { value }
    }
}

impl Drop for NoCopy {
    fn drop(&mut self) {
        NO_COPY_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Component)]
pub struct NoDefault {
    #[allow(dead_code)]
    value: i32,
}

impl NoDefault {
    #[allow(dead_code)]
    pub fn new(value: i32) -> Self {
        println!("no default ctor invoked");
        NO_DEFAULT_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoDefault { value }
    }
}

impl Clone for NoDefault {
    fn clone(&self) -> Self {
        NO_DEFAULT_CLONE_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoDefault {
            value: self.value.clone(),
        }
    }
}

impl Drop for NoDefault {
    fn drop(&mut self) {
        println!("no default drop invoked");
        NO_DEFAULT_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Component)]
pub struct NoDefaultInvoked {
    #[allow(dead_code)]
    value: i32,
}

impl NoDefaultInvoked {
    #[allow(dead_code)]
    pub fn new(value: i32) -> Self {
        NO_DEFAULT_INVOKED_CTOR_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoDefaultInvoked { value }
    }
}

impl Clone for NoDefaultInvoked {
    fn clone(&self) -> Self {
        NO_DEFAULT_INVOKED_CLONE_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        NoDefaultInvoked {
            value: self.value.clone(),
        }
    }
}

impl Drop for NoDefaultInvoked {
    fn drop(&mut self) {
        NO_DEFAULT_INVOKED_DROP_INVOKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

fn test_2_components_add_remove() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();
    world.component::<StructWithString>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    let _e2 = world.entity().add(PodDefaultCloneDrop::id());
    let e3 = world.entity().add(PodDefaultCloneDrop::id());

    test_pod_ctor(3);
    test_pod_clone(0);
    test_pod_drop(0);

    e.add(StructWithString::id());

    // e3 is after e in the Pod, StructWithString archetype table.
    e3.add(StructWithString::id());

    test_pod_ctor(3);
    test_pod_clone(0);
    test_pod_drop(0);
    test_string_ctor(2);
    test_string_clone(0);
    test_string_drop(0);

    e.remove(PodDefaultCloneDrop::id());

    // e3 String moves into e String, e moves back to the old table.
    // This should have no dtors because String hasn't actually been deleted anywhere.

    test_string_ctor(2);
    test_string_clone(0);
    test_string_drop(0);

    test_pod_ctor(3);
    test_pod_clone(0);
    test_pod_drop(1);

    drop(world);
    test_pod_ctor(3);
    test_pod_drop(3);
    test_pod_clone(0);
    test_string_ctor(2);
    test_string_drop(2);
    test_string_clone(0);
}

fn ctor_on_add() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(PodDefaultCloneDrop::id()));

    e.get::<Option<&PodDefaultCloneDrop>>(|pod| {
        assert!(pod.is_some());
        test_pod_ctor(1);
        test_pod_clone(0);
        test_pod_drop(0);

        assert_eq!(pod.unwrap().value, 10);
    });

    e.destruct();
    drop(world);
    test_pod_drop(1);
}
fn dtor_on_remove() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(PodDefaultCloneDrop::id()));
    test_pod_ctor(1);

    e.remove(PodDefaultCloneDrop::id());
    assert!(!e.has(PodDefaultCloneDrop::id()));

    test_pod_drop(1);
    test_pod_ctor(1);
    test_pod_clone(0);

    e.remove(PodDefaultCloneDrop::id());
    e.destruct();
    drop(world);
    test_pod_drop(1);
}
fn move_on_add() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.add(Position::id());
    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.destruct();
    drop(world);
    test_pod_drop(1);
    test_pod_ctor(1);
    test_pod_clone(0);
}
fn move_on_remove() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world
        .entity()
        .add(PodDefaultCloneDrop::id())
        .add(Position::id());
    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.remove(Position::id());
    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.destruct();
    drop(world);
    test_pod_drop(1);
    test_pod_ctor(1);
    test_pod_clone(0);
}

fn copy_on_set() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world.entity().set(PodDefaultCloneDrop::new(42));
    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.get::<&PodDefaultCloneDrop>(|pod| {
        assert_eq!(pod.value, 42);
    });

    drop(world);
    test_pod_ctor(1);
    test_pod_drop(1);
    test_pod_clone(0);
}

fn copy_on_override() {
    let world = world_new();
    world
        .component::<PodDefaultCloneDrop>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(PodDefaultCloneDrop::new(100));

    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    let e = world.entity().is_a(base);

    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(0);

    e.add(PodDefaultCloneDrop::id());

    test_pod_ctor(1);
    test_pod_drop(0);
    test_pod_clone(1);

    e.get::<&PodDefaultCloneDrop>(|pod| {
        assert_eq!(pod.value, 100);
    });

    drop(world);
    test_pod_ctor(1);
    test_pod_drop(2);
    test_pod_clone(1);
}

///////
///
///
fn drop_on_remove() {
    let world = world_new();

    world
        .entity_named("object 1")
        .set(PodDefaultCloneDrop::new(1));
    world
        .entity_named("object 2")
        .set(PodDefaultCloneDrop::new(2));
    world
        .entity_named("object 3")
        .set(PodDefaultCloneDrop::new(3));

    world.defer_begin();
    world
        .query::<&PodDefaultCloneDrop>()
        .build()
        .each_entity(|ent, _| {
            ent.each_component(|e| {
                ent.remove(e);
            });
        });
    world.defer_end();

    test_pod_drop(3);
}

fn set_singleton() {
    {
        let world = world_new();

        world
            .component::<PodDefaultCloneDrop>()
            .add_trait::<flecs::Singleton>();

        world.system::<()>().run(|it| {
            let world = it.world();
            world.set(PodDefaultCloneDrop::new(1));
        });

        world.progress();
    }

    // test_pod_ctor(3);
    // test_pod_drop(3);
    // test_pod_clone(0);
}

fn drop_on_world_delete() {
    {
        let world = world_new();

        world
            .entity_named("object 1")
            .set(PodDefaultCloneDrop::new(1));
        world
            .entity_named("object 2")
            .set(PodDefaultCloneDrop::new(2));
        world
            .entity_named("object 3")
            .set(PodDefaultCloneDrop::new(3));

        world.quit();

        world.progress();
    }

    test_pod_ctor(3);
    test_pod_drop(3);
    test_pod_clone(0);
}

fn set_multiple_times() {
    let world = world_new();

    let ent = world
        .entity_named("object 1")
        .set(PodDefaultCloneDrop::new(1));
    ent.set(PodDefaultCloneDrop::new(2));
    ent.set(PodDefaultCloneDrop::new(3));
    ent.destruct();

    test_pod_ctor(3);
    test_pod_drop(3);
    test_pod_clone(0);
}

//////

fn struct_w_string_add() {
    let world = world_new();
    world.component::<StructWithString>();

    let e = world.entity().add(StructWithString::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithString::id()));

    e.get::<&StructWithString>(|str_comp| {
        assert_eq!(str_comp.value, "");
    });
}

fn struct_w_string_remove() {
    let world = world_new();
    world.component::<StructWithString>();

    let e = world.entity().add(StructWithString::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithString::id()));

    e.remove(StructWithString::id());
    assert!(!e.has(StructWithString::id()));
}

fn struct_w_string_set() {
    let world = world_new();
    world.component::<StructWithString>();

    let e = world.entity().set(StructWithString::new("Hello World"));
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithString::id()));

    e.get::<&StructWithString>(|str_comp| {
        assert_eq!(str_comp.value, "Hello World");
    });
}

fn struct_w_string_override() {
    let world = world_new();

    world
        .component::<StructWithString>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(StructWithString::new("Hello World"));
    assert_ne!(base.id(), 0);

    let e = world.entity().is_a(base);
    assert_ne!(e.id(), 0);

    e.add(StructWithString::id());

    e.get::<&StructWithString>(|str_comp| {
        assert_eq!(str_comp.value, "Hello World");
    });
}

fn struct_w_string_add_2_remove() {
    let world = world_new();
    world.component::<StructWithString>();

    let e1 = world.entity().add(StructWithString::id());
    let e2 = world.entity().add(StructWithString::id());

    e1.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "");
    });
    e2.get::<&StructWithString>(|str2| {
        assert_eq!(str2.value, "");
    });

    e1.remove(StructWithString::id());
    e1.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithString>(|str2| {
        assert_eq!(str2.value, "");
    });

    e2.remove(StructWithString::id());
    e2.get::<Option<&StructWithString>>(|str2| {
        assert!(str2.is_none());
    });
}

fn struct_w_string_set_2_remove() {
    let world = world_new();
    world.component::<StructWithString>();

    let e1 = world.entity().set(StructWithString::new("hello"));
    let e2 = world.entity().set(StructWithString::new("world"));

    e1.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "hello");
    });
    e2.get::<&StructWithString>(|str2| {
        assert_eq!(str2.value, "world");
    });

    e1.remove(StructWithString::id());
    e1.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithString>(|str2| {
        assert_eq!(str2.value, "world");
    });

    e2.remove(StructWithString::id());
    e2.get::<Option<&StructWithString>>(|str2| {
        assert!(str2.is_none());
    });
}

fn struct_w_string_add_2_remove_w_tag() {
    let world = world_new();
    world.component::<StructWithString>();

    let e1 = world.entity().add(Tag).add(StructWithString::id());
    let e2 = world.entity().add(Tag).add(StructWithString::id());

    e1.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "");
    });

    e2.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "");
    });

    e1.remove(StructWithString::id());

    e1.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "");
    });

    e2.remove(StructWithString::id());

    e2.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });
}

fn struct_w_string_set_2_remove_w_tag() {
    let world = world_new();
    world.component::<StructWithString>();

    let e1 = world.entity().add(Tag).set(StructWithString::new("hello"));
    let e2 = world.entity().add(Tag).set(StructWithString::new("world"));

    e1.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "hello");
    });

    e2.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "world");
    });

    e1.remove(StructWithString::id());

    e1.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithString>(|str1| {
        assert_eq!(str1.value, "world");
    });

    e2.remove(StructWithString::id());

    e2.get::<Option<&StructWithString>>(|str1| {
        assert!(str1.is_none());
    });
}

fn struct_w_vector_add() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e = world.entity().add(StructWithVector::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithVector::id()));

    e.get::<&StructWithVector>(|str_comp| {
        assert_eq!(str_comp.value, Vec::default());
    });
}

fn struct_w_vector_remove() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e = world.entity().add(StructWithVector::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithVector::id()));

    e.remove(StructWithVector::id());
    assert!(!e.has(StructWithVector::id()));
}

fn struct_w_vector_set() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e = world.entity().set(StructWithVector::new(&[1, 2]));
    assert_ne!(e.id(), 0);
    assert!(e.has(StructWithVector::id()));

    e.get::<&StructWithVector>(|str_comp| {
        assert_eq!(str_comp.value, vec![1, 2]);
    });
}

fn struct_w_vector_override() {
    let world = world_new();

    world
        .component::<StructWithVector>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(StructWithVector::new(&[1, 2]));
    assert_ne!(base.id(), 0);

    let e = world.entity().is_a(base);
    assert_ne!(e.id(), 0);

    e.add(StructWithVector::id());

    e.get::<&StructWithVector>(|str_comp| {
        assert_eq!(str_comp.value, vec![1, 2]);
    });
}

fn struct_w_vector_add_2_remove() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e1 = world.entity().add(StructWithVector::id());
    let e2 = world.entity().add(StructWithVector::id());

    e1.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, Vec::new());
    });
    e2.get::<&StructWithVector>(|str2| {
        assert_eq!(str2.value, Vec::new());
    });

    e1.remove(StructWithVector::id());
    e1.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithVector>(|str2| {
        assert_eq!(str2.value, Vec::new());
    });

    e2.remove(StructWithVector::id());
    e2.get::<Option<&StructWithVector>>(|str2| {
        assert!(str2.is_none());
    });
}

fn struct_w_vector_set_2_remove() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e1 = world.entity().set(StructWithVector::new(&[1, 2]));
    let e2 = world.entity().set(StructWithVector::new(&[3, 4]));

    e1.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, vec![1, 2]);
    });
    e2.get::<&StructWithVector>(|str2| {
        assert_eq!(str2.value, vec![3, 4]);
    });

    e1.remove(StructWithVector::id());
    e1.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithVector>(|str2| {
        assert_eq!(str2.value, vec![3, 4]);
    });

    e2.remove(StructWithVector::id());
    e2.get::<Option<&StructWithVector>>(|str2| {
        assert!(str2.is_none());
    });
}

fn struct_w_vector_add_2_remove_w_tag() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e1 = world.entity().add(Tag).add(StructWithVector::id());
    let e2 = world.entity().add(Tag).add(StructWithVector::id());

    e1.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, Vec::new());
    });

    e2.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, Vec::new());
    });

    e1.remove(StructWithVector::id());

    e1.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, Vec::new());
    });

    e2.remove(StructWithVector::id());

    e2.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });
}

fn struct_w_vector_set_2_remove_w_tag() {
    let world = world_new();
    world.component::<StructWithVector>();

    let e1 = world.entity().add(Tag).set(StructWithVector::new(&[1, 2]));
    let e2 = world.entity().add(Tag).set(StructWithVector::new(&[3, 4]));

    e1.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, vec![1, 2]);
    });

    e2.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, vec![3, 4]);
    });

    e1.remove(StructWithVector::id());

    e1.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });

    e2.get::<&StructWithVector>(|str1| {
        assert_eq!(str1.value, vec![3, 4]);
    });

    e2.remove(StructWithVector::id());

    e2.get::<Option<&StructWithVector>>(|str1| {
        assert!(str1.is_none());
    });

    drop(world);
    test_vector_ctor(2);
    test_vector_clone(0);
    test_vector_drop(2);
}

fn implicit_component() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(PodDefaultCloneDrop::id()));

    e.get::<Option<&PodDefaultCloneDrop>>(|pod| {
        assert!(pod.is_some());
        test_pod_ctor(1);
        test_pod_clone(0);
        test_pod_drop(0);

        assert_eq!(pod.unwrap().value, 10);
    });

    world.entity().add(PodDefaultCloneDrop::id());
    test_pod_ctor(2);
    test_pod_clone(0);
    test_pod_drop(0);

    world.entity().add(PodDefaultCloneDrop::id());
    test_pod_ctor(3);
    test_pod_clone(0);
    test_pod_drop(0);
}

fn implicit_component_after_query() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    world.new_query::<&PodDefaultCloneDrop>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(PodDefaultCloneDrop::id()));

    e.get::<Option<&PodDefaultCloneDrop>>(|pod| {
        assert!(pod.is_some());
        test_pod_ctor(1);
        test_pod_clone(0);
        test_pod_drop(0);

        assert_eq!(pod.unwrap().value, 10);
    });

    world.entity().add(PodDefaultCloneDrop::id());
    test_pod_ctor(2);
    test_pod_clone(0);
    test_pod_drop(0);

    world.entity().add(PodDefaultCloneDrop::id());
    test_pod_ctor(3);
    test_pod_clone(0);
    test_pod_drop(0);
}

fn try_add<T: ComponentId>(world: &WorldGuard) {
    let c = world.component::<T>();
    let e = world.entity().add(c);
    assert!(e.has(T::id()));
    e.remove(T::id());
    assert!(!e.has(T::id()));
}

fn try_set<T: ComponentId>(world: &WorldGuard, val: T) {
    let e = world.entity().set(val);
    assert!(e.has(T::id()));
}

#[allow(dead_code)]
fn try_set_default<T: ComponentId + Default>(world: &WorldGuard) {
    let e = world.entity().set(T::default());
    assert!(e.has(T::id()));
}

fn deleted_copy() {
    let world = world_new();

    world.component::<NoCopy>();

    try_add::<NoCopy>(&world);
    try_set::<NoCopy>(&world, NoCopy::default());
}

fn default_init() {
    let world = world_new();

    world.component::<PodDefaultCloneDrop>();

    try_add::<PodDefaultCloneDrop>(&world);
    try_set::<PodDefaultCloneDrop>(&world, PodDefaultCloneDrop::default());
}

#[test]
#[should_panic]
fn no_default_ctor_add() {
    let world = World::new();

    world.component::<&NoDefault>();
    let c = world.component::<NoDefault>();
    let e = world.entity().add(c);
    assert!(e.has(NoDefault::id()));
    e.remove(NoDefault::id());
    assert!(!e.has(NoDefault::id()));
}

#[test]
#[should_panic]
fn no_default_ctor_add_relationship() {
    let world = World::new();

    world.component::<&NoDefault>();
    let obj = world.entity();
    let e = world.entity().add((NoDefault::id(), obj.id()));
    assert!(e.has((NoDefault::id(), flecs::Wildcard::ID)));
    e.remove((NoDefault::id(), flecs::Wildcard::ID));
    assert!(!e.has((NoDefault::id(), flecs::Wildcard::ID)));
}

#[test]
#[should_panic]
fn no_default_ctor_add_second() {
    let world = World::new();

    world.component::<&NoDefault>();
    let obj = world.entity();
    let c = world.component::<NoDefault>();
    let e = world.entity().add((obj.id(), c));
    assert!(e.has((flecs::Wildcard::ID, NoDefault::id())));
    e.remove((flecs::Wildcard::ID, NoDefault::id()));
    assert!(!e.has((flecs::Wildcard::ID, NoDefault::id())));
}

fn no_default_ctor_set() {
    let world = world_new();

    world.component::<&NoDefault>();
    try_set::<NoDefault>(&world, NoDefault::new(1));
}

fn no_default_ctor_invoked_set() {
    let world = world_new();

    world.component::<&NoDefaultInvoked>();

    try_set::<NoDefaultInvoked>(&world, NoDefaultInvoked::new(1));
    test_no_default_invoked_ctor(1);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
    try_set::<NoDefaultInvoked>(&world, NoDefaultInvoked::new(1));
    test_no_default_invoked_ctor(2);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
}

fn no_default_set_deferred() {
    let world = world_new();

    world.component::<&NoDefault>();

    world.defer_begin();
    world.entity().set(NoDefault::new(1));
    test_no_default_ctor(1);
    test_no_default_clone(0);
    test_no_default_drop(0);
    println!("defer ended");
    world.defer_end();
    world.entity().set(NoDefault::new(1));
    //try_set::<NoDefault>(&world, NoDefault::new(1));
    test_no_default_ctor(2);
    test_no_default_clone(0);
    test_no_default_drop(0);
    println!("world end");
}

fn set_pod_singleton() {
    let world = world_new();

    world
        .component::<&PodDefaultCloneDrop>()
        .add_trait::<flecs::Singleton>();

    world.set(PodDefaultCloneDrop::new(3));
    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);

    world.get::<&PodDefaultCloneDrop>(|pod| {
        assert_eq!(pod.value, 3);
    });

    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);
}

fn non_trivial_implicit_move() {
    let world = world_new();
    world.component::<StructWithString>();

    let e = world.entity().set(StructWithString::new("hello"));
    let e2 = world.entity().set(StructWithString::new("world"));
    assert!(e.has(StructWithString::id()));
    assert!(e2.has(StructWithString::id()));
    test_string_ctor(2);
    test_string_clone(0);
    test_string_drop(0);

    e.destruct();
    test_string_ctor(2);
    test_string_clone(0);
    test_string_drop(1);
}

fn grow_no_default_invoked() {
    let world = world_new();
    world.component::<NoDefaultInvoked>();

    let e = world.entity().set(NoDefaultInvoked::new(1));
    let e2 = world.entity().set(NoDefaultInvoked::new(2));
    assert!(e.has(NoDefaultInvoked::id()));
    assert!(e2.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(2);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    let e3 = world.entity().set(NoDefaultInvoked::new(3));
    assert!(e3.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    e.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 1));
    e2.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 2));
    e3.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 3));
}

fn grow_no_default_invoked_w_tag() {
    let world = world_new();
    world.component::<NoDefaultInvoked>();

    let e = world.entity().set(NoDefaultInvoked::new(1));
    let e2 = world.entity().set(NoDefaultInvoked::new(2));
    assert!(e.has(NoDefaultInvoked::id()));
    assert!(e2.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(2);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    let e3 = world.entity().set(NoDefaultInvoked::new(3));
    assert!(e3.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    e.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 1));
    e2.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 2));
    e3.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 3));

    e.add(Tag);
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
    e2.add(Tag);
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
    e3.add(Tag);
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
}

fn grow_no_default_invoked_w_component() {
    let world = world_new();
    world.component::<NoDefaultInvoked>();

    let e = world.entity().set(NoDefaultInvoked::new(1));
    let e2 = world.entity().set(NoDefaultInvoked::new(2));
    assert!(e.has(NoDefaultInvoked::id()));
    assert!(e2.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(2);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    let e3 = world.entity().set(NoDefaultInvoked::new(3));
    assert!(e3.has(NoDefaultInvoked::id()));
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);

    e.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 1));
    e2.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 2));
    e3.get::<&NoDefaultInvoked>(|val| assert_eq!(val.value, 3));

    e.add(Position::id());
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
    e2.add(Position::id());
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
    e3.add(Position::id());
    test_no_default_invoked_ctor(3);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(0);
}

fn delete_no_default_ctor() {
    let world = world_new();

    world.component::<NoDefault>();

    let e1 = world.entity().set(NoDefault::new(1));
    let e2 = world.entity().set(NoDefault::new(2));
    let e3 = world.entity().set(NoDefault::new(3));

    test_no_default_ctor(3);
    test_no_default_clone(0);
    test_no_default_drop(0);

    e1.get::<&NoDefault>(|val| assert_eq!(val.value, 1));
    e2.get::<&NoDefault>(|val| assert_eq!(val.value, 2));
    e3.get::<&NoDefault>(|val| assert_eq!(val.value, 3));

    e2.destruct();
    test_no_default_ctor(3);
    test_no_default_clone(0);
    test_no_default_drop(1);
}

fn on_add_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_add(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    let e = world.entity().set(Position { x: 1, y: 2 });

    assert_eq!(world.cloned::<&Count>().0, 1);

    e.add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    world.new_query::<&Position>().each_entity(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 2);
}

fn on_remove_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_remove(|e, _| {
        e.world().set(Count(e.world().count(Position::id())));
    });

    let entity = world.entity().set(Position { x: 1, y: 2 });
    assert_eq!(world.cloned::<&Count>().0, 0);

    entity.remove(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    world.set(Count(0));

    world.new_query::<&Position>().each_entity(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    entity.set(Position { x: 3, y: 4 });
    entity.destruct();
    assert_eq!(world.cloned::<&Count>().0, 1);
}

fn on_set_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_set(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.set(Position { x: 10, y: 20 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    let v = e1.cloned::<&Position>();
    assert_eq!(v.x, 10);
    assert_eq!(v.y, 20);

    let e2 = world.entity().set(Position { x: 30, y: 40 });
    assert_eq!(world.cloned::<&Count>().0, 2);
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 30);
    assert_eq!(v.y, 40);
}

fn on_add_hook_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().on_add(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);
    assert_eq!(e_arg.get(), 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), *e1.id());

    e1.add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    let e2 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 2);
    assert_eq!(e_arg.get(), *e2.id());
}

fn on_remove_hook_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().on_remove(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);
    assert_eq!(e_arg.get(), 0);

    let e1 = world.entity().add(Position::id());
    let e2 = world.entity().add(Position::id());
    let e1_id = *e1.id();
    let e2_id = *e2.id();
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.remove(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), e1_id);

    // e2 will be removed when world is dropped
    drop(world);
    assert_eq!(e_arg.get(), e2_id);
}

fn on_set_hook_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().on_set(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.set(Position { x: 10, y: 20 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), *e1.id());
    let v = e1.cloned::<&Position>();
    assert_eq!(v.x, 10);
    assert_eq!(v.y, 20);

    let e2 = world.entity().set(Position { x: 30, y: 40 });
    assert_eq!(world.cloned::<&Count>().0, 2);
    assert_eq!(e_arg.get(), *e2.id());
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 30);
    assert_eq!(v.y, 40);
}

fn on_add_hook_sparse() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_add(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    e.add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);
}

fn on_remove_hook_sparse() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_remove(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e1 = world.entity().add(Position::id());
    world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.remove(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    // remaining entity will be removed when world is dropped, count becomes 2
}

fn on_set_hook_sparse() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_set(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.set(Position { x: 10, y: 20 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    let v = e1.cloned::<&Position>();
    assert_eq!(v.x, 10);
    assert_eq!(v.y, 20);

    let e2 = world.entity().set(Position { x: 30, y: 40 });
    assert_eq!(world.cloned::<&Count>().0, 2);
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 30);
    assert_eq!(v.y, 40);
}

fn on_add_hook_sparse_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_add(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);
    assert_eq!(e_arg.get(), 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), *e1.id());

    e1.add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);

    let e2 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 2);
    assert_eq!(e_arg.get(), *e2.id());
}

fn on_remove_hook_sparse_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_remove(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);
    assert_eq!(e_arg.get(), 0);

    let e1 = world.entity().add(Position::id());
    let e2 = world.entity().add(Position::id());
    let e1_id = *e1.id();
    let e2_id = *e2.id();
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.remove(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), e1_id);

    // e2 will be removed when world is dropped
    drop(world);
    assert_eq!(e_arg.get(), e2_id);
}

fn on_set_hook_sparse_w_entity() {
    let world = World::new();

    world.set(Count(0));

    let e_arg = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let e_arg_clone = e_arg.clone();

    world.component::<Position>().add_trait::<flecs::Sparse>();
    world.component::<Position>().on_set(move |e, _| {
        e_arg_clone.set(*e.id());
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);

    let e1 = world.entity().add(Position::id());
    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.set(Position { x: 10, y: 20 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    assert_eq!(e_arg.get(), *e1.id());
    let v = e1.cloned::<&Position>();
    assert_eq!(v.x, 10);
    assert_eq!(v.y, 20);

    let e2 = world.entity().set(Position { x: 30, y: 40 });
    assert_eq!(world.cloned::<&Count>().0, 2);
    assert_eq!(e_arg.get(), *e2.id());
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 30);
    assert_eq!(v.y, 40);
}

fn chained_hooks() {
    let world = World::new();

    let add_count = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let remove_count = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let set_count = std::rc::Rc::new(std::cell::Cell::new(0i32));

    let add_count_clone = add_count.clone();
    let remove_count_clone = remove_count.clone();
    let set_count_clone = set_count.clone();

    world
        .component::<Position>()
        .on_add(move |_, _| {
            add_count_clone.set(add_count_clone.get() + 1);
        })
        .on_set(move |_, _| {
            set_count_clone.set(set_count_clone.get() + 1);
        })
        .on_remove(move |_, _| {
            remove_count_clone.set(remove_count_clone.get() + 1);
        });

    let e = world.entity();
    assert_eq!(add_count.get(), 0);
    assert_eq!(set_count.get(), 0);
    assert_eq!(remove_count.get(), 0);

    e.add(Position::id());
    assert_eq!(add_count.get(), 1);
    assert_eq!(set_count.get(), 0);
    assert_eq!(remove_count.get(), 0);

    e.set(Position { x: 10, y: 20 });
    assert_eq!(add_count.get(), 1);
    assert_eq!(set_count.get(), 1);
    assert_eq!(remove_count.get(), 0);

    e.remove(Position::id());
    assert_eq!(add_count.get(), 1);
    assert_eq!(set_count.get(), 1);
    assert_eq!(remove_count.get(), 1);
}

fn ctor_w_2_worlds() {
    {
        let world = world_new();

        test_pod_ctor(0);

        world.entity().add(PodDefaultCloneDrop::id());
        test_pod_ctor(1);
    }

    reset_pod_counters();

    {
        let world = world_new();

        test_pod_ctor(0);

        world.entity().add(PodDefaultCloneDrop::id());
        test_pod_ctor(1);
    }
}

fn ctor_w_2_worlds_explicit_registration() {
    {
        let world = world_new();

        world.component::<PodDefaultCloneDrop>();
        test_pod_ctor(0);

        world.entity().add(PodDefaultCloneDrop::id());
        test_pod_ctor(1);
    }

    reset_pod_counters();

    {
        let world = world_new();

        world.component::<PodDefaultCloneDrop>();
        test_pod_ctor(0);

        world.entity().add(PodDefaultCloneDrop::id());
        test_pod_ctor(1);
    }
}

fn defer_set() {
    let world = world_new();
    world.component::<PodDefaultCloneDrop>();

    world.defer_begin();
    let e = world.entity().set(PodDefaultCloneDrop::new(5));
    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);
    assert!(!e.has(PodDefaultCloneDrop::id()));
    world.defer_end();
    assert!(e.has(PodDefaultCloneDrop::id()));
    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);

    e.get::<&PodDefaultCloneDrop>(|pod| {
        assert_eq!(pod.value, 5);
    });

    drop(world);
    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(1);
}

fn set_w_on_add() {
    let world = World::new();

    let e1 = world.entity();

    let on_add = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let on_add_clone = on_add.clone();
    let e1_id = *e1.id();

    world.component::<Position>().on_add(move |e, _| {
        on_add_clone.set(on_add_clone.get() + 1);
        assert_eq!(*e.id(), e1_id);
    });

    e1.set(Position { x: 0, y: 0 });
    assert_eq!(on_add.get(), 1);
}

fn set_w_on_add_existing() {
    let world = World::new();

    let e1 = world.entity().add(Velocity::id());

    let on_add = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let on_add_clone = on_add.clone();
    let e1_id = *e1.id();

    world.component::<Position>().on_add(move |e, _| {
        on_add_clone.set(on_add_clone.get() + 1);
        assert_eq!(*e.id(), e1_id);
    });

    e1.set(Position { x: 0, y: 0 });
    assert_eq!(on_add.get(), 1);
}

fn set_pair_no_copy() {
    let world = world_new();

    let e = world.entity().set_pair::<NoCopy, Tag>(NoCopy::new(100));

    e.get::<&(NoCopy, Tag)>(|no_copy| {
        assert_eq!(no_copy.value, 100);
    })
}

fn set_pair_w_entity_no_copy() {
    let world = World::new();

    let tag = world.entity();

    let e = world.entity().set_first::<NoCopy>(NoCopy::new(10), tag);

    let no_copy = e.get_first_untyped::<NoCopy>(tag) as *const NoCopy;
    unsafe {
        assert_eq!((*no_copy).value, 10);
    }
}

fn set_pair_second_no_copy() {
    let world = World::new();

    let tag = world.entity();

    let e = world.entity().set_second::<NoCopy>(tag, NoCopy::new(10));

    let no_copy = e.get_second_untyped::<NoCopy>(tag) as *const NoCopy;
    unsafe {
        assert_eq!((*no_copy).value, 10);
    }
}

fn set_override_no_copy() {
    let world = World::new();

    let e = world.entity().set_auto_override(NoCopy::new(100));

    e.get::<&NoCopy>(|no_copy| {
        assert_eq!(no_copy.value, 100);
    });

    let no_copy_id = world.component_id::<NoCopy>();
    assert!(e.has(flecs::id_flags::AutoOverride::ID | *no_copy_id));
}

fn set_override_pair_no_copy() {
    let world = World::new();

    let e = world
        .entity()
        .set_pair_override::<NoCopy, Tag>(NoCopy::new(10));

    e.get::<&(NoCopy, Tag)>(|no_copy| {
        assert_eq!(no_copy.value, 10);
    });

    let no_copy_id = world.component_id::<NoCopy>();
    let tag_id = world.component_id::<Tag>();
    let pair_id = ecs_pair(*no_copy_id, *tag_id);
    assert!(e.has(flecs::id_flags::AutoOverride::ID | pair_id));
}

fn set_override_pair_w_entity_no_copy() {
    let world = World::new();

    let tag = world.entity();

    let e = unsafe {
        world
            .entity()
            .set_auto_override_second::<NoCopy>(NoCopy::new(100), tag)
    };

    let no_copy = e.get_second_untyped::<NoCopy>(tag) as *const NoCopy;
    unsafe {
        assert_eq!((*no_copy).value, 100);
    }

    let no_copy_id = world.component_id::<NoCopy>();
    let pair_id = ecs_pair(*tag.id(), *no_copy_id);
    assert!(e.has(flecs::id_flags::AutoOverride::ID | pair_id));
}

fn dtor_after_defer_set() {
    {
        let world = world_new();

        let e = world.entity();

        world.defer_begin();
        e.set(PodDefaultCloneDrop::new(10));
        assert!(!e.has(PodDefaultCloneDrop::id()));
        test_pod_ctor(1);
        test_pod_drop(0);
        test_pod_clone(0);
        world.defer_end();

        assert!(e.has(PodDefaultCloneDrop::id()));
        test_pod_ctor(1);
        test_pod_drop(0);
        test_pod_clone(0);

        e.get::<&PodDefaultCloneDrop>(|pod| {
            assert_eq!(pod.value, 10);
        });

        test_pod_ctor(1);
        test_pod_drop(0);
        test_pod_clone(0);
    }

    test_pod_ctor(1);
    test_pod_drop(1);
    test_pod_clone(0);
}

fn dtor_with_relation() {
    {
        let world = world_new();

        let e = world.entity();
        let e2 = world.entity().set(PodDefaultCloneDrop::new(5));

        e.set(PodDefaultCloneDrop::new(100)).add((Tag, e2));

        test_pod_ctor(2);
        test_pod_drop(0);

        e.get::<&PodDefaultCloneDrop>(|pod| {
            assert_eq!(pod.value, 100);
        });

        test_pod_ctor(2);
        test_pod_drop(0);
    }
    test_pod_ctor(2);
    test_pod_drop(2);
}

fn dtor_relation_target() {
    {
        let world = world_new();

        let e = world.entity();
        let e2 = world.entity().set(NoDefaultInvoked::new(5)).add((Tag, e));
        world.entity().set(NoDefaultInvoked::new(5));

        test_no_default_invoked_ctor(2);
        test_no_default_invoked_clone(0);
        test_no_default_invoked_drop(0);

        e2.get::<&NoDefaultInvoked>(|val| {
            assert_eq!(val.value, 5);
        });

        test_no_default_invoked_ctor(2);
        test_no_default_invoked_clone(0);
        test_no_default_invoked_drop(0);

        e.destruct();

        test_no_default_invoked_ctor(2);
        test_no_default_invoked_clone(0);
    }

    test_no_default_invoked_ctor(2);
    test_no_default_invoked_clone(0);
    test_no_default_invoked_drop(2);
}

fn sparse_component() {
    let world = world_new();

    world
        .component::<PodDefaultCloneDrop>()
        .add_trait::<flecs::Sparse>();

    let e = world.entity().add(PodDefaultCloneDrop::id());
    assert_ne!(e.id(), 0);
    assert!(e.has(PodDefaultCloneDrop::id()));

    e.get::<&PodDefaultCloneDrop>(|pod| {
        assert_eq!(pod.value, 10);
    });

    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);

    e.add(Position::id());

    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(0);

    e.remove(PodDefaultCloneDrop::id());

    test_pod_ctor(1);
    test_pod_clone(0);
    test_pod_drop(1);
}

fn count_in_add_hook() {
    let world = World::new();

    let count = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let count_clone = count.clone();

    world.component::<Position>().on_add(move |e, _| {
        count_clone.set(e.world().count(Position::id()) as i32);
    });

    world.entity().set(Position { x: 1, y: 2 });
    assert_eq!(count.get(), 1);

    let mut matched = 0;
    world.new_query::<&Position>().each(|_| {
        matched += 1;
    });

    assert_eq!(matched, 1);
}

fn count_in_remove_hook() {
    let world = World::new();

    let count = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let count_clone = count.clone();

    world.component::<Position>().on_remove(move |e, _| {
        count_clone.set(e.world().count(Position::id()) as i32);
    });

    let ent = world.entity().set(Position { x: 1, y: 2 });
    assert_eq!(count.get(), 0);

    ent.destruct();
    assert_eq!(count.get(), 1);

    let mut matched = 0;
    world.new_query::<&Position>().each(|_| {
        matched += 1;
    });

    assert_eq!(matched, 0);
}

fn set_multiple_hooks() {
    let world = world_new();

    let pod = world.component::<PodDefaultCloneDrop>();

    let adds = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let sets = std::rc::Rc::new(std::cell::Cell::new(0i32));
    let removes = std::rc::Rc::new(std::cell::Cell::new(0i32));

    let adds_clone = adds.clone();
    let sets_clone = sets.clone();
    let removes_clone = removes.clone();

    pod.on_add(move |_, _| {
        adds_clone.set(adds_clone.get() + 1);
    });

    pod.on_set(move |_, _| {
        sets_clone.set(sets_clone.get() + 1);
    });

    pod.on_remove(move |_, _| {
        removes_clone.set(removes_clone.get() + 1);
    });

    world.entity().add(PodDefaultCloneDrop::id());
    assert_eq!(adds.get(), 1);

    world.entity().set(PodDefaultCloneDrop::default());
    assert_eq!(adds.get(), 2);
    assert_eq!(sets.get(), 1);

    drop(world);
    assert_eq!(removes.get(), 2);
}

fn on_replace_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_replace(|e, t1, t2| {
        println!(
            "on_replace called for entity {:?}, from {:?} to {:?}",
            e.id(),
            t1,
            t2
        );
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    let e1 = world.entity().set(Position { x: 1, y: 2 });

    assert_eq!(world.cloned::<&Count>().0, 0);

    e1.set(Position { x: 3, y: 4 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    let v = e1.cloned::<&Position>();
    assert_eq!(v.x, 3);
    assert_eq!(v.y, 4);

    let e2 = world.entity().set(Position { x: 5, y: 6 });
    assert_eq!(world.cloned::<&Count>().0, 1);
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 5);
    assert_eq!(v.y, 6);
    e2.set(Position { x: 7, y: 8 });
    assert_eq!(world.cloned::<&Count>().0, 2);
    let v = e2.cloned::<&Position>();
    assert_eq!(v.x, 7);
    assert_eq!(v.y, 8);
}

/*
const void* ecs_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    ecs_id_t component)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    flecs_assert_entity_valid(world, entity, "get");
    ecs_check(ecs_id_is_valid(world, component) || ecs_id_is_wildcard(component),
        ECS_INVALID_PARAMETER, NULL);

    world = ecs_get_world(world);

    ecs_record_t *r = flecs_entities_get(world, entity);
    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    ecs_table_t *table = r->table;
    ecs_assert(table != NULL, ECS_INTERNAL_ERROR, NULL);

    if (component < FLECS_HI_COMPONENT_ID) {
        if (!world->non_trivial_lookup[component]) {
            ecs_get_low_id(table, r, component);
            return NULL;
        }
    }

    ecs_component_record_t *cr = flecs_components_get(world, component);
    if (!cr) {
        return NULL;
    }

    if (cr->flags & EcsIdDontFragment) {
        void *ptr = flecs_component_sparse_get(world, cr, table, entity);
        if (ptr) {
            return ptr;
        }
    }

    const ecs_table_record_t *tr = flecs_component_get_table(cr, table);
    if (!tr) {
        return flecs_get_base_component(world, table, component, cr, 0);
    } else {
        if (cr->flags & EcsIdSparse) {
            return flecs_component_sparse_get(world, cr, table, entity);
        }
        ecs_check(tr->column != -1, ECS_INVALID_PARAMETER,
            "component '%s' passed to get() is a tag/zero sized",
                flecs_errstr(ecs_id_str(world, component)));
    }

    int32_t row = ECS_RECORD_TO_ROW(r->row);
    return flecs_table_get_component(table, tr->column, row).ptr;
error:
    return NULL;
}

const void* ecs_record_get_id(
    const ecs_world_t *stage,
    const ecs_record_t *r,
    ecs_id_t component)
{
    const ecs_world_t *world = ecs_get_world(stage);
    ecs_component_record_t *cr = flecs_components_get(world, component);
    return flecs_get_component(
        world, r->table, ECS_RECORD_TO_ROW(r->row), cr);
}
*/
