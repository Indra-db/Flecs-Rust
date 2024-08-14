#[test]
fn query_flags() {
    assert_eq!(flecs_ecs_sys::EcsQueryMatchPrefab, 2);
    assert_eq!(flecs_ecs_sys::EcsQueryMatchDisabled, 4);
    assert_eq!(flecs_ecs_sys::EcsQueryMatchEmptyTables, 8);
    assert_eq!(flecs_ecs_sys::EcsQueryAllowUnresolvedByName, 64);
    assert_eq!(flecs_ecs_sys::EcsQueryTableOnly, 128);
}
