use crate::common_bench::*;

pub fn add_remove_hooks_components(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add_remove_hooks_components");

    bench_add_remove_hooks!(group, "1", 1);
    bench_add_remove_hooks!(group, "2", 2);
    bench_add_remove_hooks!(group, "16", 16);
    bench_add_remove_hooks!(group, "64", 64);

    group.finish();
}
