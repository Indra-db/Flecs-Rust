mod common;
use common::*;

pub fn add_remove_hooks_components(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "add_remove_hooks_components");

    bench_add_remove_hooks!(group, "1", 1);
    bench_add_remove_hooks!(group, "2", 2);
    bench_add_remove_hooks!(group, "16", 16);
    bench_add_remove_hooks!(group, "64", 64);

    group.finish();
}

criterion_group!(benches, add_remove_hooks_components);

criterion_main!(benches);
